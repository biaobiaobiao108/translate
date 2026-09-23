mod api;
mod cli;
mod db;
mod error;
mod tui;
mod views;

use std::process::ExitCode;

use api::client::build_client;
use api::dict::smart_query;
use clap::Parser;
use db::Database;
use error::Result;
use views::cli_render::{render_cli_output, render_history_items};

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[错误] 执行失败: {}", error);
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<()> {
    let args = cli::CliArgs::parse();

    // 历史记录查看不需要创建 HTTP 客户端，避免无关的代理配置影响本地操作。
    if args.show_history || args.show_favorites {
        let db = Database::init()?;
        let theme = resolve_theme(args.theme, &db);
        let items = db.list_history(args.show_favorites, 50)?;
        render_history_items(&items, args.show_favorites, theme);
        return Ok(());
    }

    if args.query.is_empty() {
        println!("[提示] 请输入要查询的单词或句子。例如:");
        println!("   tran hello");
        println!("   tran 苹果");
        println!("   tran -s \"To be, or not to be, that is the question.\"");
        println!("   tran i   (进入功能更强大的交互式 TUI 界面)");
        println!("   tran --help");
        return Ok(());
    }

    let client = build_client(args.proxy.as_deref())?;
    let db = Database::init()?;
    let theme = resolve_theme(args.theme, &db);

    if !args.sentence && args.query.len() == 1 && args.query[0] == "i" {
        tui::run_tui(client, db, theme).await?;
        return Ok(());
    }

    let query_text = args.query.join(" ");
    let output = smart_query(&client, &query_text, args.sentence).await?;
    render_cli_output(&output, theme);

    if let Err(error) = db.add_record(&query_text, &output.summary()) {
        eprintln!("[警告] 历史记录保存失败: {}", error);
    }

    Ok(())
}

fn resolve_theme(
    cli_theme: Option<views::theme::ThemeMode>,
    db: &Database,
) -> views::theme::ThemeMode {
    if let Some(theme) = cli_theme {
        return theme;
    }

    if let Ok(Some(saved)) = db.get_config("theme") {
        if let Ok(theme) = saved.parse::<views::theme::ThemeMode>() {
            return theme;
        }
    }

    views::theme::ThemeMode::Auto
}

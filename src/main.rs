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
use colored::*;
use crossterm::terminal;
use db::Database;
use error::Result;
use views::cli_render::render_cli_output;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{} {}", "❌ 执行失败:".red().bold(), error);
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<()> {
    let args = cli::CliArgs::parse();

    // 历史记录查看不需要创建 HTTP 客户端，避免无关的代理配置影响本地操作。
    if args.show_history || args.show_favorites {
        let db = Database::init()?;
        let items = db.list_history(args.show_favorites, 50)?;
        let title = if args.show_favorites {
            "⭐ 收藏生词本"
        } else {
            "📜 历史查询记录"
        };
        println!("\n  {} (共 {} 条)", title.bold().cyan(), items.len());
        println!(
            "  {}",
            "─"
                .repeat(
                    terminal::size()
                        .map(|(width, _)| usize::from(width).clamp(40, 120))
                        .unwrap_or(50)
                )
                .truecolor(90, 100, 138)
        );
        for item in items {
            let fav = if item.is_favorite {
                "★".yellow()
            } else {
                " ".normal()
            };
            println!(
                "  {} {}  {} {}",
                fav,
                item.query,
                item.result_summary,
                item.created_at.truecolor(115, 125, 160)
            );
        }
        println!();
        return Ok(());
    }

    if args.query.is_empty() {
        println!("{}", "💡 提示: 请输入要查询的单词或句子。例如:".yellow());
        println!("   tran hello");
        println!("   tran 苹果");
        println!("   tran -s \"To be, or not to be, that is the question.\"");
        println!("   tran i   (进入功能更强大的交互式 TUI 界面)");
        println!("   tran --help");
        return Ok(());
    }

    let client = build_client(args.proxy.as_deref())?;
    let db = Database::init()?;

    if !args.sentence && args.query.len() == 1 && args.query[0] == "i" {
        tui::run_tui(client, db).await?;
        return Ok(());
    }

    let query_text = args.query.join(" ");
    let output = smart_query(&client, &query_text, args.sentence).await?;
    render_cli_output(&output);

    if let Err(error) = db.add_record(&query_text, &output.summary()) {
        eprintln!("{} {}", "⚠ 历史记录保存失败:".yellow(), error);
    }

    Ok(())
}

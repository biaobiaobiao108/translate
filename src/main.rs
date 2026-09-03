mod api;
mod cli;
mod db;
mod error;
mod tui;
mod views;

use clap::Parser;
use colored::*;
use api::client::build_client;
use api::dict::{smart_query, QueryOutput};
use cli::CliArgs;
use db::Database;
use views::cli_render::render_cli_output;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse();
    let db = Database::init()?;
    let client = build_client(args.proxy.as_deref())?;

    // 检查是否进入 TUI 模式 (translate i)
    let is_tui = args.query.len() == 1 && args.query[0] == "i";

    if is_tui {
        tui::run_tui(client, db).await?;
        return Ok(());
    }

    // 历史记录查看
    if args.show_history || args.show_favorites {
        let items = db.list_history(args.show_favorites, 50)?;
        let title = if args.show_favorites { "⭐ 收藏生词本" } else { "📜 历史查询记录" };
        println!("\n  {} (共 {} 条)", title.bold().cyan(), items.len());
        println!("  {}", "─".repeat(50).dimmed());
        for item in items {
            let fav = if item.is_favorite { "★".yellow() } else { " ".normal() };
            println!("  {} {:<16} {} {}", fav, item.query.bold().white(), item.result_summary.dimmed(), item.created_at.dimmed());
        }
        println!();
        return Ok(());
    }

    // 处理查询内容
    if args.query.is_empty() {
        println!("{}", "💡 提示: 请输入要查询的单词或句子。例如:".yellow());
        println!("   translate hello");
        println!("   translate 苹果");
        println!("   translate -s \"To be, or not to be, that is the question.\"");
        println!("   translate i   (进入功能更强大的交互式 TUI 界面)");
        println!("   translate --help");
        return Ok(());
    }

    let query_text = args.query.join(" ");
    match smart_query(&client, &query_text, args.sentence).await {
        Ok(output) => {
            render_cli_output(&output);

            // 记录到本地数据库
            let summary = match &output {
                QueryOutput::Dict(d) => {
                    let mut s = String::new();
                    for def in &d.definitions {
                        s.push_str(&def.pos);
                        s.push_str(&def.meanings.join(" "));
                        s.push(' ');
                    }
                    s
                }
                QueryOutput::Sentence { translated, .. } => translated.clone(),
            };
            let _ = db.add_record(&query_text, &summary);
        }
        Err(e) => {
            eprintln!("\n{} {}\n", "❌ 查询失败:".red().bold(), e);
        }
    }

    Ok(())
}

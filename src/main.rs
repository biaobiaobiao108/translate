mod api;
mod cli;
mod error;
mod views;

use std::process::ExitCode;

use api::client::build_client;
use api::dict::smart_query;
use clap::Parser;
use error::Result;
use views::cli_render::render_cli_output;

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

    if args.query.is_empty() {
        println!("[提示] 请输入要查询的单词或句子。例如:");
        println!("   tran hello");
        println!("   tran 苹果");
        println!("   tran -s \"To be, or not to be, that is the question.\"");
        println!("   tran --help");
        return Ok(());
    }

    let client = build_client(args.proxy.as_deref())?;
    let theme = args.theme.unwrap_or_default();

    let query_text = args.query.join(" ");
    let output = smart_query(&client, &query_text, args.sentence).await?;
    render_cli_output(&output, theme);

    Ok(())
}

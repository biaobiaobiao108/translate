mod api;
mod cli;
mod error;
mod text;
mod views;

use std::io::{self, Write};
use std::process::ExitCode;

use api::client::build_client;
use api::dict::smart_query;
use clap::Parser;
use error::{AppError, Result};
use views::cli_render::render_cli_output;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(code) => code,
        Err(AppError::Io(error)) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "[错误] 执行失败: {}", error);
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<ExitCode> {
    let args = cli::CliArgs::parse();

    if args.query.is_empty() {
        print_usage_hint()?;
        return Ok(ExitCode::from(2));
    }

    let client = build_client(args.proxy.as_deref())?;
    let theme = args.theme.unwrap_or_default();

    let query_text = args.query.join(" ");
    let output = smart_query(&client, &query_text, args.sentence).await?;
    render_cli_output(&output, theme)?;

    Ok(ExitCode::SUCCESS)
}

fn print_usage_hint() -> io::Result<()> {
    let stdout = io::stdout();
    let mut output = stdout.lock();
    writeln!(output, "[提示] 请输入要查询的单词或句子。例如:")?;
    writeln!(output, "   tran hello")?;
    writeln!(output, "   tran 苹果")?;
    writeln!(
        output,
        "   tran -s \"To be, or not to be, that is the question.\""
    )?;
    writeln!(output, "   tran --help")
}

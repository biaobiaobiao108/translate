use clap::Parser;

use crate::views::theme::ThemeMode;

#[derive(Parser, Debug)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    version,
    about = "现代快速、美观的 CLI 翻译与词典工具"
)]
pub struct CliArgs {
    /// 设置终端配色：auto（智能检测）、dark（深色）或 light（浅色）
    #[arg(long = "theme", value_enum, help = "设置终端主题：auto、dark 或 light")]
    pub theme: Option<ThemeMode>,

    /// 强制以句子模式进行微软翻译
    #[arg(short = 's', long = "sentence", help = "强制将输入作为整句进行翻译")]
    pub sentence: bool,

    /// 待查询的单词、短语或句子
    #[arg(trailing_var_arg = true)]
    pub query: Vec<String>,

    /// 指定网络代理 (如 http://127.0.0.1:7890 或 socks5://127.0.0.1:1080)
    #[arg(long = "proxy", help = "设置 HTTP/SOCKS5 代理地址")]
    pub proxy: Option<String>,
}

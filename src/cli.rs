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
    pub query: Vec<String>,

    /// 指定网络代理 (如 http://127.0.0.1:7890 或 socks5://127.0.0.1:1080)
    #[arg(long = "proxy", help = "设置 HTTP/SOCKS5 代理地址")]
    pub proxy: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_options_after_query() {
        let args = CliArgs::try_parse_from([
            "tran",
            "rust",
            "--theme",
            "dark",
            "--proxy",
            "http://127.0.0.1:7890",
        ])
        .unwrap();

        assert_eq!(args.query, vec!["rust"]);
        assert_eq!(args.theme, Some(ThemeMode::Dark));
        assert_eq!(args.proxy.as_deref(), Some("http://127.0.0.1:7890"));
    }

    #[test]
    fn accepts_hyphen_leading_query_after_separator() {
        let args = CliArgs::try_parse_from(["tran", "--", "-hello"]).unwrap();
        assert_eq!(args.query, vec!["-hello"]);
    }
}

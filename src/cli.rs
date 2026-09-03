use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "translate", version = "0.1.0", about = "现代快速、美观的 CLI / TUI 翻译与词典工具")]
pub struct CliArgs {
    /// 强制以句子模式进行 Google 翻译
    #[arg(short = 's', long = "sentence", help = "强制将输入作为整句进行翻译")]
    pub sentence: bool,

    /// 待查询的单词、短语、句子或 'i' (进入 TUI 模式)
    #[arg(trailing_var_arg = true)]
    pub query: Vec<String>,

    /// 指定网络代理 (如 http://127.0.0.1:7890 或 socks5://127.0.0.1:1080)
    #[arg(long = "proxy", help = "设置 HTTP/SOCKS5 代理地址")]
    pub proxy: Option<String>,

    /// 查看查询历史记录 (CLI 模式)
    #[arg(long = "history", help = "显示最近的查询历史记录")]
    pub show_history: bool,

    /// 仅查看已收藏的生词记录
    #[arg(long = "favorites", help = "仅显示收藏的生词")]
    pub show_favorites: bool,
}

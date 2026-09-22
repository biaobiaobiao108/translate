use clap::Parser;

use crate::views::theme::ThemeMode;

#[derive(Parser, Debug)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    version,
    about = "现代快速、美观的 CLI / TUI 翻译与词典工具"
)]
pub struct CliArgs {
    /// 选择终端配色：auto 继承终端背景，dark / light 使用固定高对比配色
    #[arg(
        long = "theme",
        value_enum,
        default_value_t = ThemeMode::Auto,
        help = "设置终端主题：auto、dark 或 light"
    )]
    pub theme: ThemeMode,

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
    #[arg(
        long = "history",
        conflicts_with = "show_favorites",
        help = "显示最近的查询历史记录"
    )]
    pub show_history: bool,

    /// 仅查看已收藏的生词记录
    #[arg(
        long = "favorites",
        conflicts_with = "show_history",
        help = "仅显示收藏的生词"
    )]
    pub show_favorites: bool,
}

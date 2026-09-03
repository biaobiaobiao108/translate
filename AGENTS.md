# AGENTS.md

## 项目简介

translate 是一个基于 Rust 开发的高性能、现代化 CLI / TUI 翻译工具，采用优雅的 Tokyo Night 配色方案。
- **CLI 模式**：极速单词详尽词典卡片查询（音标、词性、权威例句）、长句中英互译；
- **TUI 模式**：左右等宽（50/50）双栏沉浸式对照编辑、50+ 行长文本视口跟随滚动、微秒级批量粘贴、输入法（IME）精确跟随定位、生词本与历史抽屉管理。

---

## 核心设计与技术栈

- **编程语言**：Rust (Edition 2021)
- **TUI 框架**：ratatui (0.29), crossterm (0.28), tui-textarea (0.7.0)
- **网络与接口**：
  - reqwest (0.12, rustls-tls)：支持代理与系统环境检测；
  - Google 公共翻译接口：智能中英互译；
  - 有道开放词典 JSON 接口：权威英美音标、词性分类释义、中英权威双语例句。
- **本地存储**：rusqlite (SQLite3 bundled)，自动存储历史记录与收藏生词本至 ~/.translate/history.db。
- **文字与光标**：unicode-width 精确度量终端全角/半角字符宽度，严格同步物理硬件光标。

---

## 目录结构

`
.
├── .github/
│   └── workflows/
│       └── release.yml      # 多平台独立二进制 CI/CD 打包流程
├── src/
│   ├── api/
│   │   ├── client.rs        # HTTP 客户端封装（代理支持）
│   │   ├── dict.rs          # 词典查询与智能分流
│   │   ├── google.rs        # Google 翻译接口
│   │   └── mod.rs
│   ├── cli/
│   │   ├── args.rs          # 命令行参数解析（clap）
│   │   └── mod.rs
│   ├── db/
│   │   ├── history.rs       # SQLite 历史与收藏夹持久化
│   │   └── mod.rs
│   ├── error.rs             # 统一异常定义
│   ├── main.rs              # 程序入口
│   ├── tui/
│   │   ├── app.rs           # TUI 状态机、长文折行与滚动算法
│   │   ├── event.rs         # 异步事件循环、微秒级按键流聚合与粘贴捕获
│   │   ├── mod.rs           # 键盘事件分发与主循环
│   │   └── ui.rs            # Tokyo Night 双栏渲染、输入法光标锚定
│   └── views/
│       ├── cli_render.rs    # CLI 词典卡片与译文渲染
│       ├── mod.rs
│       └── theme.rs         # Tokyo Night 标准色板
├── .gitignore
├── AGENTS.md
└── Cargo.toml
`

---

## 开发与构建规范

1. **构建要求**：
   - 依赖 Rust 稳定版工具链；
   - Windows 环境优先使用 x86_64-pc-windows-gnu (MinGW GCC) 或 MSVC；
   - Linux / macOS 直接使用标准稳定版工具链构建。
2. **构建命令**：
   - 开发调试：cargo check / cargo run -- <args>
   - 生产发布：cargo build --release
3. **交互规范**：
   - 终端输入长文本时，必须避免频繁触发网络请求；
   - 任何涉及字符索引的操作，必须保证 UTF-8 字符边界安全；
   - 保持 Tokyo Night 配色规范统一。

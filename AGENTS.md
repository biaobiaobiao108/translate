# AGENTS.md

## 项目简介

tran 是一个基于 Rust 开发的高性能、现代化 CLI 翻译工具，采用优雅的 Tokyo Night 配色方案。
- **CLI 模式**：极速单词详尽词典卡片查询（音标、词性、权威例句）、长句中英互译；

---

## 核心设计与技术栈

- **编程语言**：Rust (Edition 2021)
- **终端渲染**：colored、unicode-width，支持自适应明暗主题与中英文显示宽度换行
- **网络与接口**：
  - reqwest (0.12, rustls-tls)：支持代理与系统环境检测；
  - 微软翻译接口 (Bing Translator)：智能中英互译，国内直连免翻墙；
  - 有道开放词典 JSON 接口：权威英美音标、词性分类释义、中英权威双语例句。
- **运行方式**：无状态 CLI，查询结束后不写入本地数据库。
- **文字与排版**：unicode-width 精确度量终端全角/半角字符宽度，保证 CLI 卡片换行稳定。

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
│   │   ├── microsoft.rs     # 微软翻译接口
│   │   └── mod.rs
│   ├── cli.rs               # 命令行参数解析（clap）
│   ├── error.rs             # 统一异常定义
│   ├── main.rs              # 程序入口
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
2. **构建与安装命令**：
   - 开发调试：cargo check / cargo run -- <args>
   - 生产发布：cargo build --release
   - **安装到 PATH**：每次修改完代码并验证通过后，必须执行 `cargo install --path .` 将最新的 `tran` 安装更新到系统 PATH。
   - **版本控制**：每次修改完代码并验证通过后，都必须进行一次 `git commit` 保存变更。
3. **交互规范**：
   - 终端输入长文本时，必须避免频繁触发网络请求；
   - 任何涉及字符索引的操作，必须保证 UTF-8 字符边界安全；
   - 保持 Tokyo Night 配色规范统一。

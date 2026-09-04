# translate · 高性能现代化 CLI / TUI 翻译工具

<div align="center">
  <img src="doc/assets/logo.svg" alt="translate logo" width="180" height="180" />
  <br>
  <p><strong>专为极客与开发者打造的高性能终端翻译利器 · 沉浸式 Tokyo Night 配色 · 原生单一独立二进制</strong></p>
  <p>
    <a href="https://github.com/biaobiaobiao108/translate/releases"><img src="https://img.shields.io/github/v/release/biaobiaobiao108/translate?color=7aa2f7&label=Release" alt="GitHub Release"></a>
    <img src="https://img.shields.io/badge/Language-Rust%202021-e0af68.svg" alt="Rust 2021">
    <img src="https://img.shields.io/badge/Theme-Tokyo%20Night-bb9af7.svg" alt="Tokyo Night">
    <img src="https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-7dcfff.svg" alt="Platform">
    <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-9ece6a.svg" alt="License MIT"></a>
  </p>
</div>

---

## 📖 项目简介

**translate** 是一款基于 Rust 开发的现代化极速 CLI / TUI 翻译与词典工具。它采用优雅深邃的 **Tokyo Night** 主题配色，告别传统繁重缓慢的图形翻译软件，让查词与翻译在终端中达到极致的敏捷与愉悦。

- **CLI 模式**：极速毫秒级响应，单词智能分流呈现详尽权威词典卡片（英美音标、词性精释、权威中英双语例句）；长句自动调用 Google 智能翻译接口。
- **TUI 模式**：左右等宽 (50/50) 沉浸式双栏对照编辑，支持 50+ 行超长文本视口跟随滚动、微秒级批量粘贴注入保护、输入法 (IME) 物理光标精确跟随定位、本地 SQLite 生词本与历史抽屉管理。

---

## ✨ 核心特性

- ⚡ **CLI 极速秒级查词**：智能分流单词与长句。查词立显权威英美双音标、词性分类详尽释义与中英权威双语例句。
- 🎨 **Tokyo Night 经典配色**：全界面严格遵循 Tokyo Night 标准色板（`#1a1b26`、`#7aa2f7`、`#7dcfff`、`#bb9af7`），高对比度柔和护眼。
- 🖥️ **TUI 左右 50/50 双栏沉浸对照**：左栏多行编辑输入，右栏实时呈现翻译与释义，视野开阔无遮挡。
- 📝 **工业级文本编辑**：基于 `tui-textarea` 深度调优，支持长文本跨行移动、平滑自动折行与全向光标漫游。
- 🚀 **毫秒级长文粘贴保护**：底层事件循环全面支持终端括号化粘贴（Bracketed Paste），粘贴海量多行文本无感注入，绝不误触发网络请求。
- 🎯 **输入法 (IME) 精准跟随**：通过 `unicode-width` 动态测量全角中文字符与英文字符真实列宽，硬件物理光标与系统输入法候选框时刻同步定位在字符最右侧。
- ⭐ **本地生词本与历史抽屉**：轻量 SQLite 本地持久化（`~/.translate/history.db`），支持一键收藏生词、历史回溯、条目过滤与一键重查。
- 🌐 **代理与环境感知**：支持 `--proxy` 参数或自动感知系统环境变量（HTTP / SOCKS5），网络访问无缝流畅。

---

## 📥 下载安装与 PATH 配置

### 方式一：下载预编译独立二进制（推荐，开箱即用）

无需安装任何运行时或解压工具，下载对应系统架构的单一文件即可：

| 操作系统 / 架构 | 发布文件 | 说明 |
| :--- | :--- | :--- |
| **Windows 10 / 11 (x86_64)** | [tran-x86_64-pc-windows-msvc.exe](https://github.com/biaobiaobiao108/translate/releases/latest/download/tran-x86_64-pc-windows-msvc.exe) | 下载后重命名为 `tran.exe` 并配置 PATH |
| **macOS (Apple Silicon M1~M4)** | [tran-aarch64-apple-darwin](https://github.com/biaobiaobiao108/translate/releases/latest/download/tran-aarch64-apple-darwin) | 重命名为 `tran` 并移入 `/usr/local/bin` |
| **Linux (Debian / Ubuntu / CentOS x86_64)** | [tran-x86_64-unknown-linux-gnu](https://github.com/biaobiaobiao108/translate/releases/latest/download/tran-x86_64-unknown-linux-gnu) | 重命名为 `tran` 并移入 `/usr/local/bin` |
| **Linux (ARM64 / aarch64)** | [tran-aarch64-unknown-linux-gnu](https://github.com/biaobiaobiao108/translate/releases/latest/download/tran-aarch64-unknown-linux-gnu) | 重命名为 `tran` 并移入 `/usr/local/bin` |

> 提示：也可以前往 [Releases 页面](https://github.com/biaobiaobiao108/translate/releases) 查看所有历史版本与产物校验。

#### 💡 如何配置 PATH，以便在终端中随处使用 `tran` 命令？

二进制程序下载后，默认带有平台后缀文件名且位于下载目录。为了能在任意终端窗口中直接输入 `tran` 运行，需要将其**重命名为 `tran`（Windows 为 `tran.exe`）并移入系统环境变量 PATH 所包含的目录**。各平台操作命令如下：

#### 🪟 Windows 系统
打开 PowerShell（进入你存放下载文件的目录，如 `Downloads`）：

- **方式 A（管理员运行，最省心）：直接移入 `C:\Windows`**
  ```powershell
  Move-Item .\tran-x86_64-pc-windows-msvc.exe "C:\Windows\tran.exe"
  ```
- **方式 B（普通用户权限）：移入用户目录并配置 PATH**
  ```powershell
  # 1. 创建专用工具目录并移动重命名
  New-Item -ItemType Directory -Force -Path "$HOME\bin"
  Move-Item .\tran-x86_64-pc-windows-msvc.exe "$HOME\bin\tran.exe"

  # 2. 将 $HOME\bin 永久加入当前用户的 PATH 环境变量
  [Environment]::SetEnvironmentVariable("Path", [Environment]::GetEnvironmentVariable("Path", "User") + ";$HOME\bin", "User")
  ```
  *(注：如果你的电脑已经装有 Rust，也可以直接移动至 `$HOME\.cargo\bin\tran.exe`，该目录默认已在 PATH 中)*

#### 🍎 macOS 系统 (Apple Silicon)
打开 Terminal 终端，进入下载目录：
```bash
# 1. 赋予执行权限并重命名移动至 /usr/local/bin (系统标准命令目录)
chmod +x tran-aarch64-apple-darwin
sudo mv tran-aarch64-apple-darwin /usr/local/bin/tran

# 2. 若 macOS 拦截提示“无法打开，因为无法验证开发者”，执行以下命令放行：
xattr -d com.apple.quarantine /usr/local/bin/tran
```

#### 🐧 Linux 系统 (Ubuntu / Debian / CentOS / Arch 等)
打开终端，进入下载目录：
```bash
# 赋予执行权限并重命名移动至 /usr/local/bin
chmod +x tran-*-linux-*
sudo mv tran-*-linux-* /usr/local/bin/tran
```

#### ✅ 验证全局命令
完成上述配置后，**新开一个终端窗口**（让 PATH 生效），执行：
```bash
tran --version
# 或直接查询一个单词
tran rust
```
如果正常打印出版本号或 Tokyo Night 配色词典卡片，即代表全局配置成功！

---

### 方式二：通过 Cargo 一键安装（Rust 用户首选）

如果你本地已安装 Rust 开发环境，可直接通过 `cargo` 从 GitHub 仓库一键安装：

```bash
cargo install --git https://github.com/biaobiaobiao108/translate.git tran
```
该命令会自动编译并将二进制安装到 `~/.cargo/bin/tran`，无需手动配置 PATH，开箱即用。

---

### 方式三：从源码编译构建

确保已安装 Rust 稳定版工具链：

```bash
git clone https://github.com/biaobiaobiao108/translate.git
cd translate
cargo build --release
# 编译产物位于 target/release/tran (Windows 下为 tran.exe)
```

或使用 `cargo install` 安装到本地：

```bash
cargo install --path .
```

---

## 🚀 快速上手

### 1. CLI 命令行模式

```bash
# 单词查询：自动展示英美音标、各词性释义、权威双语例句
tran rust

# 句子中英互译：自动检测语种并翻译
tran "Simplicity is prerequisite for reliability."
tran "人生苦短，我用 Rust。"

# 强制句子翻译模式（哪怕是单个单词）
tran -s hello

# 查看最近查询历史记录
tran --history

# 仅查看已收藏的生词记录
tran --favorites

# 使用网络代理
tran rust --proxy http://127.0.0.1:7890
```

### 2. TUI 双栏沉浸模式

直接执行 `tran i` 进入双栏交互界面：

```bash
tran i
```

---

## ⌨️ TUI 快捷键操作指南

### 原文输入区（Input Pane）

| 按键 | 说明 |
| :--- | :--- |
| `Enter` | 执行翻译 / 查词 |
| `Shift + Enter` / `Ctrl + J` | 换行（支持多行排版长段落） |
| `方向键 (↑ ↓ ← →)` | 自由漫游光标 |
| `PageUp / PageDown` | 快速翻页 |
| `Tab` | 在 **[原文输入区]** 与 **[译文释义区]** 之间切换焦点 |
| `Ctrl + H` | 打开 / 关闭 生词本与历史记录抽屉 |
| `Ctrl + C` | 退出程序 |

### 译文与释义区（Result Pane）

| 按键 | 说明 |
| :--- | :--- |
| `↑ / ↓` | 单行平滑滚动浏览长文本 |
| `PageUp / PageDown` | 快速跨页翻页 |
| `Home` | 一键回到顶部 |
| `Tab` / `Esc` | 将焦点切换回输入区 |
| `h` | 打开 / 关闭 生词本与历史记录抽屉 |
| `?` | 打开快捷键帮助弹窗 |
| `q` | 退出程序 |

### 历史记录与生词本抽屉（History Drawer）

| 按键 | 说明 |
| :--- | :--- |
| `↑ / ↓` | 选择历史条目 |
| `PageUp / PageDown` | 快速翻页 |
| `Enter` | 将选中的历史条目载入输入框并自动触发翻译 |
| `f` | 收藏 / 取消收藏当前条目（标记生词 ⭐） |
| `c` | 切换过滤模式（全部历史 ↔ 仅看收藏生词） |
| `d` | 删除当前选中的历史记录 |
| `Esc` / `h` | 关闭抽屉并返回输入区 |

---

## 🌐 在线介绍页

本项目在 [`docs/index.html`](docs/index.html) 提供了现代美观的静态产品展示页。欢迎在浏览器中直接打开预览：

```bash
# Windows
start docs/index.html

# macOS
open docs/index.html

# Linux
xdg-open docs/index.html
```

---

## 🛠️ 技术栈与依赖

- **TUI 界面框架**：[ratatui](https://github.com/ratatui/ratatui) 0.29 + [crossterm](https://github.com/crossterm-rs/crossterm) 0.28
- **文本编辑器**：[tui-textarea](https://github.com/rhysd/tui-textarea) 0.7
- **网络与解析**：[reqwest](https://github.com/seanmonstar/reqwest) 0.12 (rustls-tls) + [serde](https://serde.rs/)
- **本地持久化**：[rusqlite](https://github.com/rusqlite/rusqlite) (SQLite3 Bundled)
- **字体与排版**：[unicode-width](https://github.com/unicode-rs/unicode-width)
- **命令行解析**：[clap](https://github.com/clap-rs/clap) 4.5

---

## 📄 开源许可

本项目遵循 [MIT License](LICENSE) 协议开源。

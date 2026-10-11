# tran · 高性能现代化 CLI 翻译工具

<div align="center">
  <img src="docs/assets/logo.svg" alt="tran logo" width="180" height="180" />
  <br>
  <p><strong>专为极客与开发者打造的高性能终端翻译利器 · 自适应终端配色 · 原生单一独立二进制</strong></p>
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

**tran** 是一款基于 Rust 开发的现代化 CLI 翻译与词典工具。默认根据终端环境选择高可读配色，也支持高对比的 **Tokyo Night 深色** 和浅色主题，让查词与翻译在不同终端中保持清晰、可控。

- **CLI 模式**：极速毫秒级响应，单词智能分流呈现详尽权威词典卡片（英美音标、词性精释、权威中英双语例句）；长句自动调用微软智能翻译接口（Bing Translator，国内直连免代理）。

---

## ✨ 核心特性

- ⚡ **CLI 极速秒级查词**：智能分流单词与长句。查词立显权威英美双音标、词性分类详尽释义与中英权威双语例句。
- 🎨 **自适应终端主题**：默认 `auto` 模式根据终端环境选择对应的高对比字体颜色，也可通过 `--theme dark` 使用 Tokyo Night 深色主题，或通过 `--theme light` 使用浅色高对比主题。
- 🔎 **高可读终端排版**：CLI 按中英文实际显示宽度换行，并根据终端明暗背景选择对应高对比字体颜色，减少不同终端字体与背景造成的阅读问题。
- ⭐ **本地历史与收藏查看**：轻量 SQLite 本地持久化，支持去重、自动保留最近记录、历史回溯与收藏过滤。新安装使用平台数据目录，旧版本的 `~/.translate/history.db` 会继续沿用。
- 🌐 **代理与环境感知**：支持 `--proxy` 参数或自动感知系统环境变量（HTTP / SOCKS5），网络访问无缝流畅。

新安装的数据目录遵循系统约定：Windows 使用 `%LOCALAPPDATA%\\tran`，macOS 使用 `~/Library/Application Support/tran`，Linux 使用 `$XDG_DATA_HOME/tran`（未设置时使用系统默认目录）。

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

> 提示：也可以前往 [Releases 页面](https://github.com/biaobiaobiao108/translate/releases) 查看所有历史版本与发布产物。

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

### 2. 终端主题模式

```bash
# 默认：根据终端环境自动选择深色或浅色高对比配色
tran --theme auto rust

# 固定高对比主题
tran --theme dark rust
tran --theme light rust
```

`auto` 模式会读取常见终端环境提示，并为深色或浅色背景选择对应的字体颜色；无法判断时默认使用深色终端配色。

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

- **网络与解析**：[reqwest](https://github.com/seanmonstar/reqwest) 0.12 (rustls-tls) + [serde](https://serde.rs/)
- **本地持久化**：[rusqlite](https://github.com/rusqlite/rusqlite) (SQLite3 Bundled)
- **字体与排版**：[unicode-width](https://github.com/unicode-rs/unicode-width)
- **命令行解析**：[clap](https://github.com/clap-rs/clap) 4.5

---

## 📄 开源许可

本项目遵循 [MIT License](LICENSE) 协议开源。

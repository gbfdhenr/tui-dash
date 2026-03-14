<div align="center">

# TUI-Dash

**一个用 Rust 编写的终端系统监控面板，提供简洁高效的实时系统信息查看。支持 Windows、Linux 和 macOS**

**A terminal dashboard for system monitoring, written in Rust, providing a simple and efficient real-time system information viewer.**
**Support Windows, Linux and macOS**

</div>

## 功能特性 / Features

### 全面的系统监控 / Comprehensive System Monitoring

- **CPU** / **CPU Usage**
  - 全局 CPU 使用率 / Global CPU usage
  - 每个核心的使用率 / Per-core usage
  - 实时更新 / Real-time updates

- **Memory & Swap** / **内存与交换分区**
  - 已用/总量（GB）/ Used/Total (GB)
  - 使用百分比 / Usage percentage
  - Swap 分区使用情况 / Swap partition usage

- **Disk** / **磁盘**
  - 每个挂载点的使用情况 / Usage per mount point
  - 已用/总量/可用 / Used/Total/Free
  - 使用率百分比 / Usage percentage
  - 高于 80% 时红色警告 / Red warning when usage > 80%
  - 读取/写入速度 / Read/Write speed

- **Network** / **网络**
  - 每个接口的网络流量 / Traffic per interface
  - 接收/发送字节数（MB/KB）/ Received/Sent bytes (MB/KB)
  - 接收/发送速度 / Receive/Transmit speed

- **Process** / **进程监控**
  - 进程列表显示 / Process list display
  - 进程名称、PID、CPU、内存使用率 / Process name, PID, CPU, Memory usage
  - 支持排序和搜索 / Support sorting and searching
  - 实时更新 / Real-time updates

- **Logs** / **系统日志**
  - 最近 500 行系统日志 / Last 500 lines of system logs
  - 支持多种日志类别 / Multiple log categories
  - 自动换行和缩进 / Auto-wrap with indentation
  - 拖动滚动条支持 / Draggable scrollbar support
  - 日志来源：
    - 系统日志（journalctl） / System logs (journalctl)
    - 内核日志 / Kernel logs
    - 错误日志 / Error logs
    - 引导日志 / Boot logs

### 多语言支持 / Multi-language Support

- 自动检测系统语言 / Auto-detect system language
- 支持中文和英文 / Support Chinese and English
- 环境变量：`LANG`, `LC_ALL`, `LC_MESSAGES` / Environment variables

### 鼠标支持 / Mouse Support

- 点击标签页切换 / Click tabs to switch
- 点击日志分类切换 / Click log categories to switch
- 滚轮滚动日志 / Scroll logs with mouse wheel
- 拖动滚动条快速定位 / Drag scrollbar for fast navigation

---

## 安装 / Installation

### 方法一：使用安装脚本（推荐） / Using Install Script (Recommended)

```bash
# 下载安装脚本
curl -sSL https://raw.githubusercontent.com/gbfhdner/tui-dash/main/install.sh -o install.sh

# 运行安装脚本
chmod +x install.sh
./install.sh

# 运行工具
tui-dash
```

### 方法二：从源码编译 / Build from Source

```bash
# 1. 克隆仓库 / Clone the repository
git clone https://github.com/gbfhdner/tui-dash.git
cd tui-dash

# 2. 编译发布版本 / Build release version
cargo build --release

# 3. 运行 / Run
./target/release/tui-dash
```

### 方法三：使用 Cargo 安装 / Install via Cargo

```bash
cargo install tui-dash --git https://github.com/gbfhdner/tui-dash
```
## 系统要求 / System Requirements

### 必需依赖 / Required Dependencies
- **Rust 工具链** (cargo, rustc) - 用于编译
- **Git** - 用于克隆源码

### 可选依赖 / Optional Dependencies
- **Linux**: libssl-dev (Debian/Ubuntu) 或 openssl-devel (RHEL/Fedora)
- **Windows**: 无需额外依赖
- **macOS**: 无需额外依赖

### 终端要求 / Terminal Requirements
- 支持 UTF-8 编码
- 支持 ANSI 颜色
- 推荐使用现代终端：
  - Linux: Alacritty, Kitty, GNOME Terminal
  - Windows: Windows Terminal, PowerShell
  - macOS: iTerm2, Terminal.app

---
Windows 支持 / Windows Support
从 1.0.0 版本开始，TUI-Dash 支持 Windows 10/11/Server 2016 及以上版本。在 Windows 上：

CPU、内存、网络功能通过跨平台库正常支持。

磁盘读写速度使用 WMI 性能计数器获取。

系统日志功能暂不支持（显示提示信息）。

---
常见问题 / FAQ

Q: 终端显示乱码怎么办？/ What if terminal shows garbled text?

A: 请确保：

终端支持 UTF-8 编码

终端支持 ANSI 颜色

使用现代终端（如 Alacritty, Kitty, GNOME Terminal, Windows Terminal 等）

Please ensure:

Terminal supports UTF-8 encoding

Terminal supports ANSI colors

Use a modern terminal (e.g., Alacritty, Kitty, GNOME Terminal, Windows Terminal, etc.)

Q: 如何切换语言？/ How to switch language?

A: 程序会自动检测系统语言。可以通过设置环境变量来强制指定语言：

The program automatically detects system language. You can force specify language via environment variables:

```bash
# 强制使用中文
export LANG=zh_CN.UTF-8
./tui-dash

# Force English
export LANG=en_US.UTF-8
./tui-dash
```

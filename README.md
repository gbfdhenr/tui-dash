<div align="center">

# TUI-Dash

**一个用 Rust 编写的终端系统监控面板，提供简洁高效的实时系统信息查看。支持Windows和Linux**

**A terminal dashboard for system monitoring, written in Rust, providing a simple and efficient real-time system information viewer.Support Windows and Linux**

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

- **Docker** / **Docker 容器**
  - 运行中容器列表 / Running containers list
  - 容器名称和 ID / Container name and ID
  - 运行状态 / Running status
  - CPU 使用率 / CPU usage percentage
  - 内存使用率 / Memory usage percentage
  - 端口映射 / Port mappings

- **Logs** / **系统日志**
  - 最近 500 行系统日志 / Last 500 lines of system logs
  - 支持多种日志类别 / Multiple log categories
  - 自动换行和缩进 / Auto-wrap with indentation
  - 拖动滚动条支持 / Draggable scrollbar support
  - 日志来源：
    - 系统日志（journalctl） / System logs (journalctl)
    - 内核日志 / Kernel logs
    - 错误日志 / Error logs
    - Docker 容器日志 / Docker container logs
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

### 方法一：从源码编译 / Build from Source

```bash
# 1. 克隆仓库 / Clone the repository
git clone https://github.com/gbfdhenr/tui-dash.git
cd tui-dash

# 2. 编译发布版本 / Build release version
cargo build --release

# 3. 运行 / Run
./target/release/tui-dash
```
方法二：使用 Cargo 安装 / Install via Cargo
```bash
cargo install tui-dash --git https://github.com/gbfdhenr/tui-dash
```

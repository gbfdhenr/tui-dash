<div align="center">

# TUI-Dash

**一个用 Rust 编写的终端系统监控面板，提供简洁高效的实时系统信息查看。**

**A terminal dashboard for system monitoring, written in Rust, providing a simple and efficient real-time system information viewer.**

</div>


## 功能特性 / Features

### 📊 全面的系统监控 / Comprehensive System Monitoring

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
  - ⚠️ 高于 80% 时红色警告 / Red warning when usage > 80%
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

### 🌐 多语言支持 / Multi-language Support

- 自动检测系统语言 / Auto-detect system language
- 支持中文和英文 / Support Chinese and English
- 环境变量：`LANG`, `LC_ALL`, `LC_MESSAGES` / Environment variables

### 🖱️ 鼠标支持 / Mouse Support

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


### 方法二：使用 Cargo 安装 / Install via Cargo

```bash
cargo install tui-dash --git https://github.com/gbfdhenr/tui-dash
```
---

## 使用方法 / Usage

### 按键快捷键 / Keyboard Shortcuts

| 按键 | 功能 | Description |
|------|------|-------------|
| `Esc` 或 `q` | 退出程序 | Exit program |
| `Tab` | 切换到下一个标签页 | Switch to next tab |
| `1` - `6` | 数字键直接跳转到对应标签页 | Jump to tab by number |
| `←` / `→` | 切换上一个/下一个标签页 | Previous/Next tab |
| `↑` / `↓` | 向上/向下滚动日志（仅日志标签页） | Scroll logs up/down (Logs tab only) |
| `Home` | 跳转到日志开始 | Jump to start of logs |
| `End` | 跳转到日志结束 | Jump to end of logs |

### 鼠标操作 / Mouse Operations

| 操作 | 功能 | Description |
|------|------|-------------|
| 点击标签 | 切换到对应标签页 | Click tab to switch |
| 点击日志分类 | 切换日志类别（日志标签页） | Click log category to switch |
| 滚轮滚动 | 滚动日志内容 | Scroll logs with wheel |
| 拖动滚动条 | 快速定位日志位置 | Drag scrollbar to navigate |
| 点击滚动条 | 跳转到对应位置 | Click scrollbar to jump |

### 标签页说明 / Tabs Description

| 标签页 | 功能 | Description |
|--------|------|-------------|
| **CPU** | 显示 CPU 使用率和各核心状态 | Display CPU usage and per-core status |
| **Memory** | 显示内存和交换分区使用情况 | Display memory and swap usage |
| **Disk** | 显示磁盘使用情况和读写速度 | Display disk usage and I/O speed |
| **Network** | 显示网络接口流量 | Display network interface traffic |
| **Docker** | 显示 Docker 容器状态 | Display Docker container status |
| **Logs** | 查看系统日志 | View system logs |

### 日志类别说明 / Log Categories

在日志标签页中，可以通过左右箭头或点击子页签切换日志类别：

In the Logs tab, you can switch log categories using left/right arrows or clicking sub-tabs:

| 类别 | 描述 | Description |
|------|------|-------------|
| **System** | 系统日志（journalctl） | System logs (journalctl) |
| **Kernel** | 内核日志 | Kernel logs |
| **Error** | 错误和警告日志 | Error and warning logs |
| **Docker** | Docker 容器日志 | Docker container logs |
| **Boot** | 系统引导日志 | Boot logs |
| **All** | 合并所有日志 | All logs combined |

---

## 系统要求 / System Requirements

### 基本要求 / Basic Requirements

- **操作系统**: Linux（日志功能需要 Linux 系统）/ Operating System: Linux (logs feature requires Linux)
- **Rust 版本**: 1.70 或更高 / Rust version: 1.70 or higher
- **终端**: 支持 ANSI 颜色的终端 / Terminal: ANSI color support required

### 可选依赖 / Optional Dependencies

- **Docker**: 用于显示 Docker 容器信息 / For displaying Docker container information
- **journalctl**: 用于读取系统日志（systemd 系统）/ For reading system logs (systemd systems)

### 支持的 Linux 发行版 / Supported Linux Distributions

- Ubuntu / Debian
- Arch Linux / Manjaro
- Fedora / RHEL / CentOS
- openSUSE
- Kali
- 其他主流 Linux 发行版 / Other mainstream Linux distributions

---

## 常见问题 / FAQ

### Q: 日志功能在非 Linux 系统上能用吗？/ Can logs feature work on non-Linux systems?

**A**: 不行。日志功能依赖 Linux 的 `journalctl` 或 `/var/log/syslog`，仅支持 Linux 系统。

No. The logs feature depends on Linux's `journalctl` or `/var/log/syslog`, and only supports Linux systems.

### Q: Docker 标签页显示错误怎么办？/ What to do if Docker tab shows error?

**A**: 请确保：
1. Docker 服务正在运行
2. 当前用户有 Docker 访问权限
3. Docker 守护进程可访问

Please ensure:
1. Docker service is running
2. Current user has Docker access permissions
3. Docker daemon is accessible

### Q: 如何调整刷新频率？/ How to adjust refresh rate?

**A**: 目前刷新频率固定为 1 秒。如需修改，请编辑 `src/main.rs` 中的 `MIN_UPDATE_INTERVAL` 常量。

Currently the refresh rate is fixed at 1 second. To modify it, edit the `MIN_UPDATE_INTERVAL` constant in `src/main.rs`.

### Q: 终端显示乱码怎么办？/ What if terminal shows garbled text?

**A**: 请确保：
1. 终端支持 UTF-8 编码
2. 终端支持 ANSI 颜色
3. 使用现代终端（如 Alacritty, Kitty, GNOME Terminal 等）

Please ensure:
1. Terminal supports UTF-8 encoding
2. Terminal supports ANSI colors
3. Use a modern terminal (e.g., Alacritty, Kitty, GNOME Terminal, etc.)

### Q: 如何切换语言？/ How to switch language?

**A**: 程序会自动检测系统语言。可以通过设置环境变量来强制指定语言：

The program automatically detects system language. You can force specify language via environment variables:

```bash
# 强制使用中文 / Force Chinese
export LANG=zh_CN.UTF-8
./tui-dash

# 强制使用英文 / Force English
export LANG=en_US.UTF-8
./tui-dash
```

---

## 许可证 / License

本项目采用 MIT 许可证开源。

This project is open-sourced under the MIT License.

```
MIT License

Copyright (c) 2024 gbfdhenr

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## 贡献 / Contributing

欢迎任何形式的贡献！

Any form of contribution is welcome!

### 贡献方式 / How to Contribute

1. **报告问题** / Report Issues
   - 在 [Issues](https://github.com/gbfdhenr/tui-dash/issues) 页面提交 bug 报告或功能建议
   - Submit bug reports or feature suggestions on the [Issues](https://github.com/gbfdhenr/tui-dash/issues) page

2. **提交代码** / Submit Code
   - Fork 本仓库 / Fork this repository
   - 创建特性分支 / Create a feature branch
   - 提交更改 / Commit your changes
   - 推送到分支 / Push to the branch
   - 创建 Pull Request / Create a Pull Request

3. **改进文档** / Improve Documentation
   - 帮助完善 README 和代码注释
   - Help improve README and code comments

### 贡献指南 / Contributing Guidelines

- 遵循 Rust 代码风格 / Follow Rust code style
- 添加必要的测试 / Add necessary tests
- 更新相关文档 / Update relevant documentation
- 保持提交信息清晰 / Keep commit messages clear

---

## 联系方式 / Contact

- **GitHub**: https://github.com/gbfdhenr/tui-dash
- **Issues**: https://github.com/gbfdhenr/tui-dash/issues
- **Discussions**: https://github.com/gbfdhenr/tui-dash/discussions

---

## 致谢 / Acknowledgments

感谢以下开源项目：

Thanks to the following open-source projects:

- [ratatui](https://github.com/ratatui-org/ratatui) - Rust TUI 库 / Rust TUI library
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) - 系统信息库 / System information library
- [crossterm](https://github.com/crossterm-rs/crossterm) - 跨平台终端库 / Cross-platform terminal library
- [bollard](https://github.com/collaboratory/bollard) - Docker 客户端库 / Docker client library

---

## Star History

如果这个项目对你有帮助，请给一个 ⭐️！

If this project helps you, please give it a ⭐️!

[![Star History Chart](https://api.star-history.com/svg?repos=gbfdhenr/tui-dash&type=Date)](https://star-history.com/#gbfdhenr/tui-dash&Date)

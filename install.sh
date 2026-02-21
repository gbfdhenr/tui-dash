#!/bin/bash
set -euo pipefail

# ===================== 基础配置 =====================
# 颜色常量
RED='\033[31m'
GREEN='\033[32m'
YELLOW='\033[33m'
BLUE='\033[34m'
NC='\033[0m'

# 核心配置（请替换为你的 GitHub 仓库信息）
GITHUB_RAW_URL="https://raw.githubusercontent.com/你的用户名/tui-dash/main/new_version_download_url.txt"
TEMP_DIR=$(mktemp -d -t tui-dash-XXXXXX)  # 安全临时目录
DOWNLOAD_FILE="$TEMP_DIR/tui-dash-src.tar.gz"

# 错误处理函数
error_exit() {
    echo -e "\033[31m❌ 错误：$1\033[0m" >&2
    exit 1
}

# 提示函数
info() {
    echo -e "${BLUE}ℹ️ $1${NC}"
}
success() {
    echo -e "${GREEN}✅ $1${NC}"
}
warning() {
    echo -e "${YELLOW}⚠️ $1${NC}"
}

# 清理函数（无论成功/失败都清理临时目录）
cleanup() {
    if [[ -d "$TEMP_DIR" ]]; then
        info "清理临时文件..."
        rm -rf "$TEMP_DIR"
    fi
    if [[ $? -ne 0 ]]; then
        warning "安装失败，清理残留二进制文件..."
        sudo rm /usr/local/bin/tui-dash 2>/dev/null || true
    fi
}
trap cleanup EXIT
check_dependency() {
    local cmd=$1
    local name=$2
    if ! command -v $cmd &> /dev/null; then
        warning "未找到 $name，即将自动安装..."
        if [[ -f /etc/debian_version ]]; then
            sudo apt update || error_exit "更新 apt 源失败"
            sudo apt install -y $name || error_exit "安装 $name 失败"
        elif [[ -f /etc/redhat-release ]]; then
            sudo dnf install -y $name || error_exit "安装 $name 失败"
        else
            error_exit "不支持的发行版，请手动安装 $name"
        fi
    fi
}

info "检查基础依赖..."
check_dependency "curl" "curl"          # 用于下载文件
check_dependency "tar" "tar"            # 用于解压源码包
check_dependency "cargo" "rustc cargo"  # Rust 核心工具

# 检查 sysinfo 系统依赖（Debian 系）
if [[ -f /etc/debian_version ]]; then
    if ! dpkg -l | grep -q libssl-dev; then
        warning "缺少 sysinfo 依赖的 libssl-dev，即将安装..."
        sudo apt install -y libssl-dev pkg-config || error_exit "安装系统库失败"
    fi
fi

# ===================== 读取版本下载链接 =====================
info "读取最新版本下载链接..."
# 下载版本链接文件（超时10秒，失败重试1次）
if ! curl -sSL --max-time 10 --retry 1 -o "$TEMP_DIR/version_url.txt" "$GITHUB_RAW_URL"; then
    error_exit "无法下载版本链接文件，请检查网络或链接是否有效：$GITHUB_RAW_URL"
fi

# 解析链接（过滤空行、注释行，取第一个有效链接）
DOWNLOAD_URL=$(grep -v '^#' "$TEMP_DIR/version_url.txt" | grep -v '^$' | head -n 1 | tr -d ' \r\n')
if [[ -z "$DOWNLOAD_URL" ]]; then
    error_exit "版本链接文件为空或无有效链接，请检查：$GITHUB_RAW_URL"
fi
info "获取到最新版本下载链接：$DOWNLOAD_URL"

# ===================== 下载并解压源码包 =====================
info "下载源码包（临时目录：$TEMP_DIR）..."
if ! curl -sSL --progress-bar --max-time 60 --retry 2 -o "$DOWNLOAD_FILE" "$DOWNLOAD_URL"; then
    error_exit "源码包下载失败，请检查链接是否有效：$DOWNLOAD_URL"
fi
if ! tar tf "$DOWNLOAD_FILE" &> /dev/null; then
    error_exit "源码包损坏或格式错误，无法解压"
fi
info "解压源码包..."
mkdir -p "$TEMP_DIR/src"
tar xzf "$DOWNLOAD_FILE" -C "$TEMP_DIR/src" --strip-components=1 || {
    error_exit "解压源码包失败，请检查压缩包格式（仅支持 .tar.gz/.tar.xz）"
}
SRC_DIR="$TEMP_DIR/src"
if [[ ! -f "$SRC_DIR/Cargo.toml" ]]; then
    error_exit "解压后的目录无 Cargo.toml，不是有效的 Rust 项目"
fi
cd "$SRC_DIR" || error_exit "进入源码目录失败"


info "开始编译 tui-dash（可能需要几分钟）..."
cargo build --release || error_exit "编译失败，请检查 Rust 环境和依赖"

info "安装到系统全局路径..."
sudo cp target/release/tui-dash /usr/local/bin/ || error_exit "复制二进制文件失败"
sudo chmod +x /usr/local/bin/tui-dash || error_exit "添加执行权限失败"

success "tui-dash 安装成功！"
echo
info "📝 快速使用指南："
echo "  - 运行工具：tui-dash"
echo "  - 更新工具：重新执行 ./install.sh"
echo "  - 卸载工具：sudo rm /usr/local/bin/tui-dash"
echo
warning "注意：运行时无需 sudo 权限，若提示系统信息读取失败，请检查 sysinfo 版本兼容性"

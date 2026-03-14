#!/bin/bash
set -euo pipefail

# ===================== 基础配置 =====================
# 颜色常量
RED='\033[31m'
GREEN='\033[32m'
YELLOW='\033[33m'
BLUE='\033[34m'
NC='\033[0m'

# GitHub 仓库配置
GITHUB_REPO="https://github.com/gbfhdner/tui-dash.git"
TEMP_DIR=$(mktemp -d -t tui-dash-XXXXXX)  # 安全临时目录

# 错误处理函数
error_exit() {
    echo -e "${RED}错误：$1${NC}" >&2
    exit 1
}

# 提示函数
info() {
    echo -e "${BLUE}信息：$1${NC}"
}
success() {
    echo -e "${GREEN}成功：$1${NC}"
}
warning() {
    echo -e "${YELLOW}警告：$1${NC}"
}

# 清理函数
cleanup() {
    if [[ -d "$TEMP_DIR" ]]; then
        info "清理临时文件..."
        rm -rf "$TEMP_DIR"
    fi
}
trap cleanup EXIT

# 检查依赖函数
check_dependency() {
    local cmd=$1
    local name=$2
    if ! command -v "$cmd" &> /dev/null; then
        error_exit "未找到 $name，请先安装 $name 后再运行此脚本"
    fi
}

# ===================== 主程序 =====================
echo "=========================================="
echo "      TUI-Dash 安装脚本"
echo "=========================================="
echo

# 检查必需依赖
info "检查必需依赖..."
check_dependency "git" "git"
check_dependency "cargo" "Rust Cargo"

# 检查可选依赖（编译依赖）
info "检查编译依赖..."
if [[ -f /etc/debian_version ]]; then
    # Debian/Ubuntu 系
    if ! dpkg -l | grep -q libssl-dev; then
        warning "缺少 libssl-dev，编译可能需要此依赖"
    fi
elif [[ -f /etc/redhat-release ]]; then
    # RHEL/CentOS/Fedora 系
    if ! rpm -q openssl-devel &> /dev/null; then
        warning "缺少 openssl-devel，编译可能需要此依赖"
    fi
fi

# 克隆仓库
info "从 GitHub 克隆源码..."
if ! git clone --depth 1 "$GITHUB_REPO" "$TEMP_DIR/tui-dash"; then
    error_exit "克隆仓库失败，请检查网络连接和仓库地址"
fi

cd "$TEMP_DIR/tui-dash" || error_exit "进入源码目录失败"

# 检查 Cargo.toml
if [[ ! -f "Cargo.toml" ]]; then
    error_exit "源码目录中未找到 Cargo.toml，不是有效的 Rust 项目"
fi

# 编译项目
info "开始编译 tui-dash（可能需要几分钟）..."
if ! cargo build --release; then
    error_exit "编译失败，请检查 Rust 环境和依赖"
fi

# 安装到系统
info "安装到系统全局路径..."
if [[ -f "target/release/tui-dash" ]]; then
    sudo cp target/release/tui-dash /usr/local/bin/ || error_exit "复制二进制文件失败"
    sudo chmod +x /usr/local/bin/tui-dash || error_exit "添加执行权限失败"
else
    error_exit "编译后的二进制文件未找到"
fi

# 清理编译缓存
info "清理编译缓存..."
cd /tmp || true
rm -rf "$TEMP_DIR"

success "tui-dash 安装成功！"
echo
info "使用说明："
echo "  - 运行工具：tui-dash"
echo "  - 更新工具：重新执行此安装脚本"
echo "  - 卸载工具：sudo rm /usr/local/bin/tui-dash"
echo
info "系统要求："
echo "  - Rust 工具链 (cargo, rustc)"
echo "  - Git"
echo "  - 支持的系统：Linux, Windows, macOS"
echo
echo "=========================================="

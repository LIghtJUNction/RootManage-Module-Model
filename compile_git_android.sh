#!/bin/bash
# filepath: c:\Users\light\Documents\GitHub\RootManage-Module-Model\MyModule\compile_git_android.sh
# Android Git 交叉编译脚本 v4.0 - 完整版
# 支持交叉编译和Android本机编译，修复所有configure问题

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'
BOLD='\033[1m'

# 表情符号
EMOJI_SUCCESS="✅"
EMOJI_ERROR="❌"
EMOJI_WARNING="⚠️"
EMOJI_INFO="ℹ️"
EMOJI_ROCKET="🚀"
EMOJI_GEAR="⚙️"
EMOJI_HAMMER="🔨"
EMOJI_MOBILE="📱"
EMOJI_ANDROID="🤖"

print_header() {
    echo -e "${BOLD}${CYAN}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║            Android Git 编译脚本 v4.0                        ║"
    echo "║    支持交叉编译和Android本机编译（Termux/原生Android）       ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_status() {
    echo -e "${GREEN}${EMOJI_INFO}${NC} ${BOLD}$1${NC}"
}

print_warning() {
    echo -e "${YELLOW}${EMOJI_WARNING}${NC} ${BOLD}$1${NC}"
}

print_error() {
    echo -e "${RED}${EMOJI_ERROR}${NC} ${BOLD}$1${NC}"
}

print_success() {
    echo -e "${GREEN}${EMOJI_SUCCESS}${NC} ${BOLD}$1${NC}"
}

# 检测运行环境
detect_environment() {
    if [ -n "$TERMUX_VERSION" ]; then
        ENV_TYPE="termux"
        return 0
    elif [ -n "$ANDROID_ROOT" ] && [ -d "/system" ]; then
        ENV_TYPE="android_native"
        return 0
    elif grep -q "Microsoft\|WSL" /proc/version 2>/dev/null; then
        ENV_TYPE="wsl"
        return 0
    elif [ "$(uname -s)" = "Linux" ]; then
        ENV_TYPE="linux"
        return 0
    else
        ENV_TYPE="unknown"
        return 1
    fi
}

# 全局变量
COMPILE_MODE=""
ENV_TYPE=""
GIT_VERSION=""
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="$SCRIPT_DIR/android-git-build"
LOG_FILE="$BUILD_DIR/compile.log"

# 检测环境
detect_environment

# 创建构建目录和日志
mkdir -p "$BUILD_DIR"
echo "编译开始时间: $(date)" > "$LOG_FILE"
echo "运行环境: $ENV_TYPE" >> "$LOG_FILE"

# 日志函数
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE"
    echo "$1"
}

# 显示菜单
show_menu() {
    print_header
    
    echo -e "${BOLD}检测到运行环境: ${PURPLE}$ENV_TYPE${NC}"
    echo ""
    echo -e "${BOLD}请选择编译方式：${NC}"
    echo ""
    
    if [ "$ENV_TYPE" = "termux" ]; then
        echo -e "${BLUE}1.${NC} ${EMOJI_ANDROID} Termux本机编译 (推荐，直接在Android上编译)"
        echo -e "${BLUE}2.${NC} ${EMOJI_HAMMER} 静态编译 (如果有交叉编译工具)"
        echo -e "${BLUE}3.${NC} ${EMOJI_MOBILE} 最小化编译"
    elif [ "$ENV_TYPE" = "android_native" ]; then
        echo -e "${BLUE}1.${NC} ${EMOJI_ANDROID} Android原生编译 (需要root和开发工具)"
        echo -e "${BLUE}2.${NC} ${EMOJI_MOBILE} 最小化编译"
    else
        echo -e "${BLUE}1.${NC} ${EMOJI_HAMMER} 静态编译 (推荐用于Android，无依赖)"
        echo -e "${BLUE}2.${NC} ${EMOJI_GEAR} NDK动态编译 (需要Android NDK)"
        echo -e "${BLUE}3.${NC} ${EMOJI_MOBILE} 最小化静态编译 (禁用HTTPS等功能)"
    fi
    
    echo -e "${BLUE}4.${NC} ${EMOJI_ROCKET} 自动选择最佳模式"
    echo -e "${BLUE}5.${NC} ${EMOJI_INFO} 显示系统信息"
    echo -e "${BLUE}q.${NC} 退出"
    echo ""
    
    while true; do
        read -p "请输入选择 (1-5/q): " choice
        case $choice in
            1)
                if [ "$ENV_TYPE" = "termux" ]; then
                    COMPILE_MODE="termux_native"
                    print_status "选择了Termux本机编译模式"
                elif [ "$ENV_TYPE" = "android_native" ]; then
                    COMPILE_MODE="android_native"
                    print_status "选择了Android原生编译模式"
                else
                    COMPILE_MODE="static"
                    print_status "选择了静态编译模式"
                fi
                break
                ;;
            2)
                if [ "$ENV_TYPE" = "termux" ] || [ "$ENV_TYPE" = "android_native" ]; then
                    COMPILE_MODE="minimal"
                    print_status "选择了最小化编译模式"
                else
                    COMPILE_MODE="ndk"
                    print_status "选择了NDK动态编译模式"
                fi
                break
                ;;
            3)
                COMPILE_MODE="minimal"
                print_status "选择了最小化编译模式"
                break
                ;;
            4)
                auto_select_mode
                break
                ;;
            5)
                show_system_info
                continue
                ;;
            q|Q)
                echo "退出编译"
                exit 0
                ;;
            *)
                print_error "无效选择，请重新输入"
                ;;
        esac
    done
}

# 自动选择最佳编译模式
auto_select_mode() {
    print_status "正在自动检测最佳编译模式..."
    
    if [ "$ENV_TYPE" = "termux" ]; then
        COMPILE_MODE="termux_native"
        print_success "检测到Termux环境，选择本机编译"
    elif [ "$ENV_TYPE" = "android_native" ]; then
        COMPILE_MODE="android_native"
        print_success "检测到Android原生环境，选择原生编译"
    elif detect_ndk_environment; then
        COMPILE_MODE="ndk"
        print_success "检测到NDK环境，选择NDK编译"
    elif command -v aarch64-linux-gnu-gcc >/dev/null 2>&1; then
        COMPILE_MODE="static"
        print_success "检测到交叉编译工具，选择静态编译"
    else
        COMPILE_MODE="minimal"
        print_warning "选择最小化编译模式"
    fi
}

# 检测NDK环境
detect_ndk_environment() {
    # 尝试自动检测NDK
    local ndk_paths=(
        "$ANDROID_NDK_HOME"
        "$PWD/android-ndk-r"*
        "$PWD/NDK/android-ndk-r"*
        "$HOME/android-ndk-r"*
        "/opt/android-ndk-r"*
        "$HOME/NDK/android-ndk-r"*
    )
    
    for ndk_path in "${ndk_paths[@]}"; do
        if [ -n "$ndk_path" ] && [ -d "$ndk_path" ]; then
            export ANDROID_NDK_HOME="$ndk_path"
            return 0
        fi
    done
    
    return 1
}

# 显示系统信息
show_system_info() {
    echo -e "${BOLD}${PURPLE}系统信息：${NC}"
    echo "运行环境: $ENV_TYPE"
    echo "操作系统: $(uname -a)"
    echo "架构: $(uname -m)"
    echo "Shell: $SHELL"
    echo "用户: $(whoami)"
    echo "工作目录: $(pwd)"
    echo "脚本目录: $SCRIPT_DIR"
    echo "构建目录: $BUILD_DIR"
    
    if [ "$ENV_TYPE" = "termux" ]; then
        echo "Termux版本: $TERMUX_VERSION"
        echo "Termux前缀: $PREFIX"
    elif [ "$ENV_TYPE" = "android_native" ]; then
        echo "Android根目录: $ANDROID_ROOT"
        echo "Android版本: $(getprop ro.build.version.release 2>/dev/null || echo "未知")"
    fi
    echo ""
    
    echo -e "${BOLD}${PURPLE}编译环境检测：${NC}"
    
    if [ "$ENV_TYPE" = "termux" ]; then
        echo -n "Termux包管理器: "
        if command -v pkg >/dev/null 2>&1; then
            echo -e "${GREEN}${EMOJI_SUCCESS} pkg available${NC}"
        else
            echo -e "${RED}${EMOJI_ERROR} pkg不可用${NC}"
        fi
        
        echo -n "Clang编译器: "
        if command -v clang >/dev/null 2>&1; then
            echo -e "${GREEN}${EMOJI_SUCCESS} $(clang --version | head -n1)${NC}"
        else
            echo -e "${RED}${EMOJI_ERROR} 未安装${NC}"
        fi
    fi
    
    echo -n "GCC交叉编译器: "
    if command -v aarch64-linux-gnu-gcc >/dev/null 2>&1; then
        echo -e "${GREEN}${EMOJI_SUCCESS} $(aarch64-linux-gnu-gcc --version | head -n1)${NC}"
    else
        echo -e "${RED}${EMOJI_ERROR} 未安装${NC}"
    fi
    
    echo -n "Android NDK: "
    if detect_ndk_environment; then
        echo -e "${GREEN}${EMOJI_SUCCESS} $ANDROID_NDK_HOME${NC}"
        if [ -f "$ANDROID_NDK_HOME/source.properties" ]; then
            NDK_VERSION=$(grep "Pkg.Revision" "$ANDROID_NDK_HOME/source.properties" | cut -d'=' -f2 | tr -d ' ')
            echo "  版本: $NDK_VERSION"
        fi
    else
        echo -e "${RED}${EMOJI_ERROR} 未设置或不存在${NC}"
    fi
    
    echo -n "Git: "
    if command -v git >/dev/null 2>&1; then
        echo -e "${GREEN}${EMOJI_SUCCESS} $(git --version)${NC}"
    else
        echo -e "${RED}${EMOJI_ERROR} 未安装${NC}"
    fi
    
    echo ""
}

# 安装依赖
install_dependencies() {
    print_status "检查并安装编译依赖..."
    log "开始安装依赖: $COMPILE_MODE (环境: $ENV_TYPE)"
    
    if [ "$COMPILE_MODE" = "termux_native" ]; then
        install_termux_dependencies
    elif [ "$COMPILE_MODE" = "android_native" ]; then
        install_android_dependencies
    elif [ "$COMPILE_MODE" = "static" ] || [ "$COMPILE_MODE" = "minimal" ]; then
        install_cross_compile_dependencies
    elif [ "$COMPILE_MODE" = "ndk" ]; then
        check_ndk_environment
    fi
}

# 安装Termux依赖
install_termux_dependencies() {
    print_status "安装Termux编译依赖..."
    
    # 更新包管理器
    pkg update || {
        print_error "Termux包更新失败"
        exit 1
    }
    
    # 安装编译工具和依赖
    pkg install -y \
        git \
        clang \
        make \
        autoconf \
        automake \
        libtool \
        pkg-config \
        gettext \
        curl \
        openssl \
        zlib \
        expat \
        libcurl || {
        print_error "Termux依赖安装失败"
        exit 1
    }
    
    print_success "Termux依赖安装完成"
}

# 安装Android原生依赖
install_android_dependencies() {
    print_status "检查Android原生编译环境..."
    
    # 检查是否有root权限
    if ! command -v su >/dev/null 2>&1; then
        print_error "Android原生编译需要root权限"
        exit 1
    fi
    
    # 检查是否有编译工具
    if ! command -v gcc >/dev/null 2>&1 && ! command -v clang >/dev/null 2>&1; then
        print_error "未找到编译器，请使用Termux或安装Android开发工具"
        exit 1
    fi
    
    print_warning "Android原生编译环境有限，建议使用Termux"
}

# 安装交叉编译依赖
install_cross_compile_dependencies() {
    if [ "$COMPILE_MODE" == "static" ] || [ "$COMPILE_MODE" == "minimal" ]; then
        if command -v apt-get >/dev/null 2>&1; then
            print_status "使用apt包管理器安装依赖..."
            
            sudo apt-get update || print_warning "包管理器更新失败"
            sudo apt-get install -y \
                gcc-aarch64-linux-gnu \
                g++-aarch64-linux-gnu \
                build-essential \
                autoconf \
                automake \
                libtool \
                make \
                pkg-config \
                gettext \
                git \
                wget \
                curl \
                cmake \
                python3 \
                python3-pip \
                libssl-dev \
                zlib1g-dev \
                libexpat1-dev \
                libcurl4-openssl-dev || {
                print_error "依赖安装失败"
                exit 1
            }
            
            print_success "交叉编译依赖安装完成"
        elif command -v yum >/dev/null 2>&1; then
            print_status "使用yum包管理器安装依赖..."
            
            sudo yum update -y || print_warning "包管理器更新失败"
            sudo yum groupinstall -y "Development Tools"
            sudo yum install -y \
                gcc-aarch64-linux-gnu \
                autoconf \
                automake \
                libtool \
                make \
                pkg-config \
                gettext \
                git \
                wget \
                curl \
                cmake \
                python3 \
                python3-pip \
                openssl-devel \
                zlib-devel \
                expat-devel \
                libcurl-devel || {
                print_error "依赖安装失败"
                exit 1
            }
            
            print_success "yum依赖安装完成"
        elif command -v pacman >/dev/null 2>&1; then
            print_status "使用pacman包管理器安装依赖..."
            
            sudo pacman -Syu --noconfirm || print_warning "包管理器更新失败"
            sudo pacman -S --noconfirm \
                base-devel \
                aarch64-linux-gnu-gcc \
                autoconf \
                automake \
                libtool \
                make \
                pkg-config \
                gettext \
                git \
                wget \
                curl \
                cmake \
                python \
                python-pip \
                openssl \
                zlib \
                expat \
                curl || {
                print_error "依赖安装失败"
                exit 1
            }
            
            print_success "pacman依赖安装完成"
        else
            print_error "不支持的包管理器，请手动安装以下依赖："
            echo "• gcc-aarch64-linux-gnu"
            echo "• build-essential/base-devel"
            echo "• autoconf, automake, libtool"
            echo "• make, pkg-config, gettext"
            echo "• git, wget, curl, cmake"
            echo "• python3, openssl-dev, zlib-dev"
            exit 1
        fi
    fi
}

# 检查NDK环境
check_ndk_environment() {
    if ! detect_ndk_environment; then
        print_error "未找到Android NDK"
        echo ""
        echo "请下载Android NDK并："
        echo "1. 解压到当前目录或设置 ANDROID_NDK_HOME 环境变量"
        echo "2. 确保NDK版本支持 API 21+"
        echo ""
        echo "下载地址: https://developer.android.com/ndk/downloads"
        exit 1
    fi
    
    print_success "NDK环境检查通过: $ANDROID_NDK_HOME"
}

# 设置编译环境
setup_environment() {
    print_status "设置编译环境..."
    log "设置编译环境: $COMPILE_MODE"
    
    # 首先设置NDK环境用于依赖编译 (强制所有模式都使用NDK编译依赖)
    setup_ndk_for_dependencies
    
    if [ "$COMPILE_MODE" = "termux_native" ]; then
        setup_termux_environment
    elif [ "$COMPILE_MODE" = "android_native" ]; then
        setup_android_environment
    elif [ "$COMPILE_MODE" = "static" ] || [ "$COMPILE_MODE" = "minimal" ]; then
        setup_cross_compile_environment
    elif [ "$COMPILE_MODE" = "ndk" ]; then
        setup_ndk_environment
    fi
    
    print_success "编译环境设置完成"
    echo "主编译器: ${CC:-未设置}"
    echo "NDK依赖编译器: $NDK_CC"
}

# 设置Termux编译环境
setup_termux_environment() {
    export CC=clang
    export CXX=clang++
    export AR=llvm-ar
    export STRIP=llvm-strip
    export RANLIB=llvm-ranlib
    export HOST="aarch64-linux-android"
    export PKG_CONFIG_PATH="$PREFIX/lib/pkgconfig"
    
    print_success "Termux编译环境设置完成"
    echo "编译器: $CC"
    echo "前缀目录: $PREFIX"
    echo "版本: $(clang --version | head -n1)"
}

# 设置Android原生环境
setup_android_environment() {
    if command -v clang >/dev/null 2>&1; then
        export CC=clang
        export CXX=clang++
    elif command -v gcc >/dev/null 2>&1; then
        export CC=gcc
        export CXX=g++
    else
        print_error "未找到可用的编译器"
        exit 1
    fi
    
    export HOST="aarch64-linux-android"
    
    print_success "Android原生编译环境设置完成"
    echo "编译器: $CC"
}

# 设置交叉编译环境
setup_cross_compile_environment() {
    export CC=aarch64-linux-gnu-gcc
    export CXX=aarch64-linux-gnu-g++
    export AR=aarch64-linux-gnu-ar
    export STRIP=aarch64-linux-gnu-strip
    export RANLIB=aarch64-linux-gnu-ranlib
    export HOST="aarch64-linux-gnu"
    export PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig
    
    # 检查编译器
    if ! command -v "$CC" >/dev/null 2>&1; then
        print_error "交叉编译器不存在: $CC"
        exit 1
    fi
    
    print_success "静态编译环境设置完成"
    echo "编译器: $CC"
    echo "目标平台: $HOST"
    echo "版本: $($CC --version | head -n1)"
}

# 设置NDK环境
setup_ndk_environment() {
    export TOOLCHAIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64"
    export TARGET=aarch64-linux-android
    export API=21
    export CC="$TOOLCHAIN/bin/$TARGET$API-clang"
    export CXX="$TOOLCHAIN/bin/$TARGET$API-clang++"
    export AR="$TOOLCHAIN/bin/llvm-ar"
    export STRIP="$TOOLCHAIN/bin/llvm-strip"
    export RANLIB="$TOOLCHAIN/bin/llvm-ranlib"
    export HOST="aarch64-linux-android"
    
    # 检查编译器
    if [ ! -f "$CC" ]; then
        print_error "NDK编译器不存在: $CC"
        print_error "请检查NDK安装和版本"
        exit 1
    fi
    
    print_success "NDK编译环境设置完成"
    echo "编译器: $CC"
    echo "目标平台: $HOST"
    echo "版本: $($CC --version 2>/dev/null | head -n1 || echo "无法获取版本")"
}

# 编译依赖库
compile_dependencies() {
    print_status "开始编译Git依赖库..."
    cd "$BUILD_DIR"
    
    # 创建依赖库安装目录
    local deps_dir="$BUILD_DIR/deps"
    mkdir -p "$deps_dir"/{lib,include,bin,share}
    
    # 设置依赖库路径
    export DEPS_PREFIX="$deps_dir"
    export PKG_CONFIG_PATH="$deps_dir/lib/pkgconfig:$PKG_CONFIG_PATH"
    export LD_LIBRARY_PATH="$deps_dir/lib:$LD_LIBRARY_PATH"
    export CPPFLAGS="-I$deps_dir/include $CPPFLAGS"
    export LDFLAGS="-L$deps_dir/lib $LDFLAGS"
    
    print_status "依赖库安装目录: $deps_dir"
    log "开始编译依赖库，目标目录: $deps_dir"
    
    # 保存当前环境变量
    local saved_cc="$CC"
    local saved_cxx="$CXX"
    local saved_ar="$AR"
    local saved_strip="$STRIP"
    local saved_ranlib="$RANLIB"
    
    # 显示进度
    local total_deps=4
    local current_dep=0
    
    # 编译各个依赖库（NDK环境已在setup_environment中设置）
    if [ "$COMPILE_MODE" != "minimal" ]; then
        ((current_dep++))
        print_status "[$current_dep/$total_deps] 编译zlib..."
        compile_zlib_ndk || {
            print_error "zlib编译失败"
            return 1
        }
        
        ((current_dep++))
        print_status "[$current_dep/$total_deps] 编译OpenSSL..."
        compile_openssl_ndk || {
            print_error "OpenSSL编译失败"
            return 1
        }
        
        ((current_dep++))
        print_status "[$current_dep/$total_deps] 编译curl..."
        compile_curl || {
            print_warning "curl编译失败，将在Git编译时跳过HTTP支持"
            export NO_CURL=1
        }
        
        ((current_dep++))
        print_status "[$current_dep/$total_deps] 编译expat..."
        compile_expat || {
            print_warning "expat编译失败，将在Git编译时跳过XML支持"
            export NO_EXPAT=1
        }
    else
        print_status "最小化编译模式，仍需编译核心依赖"
        ((current_dep++))
        print_status "[$current_dep/2] 编译zlib..."
        compile_zlib_ndk || {
            print_error "zlib编译失败"
            return 1
        }
        
        ((current_dep++))
        print_status "[$current_dep/2] 编译OpenSSL..."
        compile_openssl_ndk || {
            print_warning "OpenSSL编译失败，将禁用HTTPS支持"
            export NO_OPENSSL=1
        }
    fi
    
    # 恢复环境变量
    export CC="$saved_cc"
    export CXX="$saved_cxx"
    export AR="$saved_ar"
    export STRIP="$saved_strip"
    export RANLIB="$saved_ranlib"
    
    # 验证编译的库文件架构
    verify_dependencies_architecture
    
    # 创建依赖库信息文件
    create_dependencies_info
    
    print_success "依赖库编译完成"
    log "依赖库编译完成，位置: $deps_dir"
}

# 创建依赖库信息文件
create_dependencies_info() {
    local info_file="$DEPS_PREFIX/DEPENDENCIES_BUILD_INFO.txt"
    
    cat > "$info_file" << EOF
Git Android Dependencies Build Information
========================================

构建时间: $(date)
构建环境: $ENV_TYPE
编译模式: $COMPILE_MODE
目标架构: ARM64 (aarch64)

NDK信息:
- NDK路径: $ANDROID_NDK_HOME
- 工具链: $NDK_TOOLCHAIN  
- 目标: $NDK_TARGET$NDK_API
- 编译器: $NDK_CC

编译的依赖库:
EOF
    
    # 检查并记录库文件信息
    for lib_file in "$DEPS_PREFIX/lib"/*.a; do
        if [ -f "$lib_file" ]; then
            local lib_name=$(basename "$lib_file")
            local lib_size=$(du -h "$lib_file" 2>/dev/null | cut -f1 || echo "unknown")
            local lib_arch=$(file "$lib_file" 2>/dev/null | grep -o "aarch64\|ARM64\|arm64" || echo "unknown")
            echo "- $lib_name ($lib_size, $lib_arch 架构)" >> "$info_file"
        fi
    done
    
    # 检查头文件
    echo "" >> "$info_file"
    echo "头文件目录:" >> "$info_file"
    find "$DEPS_PREFIX/include" -type d 2>/dev/null | sed 's/^/- /' >> "$info_file" || true
    
    print_status "依赖库信息已保存到: $info_file"
}

# 设置NDK环境用于依赖编译
setup_ndk_for_dependencies() {
    print_status "设置NDK环境用于依赖库编译..."
    
    # 检测NDK环境
    if [ -z "$ANDROID_NDK_HOME" ]; then
        detect_ndk_environment || {
            print_error "编译Android依赖需要NDK环境"
            print_error "请设置ANDROID_NDK_HOME环境变量或将NDK放在当前目录"
            exit 1
        }
    fi
    
    # 设置NDK工具链
    export NDK_TOOLCHAIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt"
    
    # 检测宿主系统
    local host_os=""
    case "$(uname -s)" in
        Linux*) host_os="linux-x86_64" ;;
        Darwin*) host_os="darwin-x86_64" ;;
        CYGWIN*|MINGW*|MSYS*) host_os="windows-x86_64" ;;
        *) host_os="linux-x86_64" ;;
    esac
    
    export NDK_TOOLCHAIN="$NDK_TOOLCHAIN/$host_os"
    
    if [ ! -d "$NDK_TOOLCHAIN" ]; then
        print_error "NDK工具链目录不存在: $NDK_TOOLCHAIN"
        exit 1
    fi
    
    # 设置Android目标
    export NDK_TARGET="aarch64-linux-android"
    export NDK_API="21"
    export NDK_ARCH="arm64"
    
    # 设置NDK编译器
    export NDK_CC="$NDK_TOOLCHAIN/bin/${NDK_TARGET}${NDK_API}-clang"
    export NDK_CXX="$NDK_TOOLCHAIN/bin/${NDK_TARGET}${NDK_API}-clang++"
    export NDK_AR="$NDK_TOOLCHAIN/bin/llvm-ar"
    export NDK_STRIP="$NDK_TOOLCHAIN/bin/llvm-strip"
    export NDK_RANLIB="$NDK_TOOLCHAIN/bin/llvm-ranlib"
    export NDK_NM="$NDK_TOOLCHAIN/bin/llvm-nm"
    
    # 检查编译器是否存在
    if [ ! -f "$NDK_CC" ]; then
        print_error "NDK编译器不存在: $NDK_CC"
        print_error "请检查NDK版本和安装"
        exit 1
    fi
    
    print_success "NDK依赖编译环境设置完成"
    echo "工具链: $NDK_TOOLCHAIN"
    echo "目标: $NDK_TARGET$NDK_API"
    echo "编译器: $NDK_CC"
}

# 验证依赖库架构
verify_dependencies_architecture() {
    print_status "验证依赖库架构..."
    
    local libs_to_check=(
        "$DEPS_PREFIX/lib/libz.a"
        "$DEPS_PREFIX/lib/libssl.a"
        "$DEPS_PREFIX/lib/libcrypto.a"
    )
    
    for lib in "${libs_to_check[@]}"; do
        if [ -f "$lib" ]; then
            local arch_info=$(file "$lib" 2>/dev/null | grep -o "aarch64\|ARM64\|arm64" || echo "unknown")
            if [ "$arch_info" != "unknown" ]; then
                print_success "✓ $(basename "$lib"): $arch_info 架构"
            else
                print_warning "⚠ $(basename "$lib"): 架构检测失败，请手动验证"
            fi
        else
            print_warning "⚠ 未找到库文件: $(basename "$lib")"
        fi
    done
}

# 使用NDK编译zlib (ARM64)
compile_zlib_ndk() {
    print_status "使用NDK编译zlib (ARM64)..."
    cd "$BUILD_DIR"
    
    if [ ! -d "zlib" ]; then
        print_status "下载zlib源码..."
        # 优先使用git clone，更稳定
        git clone --depth=1 https://github.com/madler/zlib.git || \
        git clone --depth=1 https://gitee.com/mirrors/zlib.git zlib || \
        {
            print_status "Git克隆失败，尝试下载tar包..."
            wget https://zlib.net/zlib-1.3.tar.gz && tar -xzf zlib-1.3.tar.gz && mv zlib-1.3 zlib || \
            curl -L https://zlib.net/zlib-1.3.tar.gz -o zlib-1.3.tar.gz && tar -xzf zlib-1.3.tar.gz && mv zlib-1.3 zlib || {
                print_error "zlib源码下载失败，请检查网络连接"
                return 1
            }
        }
    fi
    
    cd zlib
    
    # 清理之前的编译
    make clean >/dev/null 2>&1 || true
    rm -f Makefile
    
    print_status "配置zlib使用NDK编译器..."
    
    # 设置NDK编译环境
    export CC="$NDK_CC"
    export CXX="$NDK_CXX"
    export AR="$NDK_AR"
    export RANLIB="$NDK_RANLIB"
    export STRIP="$NDK_STRIP"
    
    # 设置编译标志
    export CFLAGS="-fPIC -O2 -DANDROID -D__ANDROID_API__=$NDK_API"
    export CXXFLAGS="$CFLAGS"
    export LDFLAGS="-L$NDK_TOOLCHAIN/sysroot/usr/lib/$NDK_TARGET/$NDK_API"
    
    # 配置zlib
    ./configure --prefix="$DEPS_PREFIX" --static || {
        print_error "zlib NDK配置失败"
        return 1
    }
    
    # 编译和安装
    make -j$(nproc 2>/dev/null || echo "2") || {
        print_error "zlib NDK编译失败"
        return 1
    }
    
    make install || {
        print_error "zlib NDK安装失败"
        return 1
    }
    
    print_success "zlib NDK编译完成 (ARM64)"
    
    # 验证生成的库文件
    if [ -f "$DEPS_PREFIX/lib/libz.a" ]; then
        local arch_info=$(file "$DEPS_PREFIX/lib/libz.a" 2>/dev/null || echo "无法检测")
        print_status "libz.a 架构信息: $arch_info"
    else
        print_warning "libz.a 文件未找到"
    fi
    
    log "zlib NDK编译完成: $DEPS_PREFIX"
}

# 使用NDK编译OpenSSL (ARM64)
compile_openssl_ndk() {
    print_status "使用NDK编译OpenSSL (ARM64)..."
    cd "$BUILD_DIR"
    
    if [ ! -d "openssl" ]; then
        print_status "下载OpenSSL源码..."
        git clone --depth=1 --branch OpenSSL_1_1_1-stable https://github.com/openssl/openssl.git || \
        git clone --depth=1 --branch OpenSSL_1_1_1-stable https://gitee.com/mirrors/openssl.git openssl || \
        {
            print_status "Git克隆失败，尝试下载tar包..."
            wget https://www.openssl.org/source/openssl-1.1.1w.tar.gz && tar -xzf openssl-1.1.1w.tar.gz && mv openssl-1.1.1w openssl || \
            curl -L https://www.openssl.org/source/openssl-1.1.1w.tar.gz -o openssl-1.1.1w.tar.gz && tar -xzf openssl-1.1.1w.tar.gz && mv openssl-1.1.1w openssl || {
                print_error "OpenSSL源码下载失败，请检查网络连接"
                return 1
            }
        }
    fi
    
    cd openssl
    
    # 清理之前的编译
    make clean >/dev/null 2>&1 || true
    rm -f Makefile configdata.pm
    
    print_status "配置OpenSSL使用NDK编译器..."
    
    # 设置NDK编译环境
    export PATH="$NDK_TOOLCHAIN/bin:$PATH"
    export CC="$NDK_CC"
    export CXX="$NDK_CXX"
    export AR="$NDK_AR"
    export RANLIB="$NDK_RANLIB"
    export STRIP="$NDK_STRIP"
    export NM="$NDK_NM"
    
    # 设置Android特定的环境变量
    export ANDROID_NDK_ROOT="$ANDROID_NDK_HOME"
    export ANDROID_API="$NDK_API"
    
    # 设置编译标志
    local ssl_cflags="-fPIC -O2 -DANDROID -D__ANDROID_API__=$NDK_API"
    ssl_cflags="$ssl_cflags -I$NDK_TOOLCHAIN/sysroot/usr/include"
    ssl_cflags="$ssl_cflags -I$NDK_TOOLCHAIN/sysroot/usr/include/$NDK_TARGET"
    
    export CPPFLAGS="$ssl_cflags"
    export CFLAGS="$ssl_cflags"
    export CXXFLAGS="$ssl_cflags"
    export LDFLAGS="-L$NDK_TOOLCHAIN/sysroot/usr/lib/$NDK_TARGET/$NDK_API"
    
    # 配置OpenSSL选项
    local ssl_options="--prefix=$DEPS_PREFIX --openssldir=$DEPS_PREFIX/ssl"
    ssl_options="$ssl_options no-shared no-tests no-ui-console"
    ssl_options="$ssl_options -D__ANDROID_API__=$NDK_API"
    
    # 配置OpenSSL for Android ARM64
    print_status "运行OpenSSL Configure..."
    ./Configure android-arm64 $ssl_options || {
        print_error "OpenSSL NDK配置失败"
        print_warning "尝试查看config.log获取详细错误信息"
        [ -f "config.log" ] && tail -20 config.log
        return 1
    }
    
    # 修复Makefile中的编译器设置
    if [ -f "Makefile" ]; then
        print_status "修复Makefile编译器设置..."
        sed -i "s|^CC=.*|CC=$NDK_CC|" Makefile
        sed -i "s|^AR=.*|AR=$NDK_AR|" Makefile
        sed -i "s|^RANLIB=.*|RANLIB=$NDK_RANLIB|" Makefile
    fi
    
    # 编译OpenSSL
    print_status "编译OpenSSL..."
    make -j$(nproc 2>/dev/null || echo "2") build_libs || {
        print_error "OpenSSL NDK编译失败"
        return 1
    }
    
    # 安装OpenSSL库和头文件
    print_status "安装OpenSSL..."
    make install_dev || {
        print_error "OpenSSL NDK安装失败"
        return 1
    }
    
    print_success "OpenSSL NDK编译完成 (ARM64)"
    
    # 验证生成的库文件
    local libs_to_check=("libssl.a" "libcrypto.a")
    for lib in "${libs_to_check[@]}"; do
        if [ -f "$DEPS_PREFIX/lib/$lib" ]; then
            local arch_info=$(file "$DEPS_PREFIX/lib/$lib" 2>/dev/null || echo "无法检测")
            print_status "$lib 架构信息: $arch_info"
        else
            print_warning "$lib 文件未找到"
        fi
    done
    
    log "OpenSSL NDK编译完成: $DEPS_PREFIX"
}

# 编译curl
compile_curl() {
    print_status "编译curl..."
    cd "$BUILD_DIR"
    
    if [ ! -d "curl" ]; then
        print_status "下载curl源码..."
        git clone --depth=1 https://github.com/curl/curl.git || \
        git clone --depth=1 https://gitee.com/mirrors/curl.git curl || \
        {
            print_status "Git克隆失败，尝试下载tar包..."
            wget https://curl.se/download/curl-8.4.0.tar.gz && tar -xzf curl-8.4.0.tar.gz && mv curl-8.4.0 curl || \
            curl -L https://curl.se/download/curl-8.4.0.tar.gz -o curl-8.4.0.tar.gz && tar -xzf curl-8.4.0.tar.gz && mv curl-8.4.0 curl || {
                print_error "curl源码下载失败，请检查网络连接"
                return 1
            }
        }
    fi
    
    cd curl
    
    # 清理之前的编译
    make clean >/dev/null 2>&1 || true
    rm -f CMakeCache.txt
    
    # 检查是否有cmake
    if command -v cmake >/dev/null 2>&1; then
        compile_curl_cmake
    else
        compile_curl_autotools
    fi
}

# 使用cmake编译curl
compile_curl_cmake() {
    print_status "使用cmake编译curl..."
    
    local cmake_args="-DCMAKE_INSTALL_PREFIX=$DEPS_PREFIX"
    cmake_args="$cmake_args -DCMAKE_PREFIX_PATH=$DEPS_PREFIX"
    cmake_args="$cmake_args -DOPENSSL_ROOT_DIR=$DEPS_PREFIX"
    cmake_args="$cmake_args -DZLIB_ROOT=$DEPS_PREFIX"
    
    if [ "$COMPILE_MODE" = "static" ] || [ "$COMPILE_MODE" = "minimal" ]; then
        cmake_args="$cmake_args -DBUILD_SHARED_LIBS=OFF -DCURL_STATICLIB=ON"
    else
        cmake_args="$cmake_args -DBUILD_SHARED_LIBS=ON"
    fi
    
    if [ "$COMPILE_MODE" != "termux_native" ] && [ "$COMPILE_MODE" != "android_native" ]; then
        cmake_args="$cmake_args -DCMAKE_C_COMPILER=$CC"
        if [ -n "$CXX" ]; then
            cmake_args="$cmake_args -DCMAKE_CXX_COMPILER=$CXX"
        fi
    fi
    
    # 配置和编译
    cmake $cmake_args . && \
    make -j$(nproc 2>/dev/null || echo "2") && \
    make install || {
        print_error "curl cmake编译失败"
        return 1
    }
    
    print_success "curl编译完成"
}

# 使用autotools编译curl
compile_curl_autotools() {
    print_status "使用autotools编译curl..."
    
    # 生成configure脚本
    if [ ! -f "configure" ]; then
        autoreconf -fi || {
            print_error "curl autoreconf失败"
            return 1
        }
    fi
    
    local configure_args="--prefix=$DEPS_PREFIX"
    configure_args="$configure_args --with-ssl=$DEPS_PREFIX"
    configure_args="$configure_args --with-zlib=$DEPS_PREFIX"
    configure_args="$configure_args --disable-ldap --disable-ldaps"
    
    if [ "$COMPILE_MODE" = "static" ] || [ "$COMPILE_MODE" = "minimal" ]; then
        configure_args="$configure_args --enable-static --disable-shared"
    else
        configure_args="$configure_args --enable-shared"
    fi
    
    if [ "$COMPILE_MODE" != "termux_native" ] && [ "$COMPILE_MODE" != "android_native" ]; then
        configure_args="$configure_args --host=$HOST"
    fi
    
    # 配置和编译
    ./configure $configure_args && \
    make -j$(nproc 2>/dev/null || echo "2") && \
    make install || {
        print_error "curl autotools编译失败"
        return 1
    }
    
    print_success "curl编译完成"
}

# 编译expat
compile_expat() {
    print_status "编译expat..."
    cd "$BUILD_DIR"
    
    if [ ! -d "expat" ]; then
        print_status "下载expat源码..."
        git clone --depth=1 https://github.com/libexpat/libexpat.git expat || \
        git clone --depth=1 https://gitee.com/mirrors/libexpat.git expat || \
        {
            print_status "Git克隆失败，尝试下载tar包..."
            wget https://github.com/libexpat/libexpat/releases/download/R_2_5_0/expat-2.5.0.tar.gz && tar -xzf expat-2.5.0.tar.gz && mv expat-2.5.0 expat || \
            curl -L https://github.com/libexpat/libexpat/releases/download/R_2_5_0/expat-2.5.0.tar.gz -o expat-2.5.0.tar.gz && tar -xzf expat-2.5.0.tar.gz && mv expat-2.5.0 expat || {
                print_warning "expat源码下载失败，将跳过expat编译"
                export NO_EXPAT=1
                return 0
            }
        }
    fi
    
    cd expat
    
    # 如果是从git克隆的，进入expat子目录
    if [ -d "expat" ]; then
        cd expat
    fi
    
    # 清理之前的编译
    make clean >/dev/null 2>&1 || true
    rm -f Makefile config.status
    
    # 生成configure脚本
    if [ ! -f "configure" ]; then
        if [ -f "buildconf.sh" ]; then
            ./buildconf.sh || autoreconf -fi || {
                print_warning "expat configure脚本生成失败，跳过expat编译"
                export NO_EXPAT=1
                return 0
            }
        else
            autoreconf -fi || {
                print_warning "expat autoreconf失败，跳过expat编译"
                export NO_EXPAT=1
                return 0
            }
        fi
    fi
    
    # 设置NDK编译环境
    export CC="$NDK_CC"
    export CXX="$NDK_CXX"
    export AR="$NDK_AR"
    export STRIP="$NDK_STRIP"
    export RANLIB="$NDK_RANLIB"
    
    # 设置编译标志
    export CFLAGS="-fPIC -O2 -DANDROID -D__ANDROID_API__=$NDK_API"
    export CXXFLAGS="$CFLAGS"
    export LDFLAGS="-L$NDK_TOOLCHAIN/sysroot/usr/lib/$NDK_TARGET/$NDK_API"
    
    local configure_args="--prefix=$DEPS_PREFIX"
    configure_args="$configure_args --host=$NDK_TARGET"
    
    if [ "$COMPILE_MODE" = "static" ] || [ "$COMPILE_MODE" = "minimal" ]; then
        configure_args="$configure_args --enable-static --disable-shared"
    else
        configure_args="$configure_args --enable-shared --disable-static"
    fi
    
    # 配置和编译
    print_status "配置expat..."
    ./configure $configure_args || {
        print_warning "expat配置失败，跳过expat编译"
        export NO_EXPAT=1
        return 0
    }
    
    print_status "编译expat..."
    make -j$(nproc 2>/dev/null || echo "2") || {
        print_warning "expat编译失败，跳过expat编译"
        export NO_EXPAT=1
        return 0
    }
    
    print_status "安装expat..."
    make install || {
        print_warning "expat安装失败，跳过expat编译"
        export NO_EXPAT=1
        return 0
    }
    
    print_success "expat编译完成"
    
    # 验证生成的库文件
    if [ -f "$DEPS_PREFIX/lib/libexpat.a" ]; then
        local arch_info=$(file "$DEPS_PREFIX/lib/libexpat.a" 2>/dev/null || echo "无法检测")
        print_status "libexpat.a 架构信息: $arch_info"
    else
        print_warning "libexpat.a 文件未找到"
    fi
    
    log "expat编译完成: $DEPS_PREFIX"
}
        configure_args="$configure_args --enable-shared"
    fi
    
    if [ "$COMPILE_MODE" != "termux_native" ] && [ "$COMPILE_MODE" != "android_native" ]; then
        configure_args="$configure_args --host=$HOST"
    fi
    
    # 配置和编译
    ./configure $configure_args && \
    make -j$(nproc 2>/dev/null || echo "2") && \
    make install || {
        print_warning "expat编译失败，将在Git编译时使用NO_EXPAT=1"
        export NO_EXPAT=1
        return 0
    }
    
    print_success "expat编译完成"
    log "expat编译完成: $DEPS_PREFIX"
}

# 获取Git源码
get_git_source() {
    print_status "获取Git源码..."
    cd "$BUILD_DIR"
    
    # Git源码镜像源列表
    local git_mirrors=(
        "https://gitee.com/mirrors/git.git"
        "https://github.com/git/git.git"
        "https://git.kernel.org/pub/scm/git/git.git"
        "https://gitlab.com/git-vcs/git.git"
    )
    
    if [ ! -d "git" ]; then
        print_status "克隆Git源码仓库..."
        local cloned=false
        
        for mirror in "${git_mirrors[@]}"; do
            print_status "尝试从 $mirror 克隆..."
            if git clone --depth=1 --recursive "$mirror" git; then
                cloned=true
                break
            else
                print_warning "从 $mirror 克隆失败，尝试下一个镜像..."
            fi
        done
        
        if [ "$cloned" = false ]; then
            print_error "所有镜像源克隆都失败"
            return 1
        fi
    else
        print_status "更新Git源码..."
        cd git
        
        # 检查远程仓库连接
        if git ls-remote --exit-code origin > /dev/null 2>&1; then
            git fetch --depth=1 origin
            git reset --hard origin/master
        else
            print_warning "无法连接到远程仓库，使用现有源码"
        fi
        cd ..
    fi
    
    cd git
    
    # 获取版本信息
    GIT_VERSION=$(git describe --tags --abbrev=0 2>/dev/null || echo "unknown")
    local commit_hash=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
    local branch=$(git branch --show-current 2>/dev/null || echo "unknown")
    
    # 验证源码完整性
    if [ ! -f "Makefile" ] || [ ! -f "git.c" ]; then
        print_error "Git源码不完整，缺少必要文件"
        return 1
    fi
    
    # 创建源码信息文件
    cat > "$BUILD_DIR/git-source-info.txt" << EOF
Git源码信息:
版本: $GIT_VERSION
提交: $commit_hash
分支: $branch
获取时间: $(date)
源码目录: $BUILD_DIR/git
EOF

    print_success "Git源码准备完成"
    echo "  版本: $GIT_VERSION"
    echo "  提交: $commit_hash"
    echo "  分支: $branch"
    
    log "Git源码版本: $GIT_VERSION, 提交: $commit_hash"
    return 0
}

# 配置构建系统
configure_build() {
    print_status "配置Git构建系统..."
    cd "$BUILD_DIR/git"
    
    # 生成configure脚本
    if [ ! -f "configure" ]; then
        print_status "生成configure脚本..."
        make configure || {
            print_error "configure脚本生成失败"
            print_warning "尝试手动运行autoconf..."
            autoconf configure.ac > configure 2>/dev/null || {
                print_error "无法生成configure脚本"
                exit 1
            }
            chmod +x configure
        }
    fi
    
    # 设置configure参数
    local configure_args=""
    local prefix="$BUILD_DIR/git-android"
    
    if [ "$COMPILE_MODE" = "termux_native" ]; then
        configure_args="--prefix=$PREFIX/usr"
    elif [ "$COMPILE_MODE" = "android_native" ]; then
        configure_args="--prefix=/data/local/tmp/git"
    else
        configure_args="--prefix=$prefix --host=$HOST"
    fi
    
    # 添加依赖库路径配置
    if [ -d "$DEPS_PREFIX" ]; then
        # 清理之前的环境变量设置
        export CPPFLAGS="-I$DEPS_PREFIX/include"
        export LDFLAGS="-L$DEPS_PREFIX/lib"
        export PKG_CONFIG_PATH="$DEPS_PREFIX/lib/pkgconfig"
        
        # 检查并设置库路径
        local ssl_path=""
        local curl_path=""
        local zlib_path=""
        local expat_path=""
        
        # 检查OpenSSL
        if [ -f "$DEPS_PREFIX/lib/libssl.a" ] && [ -f "$DEPS_PREFIX/lib/libcrypto.a" ] && [ -z "$NO_OPENSSL" ]; then
            ssl_path="--with-openssl=$DEPS_PREFIX"
            print_status "✓ 使用编译的OpenSSL"
        else
            ssl_path="--without-openssl"
            print_warning "⚠ 禁用OpenSSL支持"
        fi
        
        # 检查curl
        if [ -f "$DEPS_PREFIX/lib/libcurl.a" ] && [ -z "$NO_CURL" ]; then
            curl_path="--with-curl=$DEPS_PREFIX"
            print_status "✓ 使用编译的curl"
        else
            curl_path="--without-curl"
            print_warning "⚠ 禁用curl支持（HTTP/HTTPS操作受限）"
        fi
        
        # 检查zlib
        if [ -f "$DEPS_PREFIX/lib/libz.a" ]; then
            zlib_path="--with-zlib=$DEPS_PREFIX"
            print_status "✓ 使用编译的zlib"
        else
            zlib_path="--without-zlib"
            print_warning "⚠ 禁用zlib支持"
        fi
        
        # 检查expat
        if [ -f "$DEPS_PREFIX/lib/libexpat.a" ] && [ -z "$NO_EXPAT" ]; then
            expat_path="--with-expat=$DEPS_PREFIX"
            print_status "✓ 使用编译的expat"
        else
            expat_path="--without-expat"
            print_warning "⚠ 禁用expat支持（某些XML功能受限）"
        fi
        
        configure_args="$configure_args $ssl_path $curl_path $zlib_path $expat_path"
    else
        print_warning "未找到依赖库目录，使用系统库"
        configure_args="$configure_args --with-openssl --with-curl --with-zlib --with-expat"
    fi
    
    # 通用配置参数
    configure_args="$configure_args \
        --without-tcltk \
        --without-python \
        --without-perl \
        --disable-nls"
    
    # 根据编译模式添加特定参数
    if [ "$COMPILE_MODE" = "static" ] || [ "$COMPILE_MODE" = "minimal" ]; then
        configure_args="$configure_args --enable-static --disable-shared"
        export LDFLAGS="$LDFLAGS -static -pthread"
    fi
    
    if [ "$COMPILE_MODE" = "minimal" ]; then
        configure_args="$configure_args \
            --without-openssl \
            --without-curl \
            --without-iconv \
            --without-libpcre2 \
            --without-libpcre"
    fi
    
    # 设置交叉编译环境变量
    if [ "$COMPILE_MODE" != "termux_native" ] && [ "$COMPILE_MODE" != "android_native" ]; then
        export CC="$CC"
        export CXX="$CXX"
        export AR="$AR"
        export RANLIB="$RANLIB"
        export STRIP="$STRIP"
    fi
    
    print_status "运行configure..."
    echo "Configure参数: $configure_args"
    log "Configure参数: $configure_args"
    
    # 尝试configure，如果失败则提供详细错误信息
    if ! ./configure $configure_args 2>&1 | tee -a "$LOG_FILE"; then
        print_error "configure失败"
        print_warning "查看config.log获取详细错误信息:"
        if [ -f "config.log" ]; then
            echo "最后20行config.log内容:"
            tail -20 config.log
        fi
        print_warning "完整日志文件: $LOG_FILE"
        exit 1
    fi
    
    print_success "configure完成"
    
    # 显示配置摘要
    if [ -f "config.mak.autogen" ]; then
        print_status "配置摘要:"
        echo "安装前缀: $(grep '^prefix' config.mak.autogen 2>/dev/null | cut -d'=' -f2 || echo "未设置")"
        echo "使用的库:"
        grep -E '^(NO_OPENSSL|NO_CURL|NO_EXPAT|NO_ZLIB)' config.mak.autogen 2>/dev/null || echo "  所有库均启用"
    fi
}

# 编译Git
compile_git() {
    print_status "开始编译Git..."
    cd "$BUILD_DIR/git"
    
    # 设置交叉编译环境变量
    if [ "$COMPILE_MODE" != "termux_native" ] && [ "$COMPILE_MODE" != "android_native" ]; then
        # 修改Makefile中的编译器设置
        if [ -f "config.mak.autogen" ]; then
            echo "CC = $CC" >> config.mak.autogen
            echo "AR = $AR" >> config.mak.autogen
            echo "RANLIB = $RANLIB" >> config.mak.autogen
            if [ -n "$STRIP" ]; then
                echo "STRIP = $STRIP" >> config.mak.autogen
            fi
        fi
    fi
    
    # 添加NO_EXPAT选项（如果expat编译失败）
    local make_args=""
    if [ -n "$NO_EXPAT" ]; then
        make_args="NO_EXPAT=1"
        print_status "使用 NO_EXPAT=1 编译Git（跳过expat支持）"
    fi
    
    # 获取CPU核心数
    local cores=$(nproc 2>/dev/null || echo "2")
    print_status "使用 $cores 个并行编译任务"
    
    # 编译
    make -j$cores $make_args all 2>&1 | tee -a "$LOG_FILE" || {
        print_error "Git编译失败"
        print_warning "尝试单线程编译..."
        make clean
        make $make_args all 2>&1 | tee -a "$LOG_FILE" || {
            print_error "单线程编译也失败"
            print_warning "查看日志文件: $LOG_FILE"
            exit 1
        }
    }
    
    print_success "Git编译完成"
}

# 安装Git
install_git() {
    print_status "安装Git..."
    cd "$BUILD_DIR/git"
    
    if [ "$COMPILE_MODE" = "termux_native" ] || [ "$COMPILE_MODE" = "android_native" ]; then
        make install 2>&1 | tee -a "$LOG_FILE" || {
            print_error "Git安装失败"
            exit 1
        }
        print_success "Git已安装到系统"
    else
        make install 2>&1 | tee -a "$LOG_FILE" || {
            print_error "Git安装失败"
            exit 1
        }
        print_success "Git已安装到: $(grep '^prefix' config.mak.autogen 2>/dev/null | cut -d'=' -f2 || echo "$BUILD_DIR/git-android")"
    fi
}

# 创建安装包
create_package() {
    print_status "创建Git安装包..."
    
    local install_dir=""
    if [ "$COMPILE_MODE" = "termux_native" ]; then
        install_dir="$PREFIX/usr"
    elif [ "$COMPILE_MODE" = "android_native" ]; then
        install_dir="/data/local/tmp/git"
    else
        install_dir="$BUILD_DIR/git-android"
    fi
    
    if [ ! -d "$install_dir" ]; then
        print_error "安装目录不存在: $install_dir"
        return 1
    fi
    
    # 复制编译好的ARM64依赖库到安装目录
    copy_dependencies_to_package "$install_dir"
    
    cd "$BUILD_DIR"
    local package_name="git-android-${GIT_VERSION}-${COMPILE_MODE}-$(date +%Y%m%d).tar.gz"
    
    if [ "$COMPILE_MODE" != "termux_native" ] && [ "$COMPILE_MODE" != "android_native" ]; then
        tar -czf "$package_name" -C "$install_dir" . || {
            print_error "创建安装包失败"
            return 1
        }
        print_success "安装包已创建: $BUILD_DIR/$package_name"
    fi
    
    # 创建安装脚本
    create_install_script "$install_dir"
}

# 复制ARM64依赖库到安装包
copy_dependencies_to_package() {
    local target_dir="$1"
    
    print_status "复制ARM64依赖库到安装包..."
    
    # 创建目标目录
    mkdir -p "$target_dir/lib"
    mkdir -p "$target_dir/include"
    
    # 复制zlib库和头文件
    if [ -f "$DEPS_PREFIX/lib/libz.a" ]; then
        cp "$DEPS_PREFIX/lib/libz.a" "$target_dir/lib/"
        print_success "✓ 复制 libz.a (ARM64)"
        
        # 验证复制的库文件架构
        local arch_info=$(file "$target_dir/lib/libz.a" 2>/dev/null | grep -o "aarch64\|ARM64\|arm64" || echo "unknown")
        if [ "$arch_info" != "unknown" ]; then
            print_status "  架构验证: $arch_info"
        fi
    else
        print_warning "⚠ libz.a 未找到，跳过复制"
    fi
    
    # 复制OpenSSL库和头文件
    local openssl_libs=("libssl.a" "libcrypto.a")
    for lib in "${openssl_libs[@]}"; do
        if [ -f "$DEPS_PREFIX/lib/$lib" ]; then
            cp "$DEPS_PREFIX/lib/$lib" "$target_dir/lib/"
            print_success "✓ 复制 $lib (ARM64)"
            
            # 验证复制的库文件架构
            local arch_info=$(file "$target_dir/lib/$lib" 2>/dev/null | grep -o "aarch64\|ARM64\|arm64" || echo "unknown")
            if [ "$arch_info" != "unknown" ]; then
                print_status "  架构验证: $arch_info"
            fi
        else
            print_warning "⚠ $lib 未找到，跳过复制"
        fi
    done
    
    # 复制头文件
    if [ -d "$DEPS_PREFIX/include/openssl" ]; then
        cp -r "$DEPS_PREFIX/include/openssl" "$target_dir/include/"
        print_success "✓ 复制 OpenSSL 头文件"
    fi
    
    if [ -f "$DEPS_PREFIX/include/zlib.h" ]; then
        cp "$DEPS_PREFIX/include/zlib.h" "$target_dir/include/"
        print_success "✓ 复制 zlib 头文件"
    fi
    
    if [ -f "$DEPS_PREFIX/include/zconf.h" ]; then
        cp "$DEPS_PREFIX/include/zconf.h" "$target_dir/include/"
        print_success "✓ 复制 zconf 头文件"
    fi
    
    # 创建依赖信息文件
    cat > "$target_dir/DEPENDENCIES_INFO.txt" << EOF
Git Android Dependencies Information
==================================

编译时间: $(date)
编译模式: $COMPILE_MODE
目标架构: ARM64 (aarch64)

NDK编译工具链:
- NDK路径: $ANDROID_NDK_HOME
- 工具链: $NDK_TOOLCHAIN
- 目标: $NDK_TARGET$NDK_API
- 编译器: $NDK_CC

包含的ARM64依赖库:
EOF
    
    # 添加库文件信息到依赖信息
    for lib_file in "$target_dir/lib"/*.a; do
        if [ -f "$lib_file" ]; then
            local lib_name=$(basename "$lib_file")
            local lib_size=$(du -h "$lib_file" | cut -f1)
            local lib_arch=$(file "$lib_file" 2>/dev/null | grep -o "aarch64\|ARM64\|arm64" || echo "unknown")
            echo "- $lib_name ($lib_size, $lib_arch)" >> "$target_dir/DEPENDENCIES_INFO.txt"
        fi
    done
    
    print_success "ARM64依赖库复制完成"
    echo "依赖库目录: $target_dir/lib"
    echo "头文件目录: $target_dir/include"
    echo "依赖信息: $target_dir/DEPENDENCIES_INFO.txt"
}

# 创建安装脚本
create_install_script() {
    local install_dir="$1"
    local script_name="install_git_android.sh"
    
    print_status "创建安装脚本..."
    
    cat > "$BUILD_DIR/$script_name" << 'EOF'
#!/system/bin/sh
# Git Android 安装脚本 v2.0
# 支持多种安装模式和环境检测

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# 检测环境
detect_environment() {
    if [ -n "$TERMUX_VERSION" ]; then
        ENV_TYPE="termux"
        INSTALL_PREFIX="$PREFIX/usr"
    elif [ -n "$ANDROID_ROOT" ] && [ -d "/system" ]; then
        ENV_TYPE="android_native"
        INSTALL_PREFIX="/data/local/tmp/git"
        BIN_LINK_DIR="/system/bin"
    else
        ENV_TYPE="unknown"
        print_error "未知的Android环境"
        exit 1
    fi
}

# 检查权限
check_permissions() {
    if [ "$ENV_TYPE" = "android_native" ]; then
        if [ "$(id -u)" != "0" ]; then
            print_error "Android原生环境需要root权限"
            print_status "请使用 'su' 命令获取root权限后重试"
            exit 1
        fi
        
        # 检查系统分区是否可写
        if ! touch /system/.test_write 2>/dev/null; then
            print_warning "系统分区只读，尝试重新挂载..."
            mount -o remount,rw /system || {
                print_error "无法重新挂载系统分区为可写"
                print_status "某些功能可能受限"
            }
            rm -f /system/.test_write 2>/dev/null || true
        fi
    fi
}

# 安装Git
install_git() {
    print_status "开始安装Git到Android系统..."
    
    # 创建安装目录
    mkdir -p "$INSTALL_PREFIX/bin"
    mkdir -p "$INSTALL_PREFIX/libexec/git-core"
    mkdir -p "$INSTALL_PREFIX/share/git-core"
    mkdir -p "$INSTALL_PREFIX/etc"
    
    # 检查源文件
    if [ ! -f "./bin/git" ]; then
        print_error "找不到git二进制文件: ./bin/git"
        print_status "请确保在解压的Git包目录中运行此脚本"
        exit 1
    fi
    
    # 复制主要文件
    print_status "复制Git二进制文件..."
    cp -r ./bin/* "$INSTALL_PREFIX/bin/" || {
        print_error "复制bin文件失败"
        exit 1
    }
    
    if [ -d "./libexec" ]; then
        print_status "复制Git核心库..."
        cp -r ./libexec/* "$INSTALL_PREFIX/libexec/" || {
            print_error "复制libexec文件失败"
            exit 1
        }
    fi
    
    if [ -d "./share" ]; then
        print_status "复制共享文件..."
        cp -r ./share/* "$INSTALL_PREFIX/share/" || {
            print_warning "复制share文件失败，跳过"
        }
    fi
    
    # 复制依赖库
    if [ -d "./lib" ]; then
        print_status "复制依赖库文件..."
        mkdir -p "$INSTALL_PREFIX/lib"
        cp -r ./lib/* "$INSTALL_PREFIX/lib/" || {
            print_warning "复制lib文件失败，跳过"
        }
    fi
    
    # 设置权限
    print_status "设置文件权限..."
    find "$INSTALL_PREFIX/bin" -type f -exec chmod 755 {} \; 2>/dev/null || true
    find "$INSTALL_PREFIX/libexec" -type f -exec chmod 755 {} \; 2>/dev/null || true
    
    # 创建系统链接(仅在Android原生环境)
    if [ "$ENV_TYPE" = "android_native" ] && [ -n "$BIN_LINK_DIR" ]; then
        print_status "创建系统链接..."
        ln -sf "$INSTALL_PREFIX/bin/git" "$BIN_LINK_DIR/git" || {
            print_warning "创建系统链接失败，需要手动添加到PATH"
        }
    fi
    
    # 设置环境变量文件
    create_env_setup
    
    print_success "Git安装完成!"
    
    # 测试安装
    test_installation
}

# 创建环境设置文件
create_env_setup() {
    local env_file=""
    
    if [ "$ENV_TYPE" = "termux" ]; then
        env_file="$PREFIX/etc/profile.d/git-android.sh"
        mkdir -p "$(dirname "$env_file")"
    else
        env_file="$INSTALL_PREFIX/git-env.sh"
    fi
    
    cat > "$env_file" << ENVEOF
#!/bin/bash
# Git Android 环境设置

export GIT_EXEC_PATH="$INSTALL_PREFIX/libexec/git-core"
export GIT_TEMPLATE_DIR="$INSTALL_PREFIX/share/git-core/templates"

# 添加到PATH
if [[ ":\$PATH:" != *":$INSTALL_PREFIX/bin:"* ]]; then
    export PATH="$INSTALL_PREFIX/bin:\$PATH"
fi

# 设置Git配置
export GIT_CONFIG_GLOBAL="$INSTALL_PREFIX/etc/gitconfig"
ENVEOF
    
    chmod 755 "$env_file"
    print_status "环境设置文件已创建: $env_file"
    
    if [ "$ENV_TYPE" = "android_native" ]; then
        print_status "要使用Git，请运行: source $env_file"
    fi
}

# 测试安装
test_installation() {
    print_status "测试Git安装..."
    
    local git_cmd="$INSTALL_PREFIX/bin/git"
    
    if [ ! -f "$git_cmd" ]; then
        print_error "Git二进制文件不存在: $git_cmd"
        return 1
    fi
    
    # 测试版本
    local version_output
    if version_output=$("$git_cmd" --version 2>&1); then
        print_success "Git版本: $version_output"
    else
        print_error "Git版本测试失败: $version_output"
        return 1
    fi
    
    # 测试帮助
    if "$git_cmd" --help >/dev/null 2>&1; then
        print_success "Git帮助命令测试通过"
    else
        print_warning "Git帮助命令测试失败"
    fi
    
    print_success "Git安装测试完成"
    echo ""
    echo "Git安装位置: $INSTALL_PREFIX"
    echo "版本: $version_output"
    
    if [ "$ENV_TYPE" = "android_native" ]; then
        echo ""
        echo "使用方法:"
        echo "1. 直接运行: $git_cmd [命令]"
        echo "2. 或者设置环境: source $INSTALL_PREFIX/git-env.sh && git [命令]"
    fi
}

# 卸载Git
uninstall_git() {
    print_status "卸载Git..."
    
    # 删除安装目录
    if [ -d "$INSTALL_PREFIX" ]; then
        rm -rf "$INSTALL_PREFIX"
        print_success "已删除安装目录: $INSTALL_PREFIX"
    fi
    
    # 删除系统链接
    if [ "$ENV_TYPE" = "android_native" ] && [ -L "$BIN_LINK_DIR/git" ]; then
        rm -f "$BIN_LINK_DIR/git"
        print_success "已删除系统链接"
    fi
    
    print_success "Git卸载完成"
}

# 显示使用帮助
show_help() {
    echo "Git Android 安装脚本"
    echo ""
    echo "使用方法:"
    echo "  $0 [选项]"
    echo ""
    echo "选项:"
    echo "  install   - 安装Git (默认)"
    echo "  uninstall - 卸载Git"
    echo "  test      - 测试已安装的Git"
    echo "  help      - 显示此帮助"
    echo ""
}

# 主函数
main() {
    case "${1:-install}" in
        "install"|"")
            detect_environment
            check_permissions
            install_git
            ;;
        "uninstall")
            detect_environment
            check_permissions
            uninstall_git
            ;;
        "test")
            detect_environment
            test_installation
            ;;
        "help"|"-h"|"--help")
            show_help
            ;;
        *)
            print_error "未知选项: $1"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
EOF
    
    chmod +x "$BUILD_DIR/$script_name"
    print_success "增强的安装脚本已创建: $BUILD_DIR/$script_name"
    
    # 创建简化的快速安装脚本
    cat > "$BUILD_DIR/quick_install.sh" << 'QUICKEOF'
#!/system/bin/sh
# Git Android 快速安装脚本

if [ "$(id -u)" != "0" ]; then
    echo "需要root权限，请使用 su 命令"
    exit 1
fi

INSTALL_DIR="/data/local/tmp/git"
mkdir -p "$INSTALL_DIR/bin"

if [ -f "./bin/git" ]; then
    cp -r ./bin/* "$INSTALL_DIR/bin/"
    chmod 755 "$INSTALL_DIR/bin"/*
    ln -sf "$INSTALL_DIR/bin/git" "/system/bin/git" 2>/dev/null || true
    echo "Git快速安装完成: $("$INSTALL_DIR/bin/git" --version 2>/dev/null || echo 'git')"
    echo "使用: git --version"
else
    echo "错误: 找不到git二进制文件"
    exit 1
fi
QUICKEOF
    
    chmod +x "$BUILD_DIR/quick_install.sh"
    print_success "快速安装脚本已创建: $BUILD_DIR/quick_install.sh"
}

# 进度显示函数
show_progress() {
    local current=$1
    local total=$2
    local task=$3
    local width=50
    
    local percentage=$((current * 100 / total))
    local completed=$((current * width / total))
    local remaining=$((width - completed))
    
    printf "\r${BLUE}[%3d%%]${NC} " $percentage
    printf "${GREEN}"
    printf "%*s" $completed | tr ' ' '█'
    printf "${NC}"
    printf "%*s" $remaining | tr ' ' '░'
    printf " ${BOLD}%s${NC}" "$task"
    
    if [ $current -eq $total ]; then
        echo ""
    fi
}

# 测试编译结果
test_git() {
    print_status "测试编译的Git..."
    
    local git_binary=""
    if [ "$COMPILE_MODE" = "termux_native" ]; then
        git_binary="$PREFIX/usr/bin/git"
    elif [ "$COMPILE_MODE" = "android_native" ]; then
        git_binary="/data/local/tmp/git/bin/git"
    else
        git_binary="$BUILD_DIR/git-android/bin/git"
    fi
    
    if [ ! -f "$git_binary" ]; then
        print_error "Git二进制文件不存在: $git_binary"
        return 1
    fi
    
    echo ""
    print_status "执行Git功能测试..."
    
    local test_count=0
    local passed_count=0
    local failed_tests=()
    
    # 测试1: 版本信息
    ((test_count++))
    print_status "[$test_count/7] 测试Git版本..."
    if version_output=$("$git_binary" --version 2>&1); then
        print_success "✓ 版本: $version_output"
        ((passed_count++))
    else
        print_error "✗ Git版本测试失败: $version_output"
        failed_tests+=("版本检测")
    fi
    
    # 测试2: 帮助命令
    ((test_count++))
    print_status "[$test_count/7] 测试Git帮助..."
    if "$git_binary" --help >/dev/null 2>&1; then
        print_success "✓ 帮助命令正常"
        ((passed_count++))
    else
        print_error "✗ Git帮助测试失败"
        failed_tests+=("帮助命令")
    fi
    
    # 测试3: 配置命令
    ((test_count++))
    print_status "[$test_count/7] 测试Git配置..."
    if "$git_binary" config --list >/dev/null 2>&1; then
        print_success "✓ 配置命令正常"
        ((passed_count++))
    else
        print_error "✗ Git配置测试失败"
        failed_tests+=("配置命令")
    fi
    
    # 创建测试仓库
    local test_dir="$BUILD_DIR/test_repo"
    rm -rf "$test_dir"
    mkdir -p "$test_dir"
    cd "$test_dir"
    
    # 测试4: 初始化仓库
    ((test_count++))
    print_status "[$test_count/7] 测试Git仓库初始化..."
    if "$git_binary" init >/dev/null 2>&1; then
        print_success "✓ 仓库初始化成功"
        ((passed_count++))
    else
        print_error "✗ Git init测试失败"
        failed_tests+=("仓库初始化")
        cd "$BUILD_DIR"
        rm -rf "$test_dir"
        show_test_summary $test_count $passed_count "${failed_tests[@]}"
        return 1
    fi
    
    # 测试5: 添加文件
    ((test_count++))
    print_status "[$test_count/7] 测试Git文件操作..."
    echo "# Git Android Test" > README.md
    echo "test content" > test.txt
    if "$git_binary" add . >/dev/null 2>&1; then
        print_success "✓ 文件添加成功"
        ((passed_count++))
    else
        print_error "✗ Git add测试失败"
        failed_tests+=("文件添加")
    fi
    
    # 测试6: 提交
    ((test_count++))
    print_status "[$test_count/7] 测试Git提交..."
    if "$git_binary" -c user.name="Git Android Test" -c user.email="test@android.com" commit -m "Initial commit" >/dev/null 2>&1; then
        print_success "✓ 提交操作成功"
        ((passed_count++))
    else
        print_error "✗ Git commit测试失败"
        failed_tests+=("提交操作")
    fi
    
    # 测试7: 日志查看
    ((test_count++))
    print_status "[$test_count/7] 测试Git日志..."
    if "$git_binary" log --oneline >/dev/null 2>&1; then
        print_success "✓ 日志查看成功"
        ((passed_count++))
    else
        print_error "✗ Git log测试失败"
        failed_tests+=("日志查看")
    fi
    
    cd "$BUILD_DIR"
    rm -rf "$test_dir"
    
    # 显示测试摘要
    show_test_summary $test_count $passed_count "${failed_tests[@]}"
    
    # 显示二进制信息
    show_binary_info "$git_binary"
    
    return $((test_count - passed_count))
}

# 显示测试摘要
show_test_summary() {
    local total=$1
    local passed=$2
    shift 2
    local failed_tests=("$@")
    local failed=$((total - passed))
    
    echo ""
    echo -e "${BOLD}${CYAN}═══ Git功能测试摘要 ═══${NC}"
    echo -e "总测试数: ${BOLD}$total${NC}"
    echo -e "通过: ${GREEN}${BOLD}$passed${NC}"
    echo -e "失败: ${RED}${BOLD}$failed${NC}"
    echo -e "成功率: ${BOLD}$((passed * 100 / total))%${NC}"
    
    if [ $failed -gt 0 ]; then
        echo ""
        echo -e "${RED}${BOLD}失败的测试:${NC}"
        for test in "${failed_tests[@]}"; do
            echo -e "  ${RED}✗${NC} $test"
        done
        echo ""
        print_warning "某些功能可能不完整，但基本Git操作应该可用"
    else
        echo ""
        print_success "🎉 所有测试通过！Git功能完整"
    fi
    echo ""
}

# 显示二进制信息
show_binary_info() {
    local git_binary="$1"
    
    echo -e "${BOLD}${PURPLE}═══ Git二进制信息 ═══${NC}"
    echo "文件路径: $git_binary"
    
    if command -v du >/dev/null 2>&1; then
        local size=$(du -h "$git_binary" 2>/dev/null | cut -f1 || echo "未知")
        echo "文件大小: $size"
    fi
    
    if command -v file >/dev/null 2>&1; then
        local arch_info=$(file "$git_binary" 2>/dev/null || echo "无法检测架构信息")
        echo "架构信息: $arch_info"
    fi
    
    if command -v ldd >/dev/null 2>&1 && [ "$COMPILE_MODE" != "static" ]; then
        echo "依赖库:"
        ldd "$git_binary" 2>/dev/null | head -10 || echo "  静态编译或无法检测依赖"
    fi
    
    echo ""
}

# 清理构建文件
cleanup() {
    if [ "$1" = "all" ]; then
        print_status "清理所有构建文件..."
        rm -rf "$BUILD_DIR"
        print_success "清理完成"
    else
        print_status "清理临时文件..."
        cd "$BUILD_DIR"
        if [ -d "git" ]; then
            cd git
            make clean >/dev/null 2>&1 || true
            cd ..
        fi
        print_success "临时文件清理完成"
    fi
}

# 显示使用帮助
show_help() {
    echo -e "${BOLD}${CYAN}Android Git 编译脚本使用说明${NC}"
    echo ""
    echo "这个脚本可以在多种环境下编译Git for Android："
    echo ""
    echo -e "${BOLD}支持的环境:${NC}"
    echo "• Termux (Android终端模拟器)"
    echo "• Android原生环境 (需要root)"  
    echo "• Linux交叉编译环境"
    echo "• WSL/WSL2"
    echo ""
    echo -e "${BOLD}编译模式:${NC}"
    echo "• Termux本机编译: 在Termux中直接编译"
    echo "• Android原生编译: 在Android系统中编译"
    echo "• 静态编译: 交叉编译生成静态链接的二进制文件"
    echo "• NDK编译: 使用Android NDK编译"
    echo "• 最小化编译: 编译功能最少的版本"
    echo ""
    echo -e "${BOLD}使用方法:${NC}"
    echo "./compile_git_android.sh          # 交互式选择编译模式"
    echo "./compile_git_android.sh auto     # 自动选择最佳模式"
    echo "./compile_git_android.sh clean    # 清理构建文件"
    echo "./compile_git_android.sh help     # 显示此帮助"
    echo ""
    echo -e "${BOLD}环境要求:${NC}"
    echo "• Termux: pkg install git clang make autoconf"
    echo "• 交叉编译: apt install gcc-aarch64-linux-gnu"
    echo "• NDK: 设置ANDROID_NDK_HOME环境变量"
    echo ""
}

# 主函数
main() {
    # 设置错误处理
    set -e
    trap 'handle_error $? $LINENO' ERR
    
    # 检查参数
    case "${1:-}" in
        "help"|"-h"|"--help")
            show_help
            exit 0
            ;;
        "clean")
            cleanup all
            exit 0
            ;;
        "auto")
            auto_select_mode
            ;;
        "resume")
            resume_build
            ;;
        "")
            show_menu
            ;;
        *)
            print_error "未知参数: $1"
            echo "使用 '$0 help' 查看帮助"
            exit 1
            ;;
    esac
    
    # 开始编译流程
    print_header
    print_status "开始Git Android编译流程..."
    log "编译开始，模式: $COMPILE_MODE，环境: $ENV_TYPE"
    
    # 保存编译状态
    save_build_state "started"
    
    # 执行编译步骤(带错误恢复)
    execute_build_steps
    
    # 显示完成信息
    show_completion_info
    
    # 保存完成状态
    save_build_state "completed"
    
    log "编译完成，模式: $COMPILE_MODE"
}

# 错误处理函数
handle_error() {
    local exit_code=$1
    local line_number=$2
    
    print_error "编译在第 $line_number 行出错，退出码: $exit_code"
    print_warning "编译被中断，可以使用 '$0 resume' 尝试继续"
    
    # 保存错误状态
    save_build_state "error:$exit_code:$line_number"
    
    # 显示错误日志
    if [ -f "$LOG_FILE" ]; then
        print_status "最近的日志输出:"
        tail -20 "$LOG_FILE" 2>/dev/null || true
    fi
    
    exit $exit_code
}

# 执行构建步骤
execute_build_steps() {
    local steps=(
        "install_dependencies:安装依赖"
        "setup_environment:设置环境"
        "compile_dependencies:编译依赖库"
        "get_git_source:获取Git源码"
        "configure_build:配置构建"
        "compile_git:编译Git"
        "install_git:安装Git"
        "create_package:创建包"
        "test_git:测试功能"
    )
    
    local total_steps=${#steps[@]}
    local current_step=0
    
    for step_info in "${steps[@]}"; do
        local step_func="${step_info%%:*}"
        local step_desc="${step_info##*:}"
        ((current_step++))
        
        print_status "[$current_step/$total_steps] $step_desc..."
        show_progress $current_step $total_steps "$step_desc"
        
        # 检查是否已完成该步骤
        if check_step_completed "$step_func"; then
            print_success "✓ $step_desc (已完成，跳过)"
            continue
        fi
        
        # 执行步骤
        save_build_state "executing:$step_func"
        if $step_func; then
            mark_step_completed "$step_func"
            print_success "✓ $step_desc 完成"
        else
            print_error "✗ $step_desc 失败"
            return 1
        fi
    done
}

# 断点续编功能
resume_build() {
    print_status "尝试从上次中断的地方继续编译..."
    
    if [ ! -f "$BUILD_DIR/.build_state" ]; then
        print_error "未找到构建状态文件，请重新开始编译"
        exit 1
    fi
    
    local last_state=$(cat "$BUILD_DIR/.build_state")
    print_status "上次构建状态: $last_state"
    
    case "$last_state" in
        completed)
            print_success "上次编译已完成"
            show_completion_info
            exit 0
            ;;
        error:*)
            print_warning "上次编译出现错误，尝试继续..."
            ;;
        executing:*)
            local step="${last_state#executing:}"
            print_status "从步骤 '$step' 继续执行"
            ;;
    esac
    
    # 检测编译模式
    if [ -f "$BUILD_DIR/.compile_mode" ]; then
        COMPILE_MODE=$(cat "$BUILD_DIR/.compile_mode")
        print_status "使用上次的编译模式: $COMPILE_MODE"
    else
        print_error "无法确定编译模式，请重新开始编译"
        exit 1
    fi
    
    # 继续执行构建步骤
    execute_build_steps
    show_completion_info
}

# 保存构建状态
save_build_state() {
    local state="$1"
    echo "$state" > "$BUILD_DIR/.build_state"
    echo "$COMPILE_MODE" > "$BUILD_DIR/.compile_mode"
    echo "$(date)" > "$BUILD_DIR/.build_time"
}

# 检查步骤是否完成
check_step_completed() {
    local step="$1"
    [ -f "$BUILD_DIR/.completed_$step" ]
}

# 标记步骤为已完成
mark_step_completed() {
    local step="$1"
    touch "$BUILD_DIR/.completed_$step"
}

# 显示完成信息
show_completion_info() {
    echo ""
    print_success "🎉 Git Android编译完成！"
    echo ""
    echo -e "${BOLD}${CYAN}═══ 编译摘要 ═══${NC}"
    echo -e "编译模式: ${PURPLE}$COMPILE_MODE${NC}"
    echo -e "Git版本: ${GREEN}$GIT_VERSION${NC}"
    echo -e "构建目录: $BUILD_DIR"
    echo -e "日志文件: $LOG_FILE"
    echo -e "完成时间: ${BLUE}$(date)${NC}"
    
    # 显示输出文件
    show_output_files
    
    # 显示ARM64依赖库信息
    show_dependencies_info
    
    # 显示安装说明
    show_installation_guide
    
    # 显示性能统计
    show_build_statistics
}

# 显示输出文件
show_output_files() {
    echo ""
    echo -e "${BOLD}${YELLOW}═══ 输出文件 ═══${NC}"
    
    # 查找生成的包文件
    local packages=($(find "$BUILD_DIR" -name "git-android-*.tar.gz" 2>/dev/null || true))
    if [ ${#packages[@]} -gt 0 ]; then
        for package in "${packages[@]}"; do
            local size=$(du -h "$package" 2>/dev/null | cut -f1 || echo "unknown")
            echo -e "📦 ${GREEN}$(basename "$package")${NC} ($size)"
        done
    fi
    
    # 查找安装脚本
    local scripts=($(find "$BUILD_DIR" -name "*.sh" -executable 2>/dev/null || true))
    if [ ${#scripts[@]} -gt 0 ]; then
        echo ""
        echo -e "${BOLD}安装脚本:${NC}"
        for script in "${scripts[@]}"; do
            echo -e "📜 ${CYAN}$(basename "$script")${NC}"
        done
    fi
}

# 显示依赖库信息
show_dependencies_info() {
    if [ -d "$BUILD_DIR/deps" ]; then
        echo ""
        echo -e "${BOLD}${GREEN}═══ ARM64依赖库 ═══${NC}"
        echo -e "依赖库目录: ${CYAN}$BUILD_DIR/deps${NC}"
        
        # 检查并显示已编译的库文件
        local lib_count=0
        for lib_file in "$BUILD_DIR/deps/lib"/*.a; do
            if [ -f "$lib_file" ]; then
                local lib_name=$(basename "$lib_file")
                local lib_size=$(du -h "$lib_file" 2>/dev/null | cut -f1 || echo "unknown")
                local lib_arch=$(file "$lib_file" 2>/dev/null | grep -o "aarch64\|ARM64\|arm64" || echo "unknown")
                echo -e "✓ ${GREEN}$lib_name${NC} ($lib_size) - ${PURPLE}$lib_arch${NC} 架构"
                ((lib_count++))
            fi
        done
        
        if [ $lib_count -eq 0 ]; then
            print_warning "⚠ 未找到编译的ARM64依赖库"
        else
            print_success "✅ 成功编译 $lib_count 个ARM64依赖库"
        fi
        
        # 检查NDK编译器信息
        if [ -n "$NDK_CC" ] && [ -f "$NDK_CC" ]; then
            echo ""
            echo -e "${BOLD}NDK编译器信息:${NC}"
            echo -e "编译器: ${CYAN}$NDK_CC${NC}"
            echo -e "目标: ${PURPLE}$NDK_TARGET$NDK_API${NC}"
            echo -e "架构: ${GREEN}ARM64 (aarch64)${NC}"
        fi
    fi
}

# 显示安装说明
show_installation_guide() {
    if [ "$COMPILE_MODE" != "termux_native" ] && [ "$COMPILE_MODE" != "android_native" ]; then
        echo ""
        echo -e "${BOLD}${CYAN}═══ 安装说明 ═══${NC}"
        echo -e "1. 将生成的 ${GREEN}tar.gz${NC} 文件复制到Android设备"
        echo -e "2. 解压到 ${CYAN}/data/local/tmp/${NC} 目录"
        echo -e "3. 运行 ${YELLOW}install_git_android.sh${NC} 安装脚本"
        echo -e "4. 或使用 ${YELLOW}quick_install.sh${NC} 快速安装"
        echo ""
        echo -e "${BOLD}包含的ARM64依赖库:${NC}"
        echo -e "• ${GREEN}OpenSSL${NC} (libssl.a, libcrypto.a) - 使用NDK编译"
        echo -e "• ${GREEN}zlib${NC} (libz.a) - 使用NDK编译"
        echo -e "• ${GREEN}curl${NC} (libcurl.a) - HTTP/HTTPS支持"
        echo -e "• 所有库文件均为 ${PURPLE}ARM64${NC} 架构，确保Android兼容性"
    fi
}

# 显示构建统计
show_build_statistics() {
    if [ -f "$BUILD_DIR/.build_time" ]; then
        local start_time=$(cat "$BUILD_DIR/.build_time" 2>/dev/null || echo "unknown")
        local end_time=$(date)
        
        echo ""
        echo -e "${BOLD}${PURPLE}═══ 构建统计 ═══${NC}"
        echo -e "开始时间: ${CYAN}$start_time${NC}"
        echo -e "结束时间: ${CYAN}$end_time${NC}"
        
        # 计算总大小
        local total_size=$(du -sh "$BUILD_DIR" 2>/dev/null | cut -f1 || echo "unknown")
        echo -e "构建目录大小: ${YELLOW}$total_size${NC}"
        
        # 显示日志大小
        if [ -f "$LOG_FILE" ]; then
            local log_size=$(du -h "$LOG_FILE" 2>/dev/null | cut -f1 || echo "unknown")
            echo -e "日志文件大小: ${YELLOW}$log_size${NC}"
        fi
    fi
}

# 捕获退出信号
trap 'print_error "编译被中断"; exit 1' INT TERM

# 检测运行环境
if [ -n "$TERMUX_VERSION" ]; then
    ENV_TYPE="termux"
elif [ -n "$ANDROID_ROOT" ] && [ -d "/system" ]; then
    ENV_TYPE="android_native"
elif grep -q "Microsoft\|WSL" /proc/version 2>/dev/null; then
    ENV_TYPE="wsl"
elif [ "$(uname -s)" = "Linux" ]; then
    ENV_TYPE="linux"
else
    ENV_TYPE="unknown"
fi

# 运行主函数
main "$@"
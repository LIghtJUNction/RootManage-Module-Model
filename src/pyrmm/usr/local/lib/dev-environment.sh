#!/bin/bash
# 开发环境配置库
# Development Environment Configuration Library
#
# 这个库提供开发环境的设置、配置和管理功能
# This library provides setup, configuration and management functions for development environment

# 配置文件路径
CONFIG_FILE="/usr/local/etc/kernelsu-dev.conf"
TEMPLATES_CONFIG="/usr/local/etc/templates.conf"

# 加载配置文件
# Load configuration file
load_dev_config() {
    if [[ -f "$CONFIG_FILE" ]]; then
        source "$CONFIG_FILE"
    else
        echo "Warning: Configuration file not found: $CONFIG_FILE"
        # 设置默认值
        KERNELSU_DEV_ROOT="/usr/local/share/kernelsu-dev"
        DEFAULT_MIN_API=21
        DEFAULT_TARGET_API=34
        LOG_LEVEL="INFO"
    fi
}

# 初始化开发环境
# Initialize development environment
init_dev_environment() {
    echo "Initializing KernelSU development environment..."
    
    # 加载配置
    load_dev_config
    
    # 创建必要的目录
    create_dev_directories
    
    # 设置环境变量
    setup_environment_variables
    
    # 检查依赖工具
    check_dependencies
    
    # 设置Shell配置
    setup_shell_config
    
    echo "Development environment initialized successfully!"
}

# 创建开发目录
# Create development directories
create_dev_directories() {
    local dirs=(
        "$KERNELSU_DEV_ROOT"
        "$KERNELSU_DEV_ROOT/templates"
        "$KERNELSU_DEV_ROOT/examples"
        "$KERNELSU_DEV_ROOT/docs"
        "$KERNELSU_DEV_ROOT/tools"
        "${CACHE_DIR:-$HOME/.cache/kernelsu-dev}"
        "${LOG_DIR:-$HOME/.local/share/kernelsu-dev/logs}"
    )
    
    for dir in "${dirs[@]}"; do
        if [[ ! -d "$dir" ]]; then
            mkdir -p "$dir"
            echo "Created directory: $dir"
        fi
    done
}

# 设置环境变量
# Setup environment variables
setup_environment_variables() {
    local env_file="$HOME/.kernelsu-dev-env"
    
    cat > "$env_file" << EOF
# KernelSU开发环境变量
# KernelSU Development Environment Variables

export KERNELSU_DEV_ROOT="$KERNELSU_DEV_ROOT"
export PATH="\$PATH:/usr/local/bin:/usr/bin"
export ANDROID_SDK_ROOT="\${ANDROID_SDK_ROOT:-/opt/android-sdk}"
export ANDROID_NDK_ROOT="\${ANDROID_NDK_ROOT:-\$ANDROID_SDK_ROOT/ndk}"

# 模块开发配置
export MODULE_DEFAULT_AUTHOR="${MODULE_DEFAULT_AUTHOR:-Unknown}"
export MODULE_DEFAULT_VERSION="${MODULE_DEFAULT_VERSION:-v1.0.0}"
export MODULE_DEFAULT_MIN_API="${DEFAULT_MIN_API:-21}"
export MODULE_DEFAULT_TARGET_API="${DEFAULT_TARGET_API:-34}"

# 开发工具配置
export ENABLE_COLOR_OUTPUT="${ENABLE_COLOR_OUTPUT:-true}"
export LOG_LEVEL="${LOG_LEVEL:-INFO}"
export DEBUG_ENABLED="\${DEBUG_ENABLED:-false}"

# 别名设置
alias module-create='create_project'
alias module-build='module-builder'
alias module-test='module-validator'
alias module-pack='module-packager'
alias module-deploy='deploy-module'
alias dev-setup='setup-dev-env'
alias dev-clean='clean_project'
EOF
    
    # 添加到Shell配置文件
    for shell_config in "$HOME/.bashrc" "$HOME/.zshrc"; do
        if [[ -f "$shell_config" ]]; then
            if ! grep -q "kernelsu-dev-env" "$shell_config"; then
                echo "source $env_file" >> "$shell_config"
                echo "Added environment setup to $shell_config"
            fi
        fi
    done
}

# 检查依赖工具
# Check dependencies
check_dependencies() {
    local required_tools=(
        "bash"
        "find"
        "grep"
        "sed"
        "awk"
        "zip"
        "unzip"
    )
    
    local optional_tools=(
        "git"
        "adb"
        "shellcheck"
        "code"
        "vim"
    )
    
    local missing_required=()
    local missing_optional=()
    
    echo "Checking required dependencies..."
    for tool in "${required_tools[@]}"; do
        if ! command -v "$tool" > /dev/null 2>&1; then
            missing_required+=("$tool")
        fi
    done
    
    echo "Checking optional dependencies..."
    for tool in "${optional_tools[@]}"; do
        if ! command -v "$tool" > /dev/null 2>&1; then
            missing_optional+=("$tool")
        fi
    done
    
    if [[ ${#missing_required[@]} -gt 0 ]]; then
        echo "Error: Missing required tools: ${missing_required[*]}"
        echo "Please install these tools before continuing."
        return 1
    fi
    
    if [[ ${#missing_optional[@]} -gt 0 ]]; then
        echo "Warning: Missing optional tools: ${missing_optional[*]}"
        echo "Some features may not be available."
    fi
    
    echo "Dependency check completed."
}

# 设置Shell配置
# Setup Shell configuration
setup_shell_config() {
    # 创建Shell函数文件
    local functions_file="$HOME/.kernelsu-dev-functions"
    
    cat > "$functions_file" << 'EOF'
# KernelSU开发环境函数
# KernelSU Development Environment Functions

# 快速创建模块项目
mkmodule() {
    local name="$1"
    local type="${2:-basic}"
    
    if [[ -z "$name" ]]; then
        echo "Usage: mkmodule <name> [type]"
        echo "Types: basic, magisk-compat, webui, system-modifier, app-patcher"
        return 1
    fi
    
    create_project "$name" "$type" "$name"
}

# 快速构建当前模块
build() {
    if [[ -f "module.prop" ]]; then
        module-builder .
    else
        echo "Error: Not in a module directory"
        return 1
    fi
}

# 快速验证当前模块
validate() {
    if [[ -f "module.prop" ]]; then
        module-validator .
    else
        echo "Error: Not in a module directory"
        return 1
    fi
}

# 快速打包当前模块
pack() {
    if [[ -f "module.prop" ]]; then
        module-packager .
    else
        echo "Error: Not in a module directory"
        return 1
    fi
}

# 显示模块信息
info() {
    if [[ -f ".kernelsu-project" ]]; then
        get_project_info
    elif [[ -f "module.prop" ]]; then
        echo "Module Properties:"
        cat module.prop
    else
        echo "Error: Not in a module directory"
        return 1
    fi
}

# 清理构建文件
clean() {
    if [[ -f ".kernelsu-project" ]]; then
        clean_project
    else
        echo "Cleaning build files..."
        rm -rf *.zip build/ dist/ output/
        find . -name "*.tmp" -delete
        find . -name "*.log" -delete
    fi
}

# 开启调试模式
debug-on() {
    export DEBUG_ENABLED=true
    export VERBOSE_OUTPUT=true
    echo "Debug mode enabled"
}

# 关闭调试模式
debug-off() {
    export DEBUG_ENABLED=false
    export VERBOSE_OUTPUT=false
    echo "Debug mode disabled"
}
EOF
    
    # 添加函数到Shell配置
    for shell_config in "$HOME/.bashrc" "$HOME/.zshrc"; do
        if [[ -f "$shell_config" ]]; then
            if ! grep -q "kernelsu-dev-functions" "$shell_config"; then
                echo "source $functions_file" >> "$shell_config"
                echo "Added development functions to $shell_config"
            fi
        fi
    done
}

# 获取开发环境状态
# Get development environment status
get_dev_status() {
    echo "KernelSU Development Environment Status"
    echo "======================================"
    
    echo "Configuration:"
    echo "  Dev Root: ${KERNELSU_DEV_ROOT:-Not set}"
    echo "  Config File: ${CONFIG_FILE}"
    echo "  Log Level: ${LOG_LEVEL:-INFO}"
    
    echo ""
    echo "Tools:"
    local tools=("module-builder" "module-validator" "module-packager" "deploy-module")
    for tool in "${tools[@]}"; do
        if command -v "$tool" > /dev/null 2>&1; then
            echo "  ✓ $tool"
        else
            echo "  ✗ $tool (not found)"
        fi
    done
    
    echo ""
    echo "Dependencies:"
    local deps=("git" "adb" "shellcheck" "code")
    for dep in "${deps[@]}"; do
        if command -v "$dep" > /dev/null 2>&1; then
            echo "  ✓ $dep"
        else
            echo "  ✗ $dep (not found)"
        fi
    done
}

# 更新开发环境
# Update development environment
update_dev_environment() {
    echo "Updating KernelSU development environment..."
    
    # 重新初始化环境
    init_dev_environment
    
    # 更新工具
    echo "Updating development tools..."
    
    # 重新加载Shell配置
    echo "Please restart your shell or run 'source ~/.bashrc' to apply changes."
}

# 导出函数
# Export functions
export -f load_dev_config
export -f init_dev_environment
export -f create_dev_directories
export -f setup_environment_variables
export -f check_dependencies
export -f setup_shell_config
export -f get_dev_status
export -f update_dev_environment

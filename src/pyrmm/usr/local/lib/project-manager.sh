#!/bin/bash
# 项目管理库
# Project Management Library
#
# 这个库提供项目创建、配置和管理功能
# This library provides project creation, configuration and management functions

# 项目配置
PROJECT_CONFIG_FILE=".kernelsu-project"
BUILD_CONFIG_FILE="build.conf"
VSCODE_DIR=".vscode"

# 创建新项目
# Create new project
create_project() {
    local project_name="$1"
    local project_type="$2"
    local project_dir="$3"
    
    if [[ -z "$project_name" || -z "$project_type" ]]; then
        echo "Error: Project name and type are required"
        return 1
    fi
    
    project_dir="${project_dir:-$project_name}"
    
    # 创建项目目录
    mkdir -p "$project_dir"
    cd "$project_dir" || return 1
    
    # 创建项目配置文件
    cat > "$PROJECT_CONFIG_FILE" << EOF
# KernelSU模块项目配置
# KernelSU Module Project Configuration

PROJECT_NAME="$project_name"
PROJECT_TYPE="$project_type"
PROJECT_VERSION="1.0.0"
PROJECT_AUTHOR="\${USER:-Unknown}"
CREATED_DATE="\$(date '+%Y-%m-%d %H:%M:%S')"

# 构建配置
MIN_API=21
TARGET_API=34
MIN_KERNELSU=10940

# 开发配置
ENABLE_DEBUG=true
ENABLE_LINT=true
ENABLE_TEST=true
EOF
    
    # 创建构建配置
    cat > "$BUILD_CONFIG_FILE" << EOF
# 构建配置文件
# Build Configuration File

BUILD_TYPE=debug
ARCH=arm64
ENABLE_WEBUI=false
ENABLE_MAGISK_COMPAT=true

# 编译选项
OPTIMIZE_LEVEL=2
STRIP_SYMBOLS=false
INCLUDE_DEBUG_INFO=true
EOF
    
    # 创建VS Code配置
    setup_vscode_config
    
    # 创建Git配置
    setup_git_config
    
    echo "Project '$project_name' created successfully in '$project_dir'"
    return 0
}

# 设置VS Code配置
# Setup VS Code configuration
setup_vscode_config() {
    mkdir -p "$VSCODE_DIR"
    
    # 创建任务配置
    cat > "$VSCODE_DIR/tasks.json" << 'EOF'
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Build Module",
            "type": "shell",
            "command": "module-builder",
            "args": ["--config", "build.conf"],
            "group": {
                "kind": "build",
                "isDefault": true
            },
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "shared"
            },
            "problemMatcher": []
        },
        {
            "label": "Validate Module",
            "type": "shell",
            "command": "module-validator",
            "args": ["."],
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "shared"
            }
        },
        {
            "label": "Package Module",
            "type": "shell",
            "command": "module-packager",
            "args": ["."],
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "shared"
            }
        },
        {
            "label": "Deploy Module",
            "type": "shell",
            "command": "deploy-module",
            "args": ["--module", "*.zip"],
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "shared"
            }
        }
    ]
}
EOF
    
    # 创建启动配置
    cat > "$VSCODE_DIR/launch.json" << 'EOF'
{
    "version": "0.2.0",
    "configurations": [
        {
            "name": "Debug Shell Script",
            "type": "bashdb",
            "request": "launch",
            "program": "${file}",
            "cwd": "${workspaceFolder}",
            "args": [],
            "console": "integratedTerminal"
        }
    ]
}
EOF
    
    # 创建设置配置
    cat > "$VSCODE_DIR/settings.json" << 'EOF'
{
    "files.associations": {
        "*.prop": "properties",
        "*.sh": "shellscript"
    },
    "shellcheck.enable": true,
    "shellcheck.executablePath": "shellcheck",
    "terminal.integrated.defaultProfile.linux": "bash",
    "editor.insertSpaces": true,
    "editor.tabSize": 4,
    "files.eol": "\n",
    "files.trimTrailingWhitespace": true,
    "files.insertFinalNewline": true
}
EOF
    
    # 创建扩展推荐
    cat > "$VSCODE_DIR/extensions.json" << 'EOF'
{
    "recommendations": [
        "timonwong.shellcheck",
        "ms-vscode.vscode-json",
        "rogalmic.bash-debug",
        "mads-hartmann.bash-ide-vscode"
    ]
}
EOF
}

# 设置Git配置
# Setup Git configuration
setup_git_config() {
    # 创建.gitignore
    cat > ".gitignore" << 'EOF'
# 构建输出
*.zip
build/
dist/
output/

# 临时文件
*.tmp
*.temp
.DS_Store
Thumbs.db

# 日志文件
*.log
logs/

# IDE配置
.idea/
*.swp
*.swo
*~

# 系统文件
.directory
*.orig

# 缓存文件
.cache/
node_modules/
EOF
    
    # 初始化Git仓库
    if command -v git > /dev/null 2>&1; then
        git init
        git add .
        git commit -m "Initial commit: KernelSU module project setup"
    fi
}

# 获取项目信息
# Get project information
get_project_info() {
    if [[ ! -f "$PROJECT_CONFIG_FILE" ]]; then
        echo "Error: Not a KernelSU module project directory"
        return 1
    fi
    
    source "$PROJECT_CONFIG_FILE"
    
    cat << EOF
Project Information:
  Name: $PROJECT_NAME
  Type: $PROJECT_TYPE
  Version: $PROJECT_VERSION
  Author: $PROJECT_AUTHOR
  Created: $CREATED_DATE
  
Build Configuration:
  Min API: $MIN_API
  Target API: $TARGET_API
  Min KernelSU: $MIN_KERNELSU
EOF
}

# 更新项目配置
# Update project configuration
update_project_config() {
    local key="$1"
    local value="$2"
    
    if [[ ! -f "$PROJECT_CONFIG_FILE" ]]; then
        echo "Error: Not a KernelSU module project directory"
        return 1
    fi
    
    # 使用sed更新配置值
    sed -i "s/^${key}=.*/${key}=\"${value}\"/" "$PROJECT_CONFIG_FILE"
    echo "Updated $key to $value"
}

# 清理项目
# Clean project
clean_project() {
    echo "Cleaning project..."
    
    # 删除构建输出
    rm -rf build/ dist/ output/ *.zip
    
    # 删除临时文件
    find . -name "*.tmp" -delete
    find . -name "*.temp" -delete
    find . -name "*.log" -delete
    
    echo "Project cleaned successfully"
}

# 验证项目结构
# Validate project structure
validate_project() {
    local errors=0
    
    if [[ ! -f "$PROJECT_CONFIG_FILE" ]]; then
        echo "Error: Missing project configuration file"
        ((errors++))
    fi
    
    if [[ ! -f "module.prop" ]]; then
        echo "Error: Missing module.prop file"
        ((errors++))
    fi
    
    if [[ ! -f "META-INF/com/google/android/updater-script" ]]; then
        echo "Warning: Missing updater-script"
    fi
    
    if [[ ! -f "service.sh" ]]; then
        echo "Warning: Missing service.sh"
    fi
    
    if [[ $errors -eq 0 ]]; then
        echo "Project structure validation passed"
        return 0
    else
        echo "Project structure validation failed with $errors errors"
        return 1
    fi
}

# 导出项目函数
# Export project functions
export -f create_project
export -f setup_vscode_config
export -f setup_git_config
export -f get_project_info
export -f update_project_config
export -f clean_project
export -f validate_project

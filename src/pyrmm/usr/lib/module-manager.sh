#!/system/bin/sh
# KernelSU 模块管理库
# 提供模块安装、卸载、更新等核心功能

# 包含通用函数库
. /usr/lib/common-functions.sh

# 模块状态常量
readonly MODULE_STATE_ENABLED=1
readonly MODULE_STATE_DISABLED=0
readonly MODULE_STATE_REMOVED=-1

# 模块类型常量
readonly MODULE_TYPE_SYSTEM=1
readonly MODULE_TYPE_USER=2
readonly MODULE_TYPE_BOOT=3

#######################################
# 获取模块信息
# 参数:
#   $1 - 模块ID
# 返回:
#   0 - 成功，1 - 失败
#######################################
get_module_info() {
    local module_id="$1"
    local module_dir="/data/adb/modules/$module_id"
    
    if [ ! -d "$module_dir" ]; then
        log_error "模块不存在: $module_id"
        return 1
    fi
    
    local module_prop="$module_dir/module.prop"
    if [ ! -f "$module_prop" ]; then
        log_error "模块配置文件不存在: $module_prop"
        return 1
    fi
    
    echo "模块信息:"
    echo "  ID: $module_id"
    echo "  路径: $module_dir"
    
    while IFS='=' read -r key value; do
        case "$key" in
            "id"|"name"|"version"|"versionCode"|"author"|"description")
                echo "  $key: $value"
                ;;
        esac
    done < "$module_prop"
    
    # 检查模块状态
    if [ -f "$module_dir/disable" ]; then
        echo "  状态: 已禁用"
    elif [ -f "$module_dir/remove" ]; then
        echo "  状态: 待删除"
    else
        echo "  状态: 已启用"
    fi
    
    return 0
}

#######################################
# 列出所有模块
# 返回:
#   0 - 成功
#######################################
list_modules() {
    local modules_dir="/data/adb/modules"
    
    if [ ! -d "$modules_dir" ]; then
        log_info "模块目录不存在"
        return 0
    fi
    
    log_info "已安装的模块:"
    
    for module_dir in "$modules_dir"/*; do
        if [ -d "$module_dir" ]; then
            local module_id=$(basename "$module_dir")
            local module_prop="$module_dir/module.prop"
            
            if [ -f "$module_prop" ]; then
                local name=$(grep "^name=" "$module_prop" | cut -d'=' -f2)
                local version=$(grep "^version=" "$module_prop" | cut -d'=' -f2)
                local status="启用"
                
                if [ -f "$module_dir/disable" ]; then
                    status="禁用"
                elif [ -f "$module_dir/remove" ]; then
                    status="待删除"
                fi
                
                printf "  %-20s %-30s %-10s %s\n" "$module_id" "${name:-未知}" "${version:-未知}" "$status"
            fi
        fi
    done
    
    return 0
}

#######################################
# 启用模块
# 参数:
#   $1 - 模块ID
# 返回:
#   0 - 成功，1 - 失败
#######################################
enable_module() {
    local module_id="$1"
    local module_dir="/data/adb/modules/$module_id"
    
    if [ ! -d "$module_dir" ]; then
        log_error "模块不存在: $module_id"
        return 1
    fi
    
    # 删除禁用标记文件
    if [ -f "$module_dir/disable" ]; then
        rm -f "$module_dir/disable"
        log_info "模块已启用: $module_id"
    else
        log_info "模块已经是启用状态: $module_id"
    fi
    
    # 删除删除标记文件
    if [ -f "$module_dir/remove" ]; then
        rm -f "$module_dir/remove"
        log_info "取消删除标记: $module_id"
    fi
    
    return 0
}

#######################################
# 禁用模块
# 参数:
#   $1 - 模块ID
# 返回:
#   0 - 成功，1 - 失败
#######################################
disable_module() {
    local module_id="$1"
    local module_dir="/data/adb/modules/$module_id"
    
    if [ ! -d "$module_dir" ]; then
        log_error "模块不存在: $module_id"
        return 1
    fi
    
    # 创建禁用标记文件
    touch "$module_dir/disable"
    log_info "模块已禁用: $module_id"
    
    return 0
}

#######################################
# 标记删除模块
# 参数:
#   $1 - 模块ID
# 返回:
#   0 - 成功，1 - 失败
#######################################
remove_module() {
    local module_id="$1"
    local module_dir="/data/adb/modules/$module_id"
    
    if [ ! -d "$module_dir" ]; then
        log_error "模块不存在: $module_id"
        return 1
    fi
    
    # 创建删除标记文件
    touch "$module_dir/remove"
    log_info "模块已标记删除，重启后生效: $module_id"
    
    return 0
}

#######################################
# 安装模块包
# 参数:
#   $1 - 模块包路径
# 返回:
#   0 - 成功，1 - 失败
#######################################
install_module() {
    local module_zip="$1"
    local temp_dir="/tmp/module_install_$$"
    
    if [ ! -f "$module_zip" ]; then
        log_error "模块包不存在: $module_zip"
        return 1
    fi
    
    log_info "开始安装模块包: $module_zip"
    
    # 创建临时目录
    mkdir -p "$temp_dir"
    
    # 解压模块包
    if ! unzip -q "$module_zip" -d "$temp_dir"; then
        log_error "解压模块包失败"
        rm -rf "$temp_dir"
        return 1
    fi
    
    # 检查模块结构
    if [ ! -f "$temp_dir/module.prop" ]; then
        log_error "无效的模块包: 缺少 module.prop"
        rm -rf "$temp_dir"
        return 1
    fi
    
    # 获取模块ID
    local module_id=$(grep "^id=" "$temp_dir/module.prop" | cut -d'=' -f2)
    if [ -z "$module_id" ]; then
        log_error "无效的模块包: 无法获取模块ID"
        rm -rf "$temp_dir"
        return 1
    fi
    
    local module_dir="/data/adb/modules/$module_id"
    
    # 备份现有模块
    if [ -d "$module_dir" ]; then
        log_info "备份现有模块: $module_id"
        cp -r "$module_dir" "$module_dir.bak"
    fi
    
    # 创建模块目录
    mkdir -p "$module_dir"
    
    # 复制模块文件
    cp -r "$temp_dir"/* "$module_dir/"
    
    # 设置权限
    chmod -R 755 "$module_dir"
    
    # 运行安装脚本
    if [ -f "$module_dir/META-INF/com/google/android/update-binary" ]; then
        log_info "执行模块安装脚本"
        # 这里需要模拟KernelSU的安装环境
        # 实际实现需要更复杂的逻辑
    fi
    
    # 清理临时文件
    rm -rf "$temp_dir"
    
    log_info "模块安装完成: $module_id"
    return 0
}

#######################################
# 检查模块完整性
# 参数:
#   $1 - 模块ID
# 返回:
#   0 - 完整，1 - 不完整
#######################################
check_module_integrity() {
    local module_id="$1"
    local module_dir="/data/adb/modules/$module_id"
    
    if [ ! -d "$module_dir" ]; then
        log_error "模块不存在: $module_id"
        return 1
    fi
    
    # 检查必需文件
    local required_files="module.prop"
    local integrity_ok=true
    
    for file in $required_files; do
        if [ ! -f "$module_dir/$file" ]; then
            log_error "缺少必需文件: $file"
            integrity_ok=false
        fi
    done
    
    # 检查module.prop格式
    if [ -f "$module_dir/module.prop" ]; then
        local required_props="id name version versionCode author"
        
        for prop in $required_props; do
            if ! grep -q "^$prop=" "$module_dir/module.prop"; then
                log_warn "缺少属性: $prop"
            fi
        done
    fi
    
    if [ "$integrity_ok" = true ]; then
        log_info "模块完整性检查通过: $module_id"
        return 0
    else
        log_error "模块完整性检查失败: $module_id"
        return 1
    fi
}

#######################################
# 更新模块
# 参数:
#   $1 - 模块ID
#   $2 - 新模块包路径
# 返回:
#   0 - 成功，1 - 失败
#######################################
update_module() {
    local module_id="$1"
    local new_module_zip="$2"
    
    log_info "更新模块: $module_id"
    
    # 检查现有模块
    if [ ! -d "/data/adb/modules/$module_id" ]; then
        log_error "模块不存在: $module_id"
        return 1
    fi
    
    # 安装新版本
    if install_module "$new_module_zip"; then
        log_info "模块更新完成: $module_id"
        return 0
    else
        log_error "模块更新失败: $module_id"
        
        # 恢复备份
        if [ -d "/data/adb/modules/$module_id.bak" ]; then
            log_info "恢复模块备份"
            rm -rf "/data/adb/modules/$module_id"
            mv "/data/adb/modules/$module_id.bak" "/data/adb/modules/$module_id"
        fi
        
        return 1
    fi
}

# 导出函数
export -f get_module_info list_modules enable_module disable_module
export -f remove_module install_module check_module_integrity update_module

#######################################
# 检查模块依赖
# 参数:
#   $1 - 模块ID
# 返回:
#   0 - 依赖满足，1 - 依赖不满足
#######################################
check_module_dependencies() {
    local module_id="$1"
    local module_dir="/data/adb/modules/$module_id"
    local deps_file="$module_dir/dependencies.txt"
    
    if [ ! -f "$deps_file" ]; then
        return 0  # 无依赖要求
    fi
    
    log_info "检查模块依赖: $module_id"
    local deps_ok=true
    
    while IFS= read -r dependency; do
        # 跳过注释和空行
        [ -z "$dependency" ] && continue
        [ "${dependency#\#}" != "$dependency" ] && continue
        
        # 解析依赖格式: module_id[>=version]
        local dep_id="${dependency%\[*}"
        local dep_version="${dependency#*\[}"
        dep_version="${dep_version%\]}"
        
        if [ ! -d "/data/adb/modules/$dep_id" ]; then
            log_error "缺少依赖模块: $dep_id"
            deps_ok=false
            continue
        fi
        
        # 检查版本要求
        if [ "$dep_version" != "$dependency" ]; then
            local current_version=$(grep "^version=" "/data/adb/modules/$dep_id/module.prop" | cut -d'=' -f2)
            if ! version_compare "$current_version" "$dep_version"; then
                log_error "依赖模块版本不满足: $dep_id (需要 $dep_version, 当前 $current_version)"
                deps_ok=false
            fi
        fi
        
        log_debug "依赖检查通过: $dep_id"
    done < "$deps_file"
    
    if [ "$deps_ok" = true ]; then
        log_info "所有依赖检查通过"
        return 0
    else
        log_error "依赖检查失败"
        return 1
    fi
}

#######################################
# 检查模块更新
# 参数:
#   $1 - 模块ID
# 返回:
#   0 - 有更新，1 - 无更新，2 - 检查失败
#######################################
check_module_updates() {
    local module_id="$1"
    local module_dir="/data/adb/modules/$module_id"
    local module_prop="$module_dir/module.prop"
    
    if [ ! -f "$module_prop" ]; then
        log_error "模块配置文件不存在: $module_prop"
        return 2
    fi
    
    # 获取更新URL
    local update_url=$(grep "^updateJson=" "$module_prop" | cut -d'=' -f2)
    if [ -z "$update_url" ]; then
        log_debug "模块未配置更新URL: $module_id"
        return 1
    fi
    
    log_info "检查模块更新: $module_id"
    
    # 下载更新信息
    local temp_file="/tmp/update_info_$$"
    if ! download_file "$update_url" "$temp_file"; then
        log_error "下载更新信息失败"
        return 2
    fi
    
    # 解析更新信息
    local current_version=$(grep "^version=" "$module_prop" | cut -d'=' -f2)
    local current_code=$(grep "^versionCode=" "$module_prop" | cut -d'=' -f2)
    local remote_version=$(grep "\"version\"" "$temp_file" | cut -d'"' -f4)
    local remote_code=$(grep "\"versionCode\"" "$temp_file" | cut -d':' -f2 | tr -d ' ,')
    
    rm -f "$temp_file"
    
    if [ -z "$remote_version" ] || [ -z "$remote_code" ]; then
        log_error "解析更新信息失败"
        return 2
    fi
    
    if [ "$remote_code" -gt "$current_code" ]; then
        log_info "发现新版本: $current_version -> $remote_version"
        return 0
    else
        log_info "已是最新版本: $current_version"
        return 1
    fi
}

#######################################
# 批量检查所有模块更新
# 返回:
#   更新数量
#######################################
check_all_updates() {
    local modules_dir="/data/adb/modules"
    local update_count=0
    
    log_info "检查所有模块更新..."
    
    for module_dir in "$modules_dir"/*; do
        if [ -d "$module_dir" ]; then
            local module_id=$(basename "$module_dir")
            if check_module_updates "$module_id"; then
                update_count=$((update_count + 1))
            fi
        fi
    done
    
    log_info "共发现 $update_count 个模块有更新"
    return $update_count
}

#######################################
# 备份模块
# 参数:
#   $1 - 模块ID
#   $2 - 备份路径 (可选)
# 返回:
#   0 - 成功，1 - 失败
#######################################
backup_module() {
    local module_id="$1"
    local backup_path="${2:-/data/adb/module_backups}"
    local module_dir="/data/adb/modules/$module_id"
    
    if [ ! -d "$module_dir" ]; then
        log_error "模块不存在: $module_id"
        return 1
    fi
    
    # 创建备份目录
    mkdir -p "$backup_path"
    
    # 生成备份文件名
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local backup_file="$backup_path/${module_id}_${timestamp}.tar.gz"
    
    log_info "备份模块: $module_id -> $backup_file"
    
    # 创建压缩备份
    if tar -czf "$backup_file" -C "/data/adb/modules" "$module_id"; then
        log_info "模块备份成功: $backup_file"
        echo "$backup_file"
        return 0
    else
        log_error "模块备份失败: $module_id"
        return 1
    fi
}

#######################################
# 恢复模块备份
# 参数:
#   $1 - 备份文件路径
# 返回:
#   0 - 成功，1 - 失败
#######################################
restore_module() {
    local backup_file="$1"
    
    if [ ! -f "$backup_file" ]; then
        log_error "备份文件不存在: $backup_file"
        return 1
    fi
    
    log_info "恢复模块备份: $backup_file"
    
    # 解压到临时目录
    local temp_dir="/tmp/module_restore_$$"
    mkdir -p "$temp_dir"
    
    if ! tar -xzf "$backup_file" -C "$temp_dir"; then
        log_error "解压备份文件失败"
        rm -rf "$temp_dir"
        return 1
    fi
    
    # 找到模块目录
    local module_id=$(ls "$temp_dir" | head -1)
    if [ -z "$module_id" ]; then
        log_error "备份文件格式错误"
        rm -rf "$temp_dir"
        return 1
    fi
    
    local target_dir="/data/adb/modules/$module_id"
    
    # 备份现有模块
    if [ -d "$target_dir" ]; then
        log_info "备份现有模块"
        mv "$target_dir" "$target_dir.old"
    fi
    
    # 恢复模块
    mv "$temp_dir/$module_id" "$target_dir"
    
    # 清理临时文件
    rm -rf "$temp_dir"
    
    log_info "模块恢复完成: $module_id"
    return 0
}

#######################################
# 列出模块备份
# 参数:
#   $1 - 备份目录 (可选)
#######################################
list_module_backups() {
    local backup_path="${1:-/data/adb/module_backups}"
    
    if [ ! -d "$backup_path" ]; then
        log_info "备份目录不存在: $backup_path"
        return 0
    fi
    
    log_info "模块备份列表:"
    printf "%-20s %-15s %-12s %s\n" "模块ID" "备份时间" "大小" "文件名"
    echo "------------------------------------------------------------"
    
    for backup_file in "$backup_path"/*.tar.gz; do
        if [ -f "$backup_file" ]; then
            local filename=$(basename "$backup_file")
            local module_id="${filename%_*}"
            local timestamp="${filename#*_}"
            timestamp="${timestamp%.tar.gz}"
            local size=$(ls -lh "$backup_file" | awk '{print $5}')
            
            printf "%-20s %-15s %-12s %s\n" "$module_id" "$timestamp" "$size" "$filename"
        fi
    done
}

#######################################
# 验证模块签名 (如果支持)
# 参数:
#   $1 - 模块ID或模块包路径
# 返回:
#   0 - 验证通过，1 - 验证失败，2 - 不支持验证
#######################################
verify_module_signature() {
    local module_path="$1"
    
    # 检查是否为已安装模块
    if [ -d "/data/adb/modules/$module_path" ]; then
        module_path="/data/adb/modules/$module_path"
    fi
    
    # 查找签名文件
    local sig_file=""
    if [ -d "$module_path" ]; then
        sig_file="$module_path/module.sig"
    elif [ -f "$module_path" ]; then
        # 从zip包中提取签名
        local temp_dir="/tmp/sig_verify_$$"
        mkdir -p "$temp_dir"
        if unzip -q "$module_path" "module.sig" -d "$temp_dir" 2>/dev/null; then
            sig_file="$temp_dir/module.sig"
        fi
    fi
    
    if [ ! -f "$sig_file" ]; then
        log_debug "模块未签名或签名文件不存在"
        return 2
    fi
    
    # 这里需要实现具体的签名验证逻辑
    # 目前只做基本检查
    log_info "检查模块签名..."
    
    # 清理临时文件
    [ -d "/tmp/sig_verify_$$" ] && rm -rf "/tmp/sig_verify_$$"
    
    log_info "签名验证功能尚未完全实现"
    return 2
}

#######################################
# 获取模块统计信息
# 返回:
#   0 - 成功
#######################################
get_module_statistics() {
    local modules_dir="/data/adb/modules"
    local total_count=0
    local enabled_count=0
    local disabled_count=0
    local pending_remove_count=0
    local total_size=0
    
    if [ ! -d "$modules_dir" ]; then
        log_info "模块目录不存在"
        return 0
    fi
    
    for module_dir in "$modules_dir"/*; do
        if [ -d "$module_dir" ]; then
            total_count=$((total_count + 1))
            
            # 计算模块大小
            local size=$(du -sk "$module_dir" 2>/dev/null | cut -f1)
            total_size=$((total_size + size))
            
            # 统计状态
            if [ -f "$module_dir/remove" ]; then
                pending_remove_count=$((pending_remove_count + 1))
            elif [ -f "$module_dir/disable" ]; then
                disabled_count=$((disabled_count + 1))
            else
                enabled_count=$((enabled_count + 1))
            fi
        fi
    done
    
    echo "模块统计信息:"
    echo "  总数量: $total_count"
    echo "  已启用: $enabled_count"
    echo "  已禁用: $disabled_count"
    echo "  待删除: $pending_remove_count"
    echo "  总大小: $(human_readable_size $((total_size * 1024)))"
    
    return 0
}

#######################################
# 清理模块缓存和临时文件
# 返回:
#   0 - 成功
#######################################
cleanup_module_cache() {
    log_info "清理模块缓存和临时文件..."
    
    local cleanup_paths=(
        "/tmp/module_*"
        "/data/adb/modules_update"
        "/data/adb/modules/.core"
        "/cache/module_*"
    )
    
    local cleaned_size=0
    
    for path_pattern in "${cleanup_paths[@]}"; do
        for path in $path_pattern; do
            if [ -e "$path" ]; then
                local size=$(du -sk "$path" 2>/dev/null | cut -f1)
                rm -rf "$path"
                cleaned_size=$((cleaned_size + size))
                log_debug "清理: $path"
            fi
        done
    done
    
    log_info "清理完成，释放空间: $(human_readable_size $((cleaned_size * 1024)))"
    return 0
}

#######################################
# 重新加载所有模块
# 返回:
#   0 - 成功
#######################################
reload_modules() {
    log_info "重新加载模块配置..."
    
    # 通知KernelSU重新扫描模块
    if command -v ksud >/dev/null 2>&1; then
        ksud module list >/dev/null 2>&1
        log_info "KernelSU模块重新加载完成"
    else
        log_warn "KernelSU守护进程不可用，需要重启生效"
    fi
    
    return 0
}

# 导出新增函数
export -f check_module_dependencies check_module_updates check_all_updates
export -f backup_module restore_module list_module_backups
export -f verify_module_signature get_module_statistics
export -f cleanup_module_cache reload_modules

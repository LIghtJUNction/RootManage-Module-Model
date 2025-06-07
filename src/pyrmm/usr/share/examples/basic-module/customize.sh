#!/system/bin/sh
# 基础模块安装脚本示例
# Basic Module Installation Script Example

# 导入通用函数
. /usr/lib/common-functions.sh

# 安装过程
log_info "开始安装基础示例模块..."

# 检查环境
if ! check_kernelsu_environment; then
    log_error "KernelSU环境检查失败"
    exit 1
fi

# 检查API级别
current_api=$(get_api_level)
if [ "$current_api" -lt 21 ]; then
    log_error "不支持的API级别: $current_api (需要 >= 21)"
    exit 1
fi

# 创建模块文件结构
log_info "创建模块文件结构..."

# 创建系统文件替换示例
mkdir -p "$MODPATH/system/etc"
cat > "$MODPATH/system/etc/basic_module.conf" << 'EOF'
# 基础模块配置文件
enabled=true
log_level=info
EOF

# 设置权限
set_perm "$MODPATH/system/etc/basic_module.conf" 0 0 0644

# 创建服务脚本
cat > "$MODPATH/service.sh" << 'EOF'
#!/system/bin/sh
# 基础模块服务脚本

# 模块启动日志
echo "$(date): 基础示例模块服务启动" >> /data/adb/modules/basic_example/service.log

# 这里可以添加模块特定的服务代码
# 例如：启动守护进程、设置系统属性等

EOF

chmod 755 "$MODPATH/service.sh"

log_info "基础示例模块安装完成"

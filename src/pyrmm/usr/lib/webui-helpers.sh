#!/bin/bash
# WebUI Helper Library for KernelSU Modules
# Web界面辅助库

# 防止重复加载
if [ "${WEBUI_HELPERS_LOADED:-0}" = "1" ]; then
    return 0
fi
readonly WEBUI_HELPERS_LOADED=1

# 导入公共函数
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [ -f "$SCRIPT_DIR/common-functions.sh" ]; then
    source "$SCRIPT_DIR/common-functions.sh"
fi

# WebUI配置
readonly DEFAULT_WEBUI_PORT="8080"
readonly WEBUI_INDEX="index.html"
readonly WEBUI_CONFIG="webui.conf"
readonly WEBUI_PID_FILE="/data/local/tmp/webui.pid"
readonly WEBUI_LOG_FILE="/data/local/tmp/webui.log"
readonly WEBUI_ACCESS_LOG="/data/local/tmp/webui_access.log"

# 支持的MIME类型
declare -A MIME_TYPES=(
    ["html"]="text/html"
    ["htm"]="text/html"
    ["css"]="text/css"
    ["js"]="application/javascript"
    ["json"]="application/json"
    ["xml"]="application/xml"
    ["txt"]="text/plain"
    ["png"]="image/png"
    ["jpg"]="image/jpeg"
    ["jpeg"]="image/jpeg"
    ["gif"]="image/gif"
    ["svg"]="image/svg+xml"
    ["ico"]="image/x-icon"
    ["pdf"]="application/pdf"
    ["zip"]="application/zip"
    ["tar"]="application/x-tar"
    ["gz"]="application/gzip"
)

# 获取文件MIME类型
get_mime_type() {
    local file="$1"
    local ext="${file##*.}"
    ext="${ext,,}"  # 转换为小写
    
    echo "${MIME_TYPES[$ext]:-application/octet-stream}"
}

# 检查WebUI目录结构
check_webui_structure() {
    local module_dir="$1"
    local webroot="$module_dir/webroot"
    
    log_debug "检查WebUI结构: $webroot"
    
    if [ ! -d "$webroot" ]; then
        log_error "WebUI目录不存在: $webroot"
        return 1
    fi
    
    if [ ! -f "$webroot/$WEBUI_INDEX" ]; then
        log_error "缺少WebUI入口文件: $webroot/$WEBUI_INDEX"
        return 1
    fi
    
    # 检查文件权限
    if [ ! -r "$webroot/$WEBUI_INDEX" ]; then
        log_error "WebUI入口文件不可读: $webroot/$WEBUI_INDEX"
        return 1
    fi
    
    log_success "WebUI结构检查通过"
    return 0
}

# 生成WebUI配置
generate_webui_config() {
    local module_dir="$1"
    local port="${2:-$DEFAULT_WEBUI_PORT}"
    local webroot="$module_dir/webroot"
    local config_file="$module_dir/$WEBUI_CONFIG"
    
    log_info "生成WebUI配置: $config_file"
    
    cat > "$config_file" << EOF
# WebUI Configuration
PORT=$port
WEBROOT=$webroot
ENABLED=true
AUTH_REQUIRED=false
AUTH_USERNAME=admin
AUTH_PASSWORD=
MAX_CONNECTIONS=10
REQUEST_TIMEOUT=30
ENABLE_LOGGING=true
LOG_LEVEL=INFO
CORS_ENABLED=true
CORS_ORIGIN=*
CACHE_ENABLED=true
CACHE_MAX_AGE=3600
GZIP_ENABLED=true
SSL_ENABLED=false
SSL_CERT=
SSL_KEY=
CUSTOM_HEADERS=
EOF
    
    log_success "WebUI配置生成完成"
    return 0
}

# 读取WebUI配置
read_webui_config() {
    local module_dir="$1"
    local config_file="$module_dir/$WEBUI_CONFIG"
    local key="$2"
    local default="${3:-}"
    
    if [ -f "$config_file" ]; then
        read_config "$config_file" "$key" "$default"
    else
        echo "$default"
    fi
}

# 检查端口是否可用
is_port_available() {
    local port="$1"
    
    if command_exists netstat; then
        ! netstat -ln 2>/dev/null | grep -q ":$port "
    elif command_exists ss; then
        ! ss -ln 2>/dev/null | grep -q ":$port "
    else
        # 尝试绑定端口
        if command_exists nc; then
            ! nc -z localhost "$port" 2>/dev/null
        else
            log_warning "无法检查端口可用性：缺少网络工具"
            return 0
        fi
    fi
}

# 查找可用端口
find_available_port() {
    local start_port="${1:-8080}"
    local max_port="${2:-8999}"
    
    for port in $(seq "$start_port" "$max_port"); do
        if is_port_available "$port"; then
            echo "$port"
            return 0
        fi
    done
    
    log_error "在范围 $start_port-$max_port 内找不到可用端口"
    return 1
}

# 启动简单HTTP服务器
start_simple_http_server() {
    local webroot="$1"
    local port="$2"
    local log_file="${3:-$WEBUI_LOG_FILE}"
    
    log_info "启动HTTP服务器: $webroot:$port"
    
    # 检查Python可用性
    if command_exists python3; then
        cd "$webroot" && python3 -m http.server "$port" > "$log_file" 2>&1 &
        echo $! > "$WEBUI_PID_FILE"
        return 0
    elif command_exists python; then
        cd "$webroot" && python -m SimpleHTTPServer "$port" > "$log_file" 2>&1 &
        echo $! > "$WEBUI_PID_FILE"
        return 0
    elif command_exists busybox && busybox httpd --help >/dev/null 2>&1; then
        busybox httpd -p "$port" -h "$webroot" -f > "$log_file" 2>&1 &
        echo $! > "$WEBUI_PID_FILE"
        return 0
    else
        log_error "无法启动HTTP服务器：缺少支持的工具"
        return 1
    fi
}

# 生成自定义HTTP服务器脚本
generate_custom_http_server() {
    local webroot="$1"
    local port="$2"
    local server_script="/data/local/tmp/webui_server.sh"
    
    cat > "$server_script" << 'EOF'
#!/system/bin/sh
# Custom WebUI HTTP Server

WEBROOT="$1"
PORT="$2"
PID_FILE="$3"
LOG_FILE="$4"

# HTTP响应函数
send_response() {
    local status="$1"
    local content_type="$2"
    local content="$3"
    local content_length="${#content}"
    
    echo "HTTP/1.1 $status"
    echo "Content-Type: $content_type"
    echo "Content-Length: $content_length"
    echo "Connection: close"
    echo ""
    echo "$content"
}

# 处理HTTP请求
handle_request() {
    local request_line
    read -r request_line
    
    local method=$(echo "$request_line" | cut -d' ' -f1)
    local path=$(echo "$request_line" | cut -d' ' -f2)
    
    # 移除查询参数
    path="${path%\?*}"
    
    # 安全检查：防止目录遍历
    case "$path" in
        *..*)
            send_response "403 Forbidden" "text/plain" "Access denied"
            return
            ;;
    esac
    
    # 默认文件
    if [ "$path" = "/" ]; then
        path="/index.html"
    fi
    
    local file_path="$WEBROOT$path"
    
    if [ -f "$file_path" ] && [ -r "$file_path" ]; then
        local mime_type="text/html"
        case "${path##*.}" in
            css) mime_type="text/css" ;;
            js) mime_type="application/javascript" ;;
            json) mime_type="application/json" ;;
            png) mime_type="image/png" ;;
            jpg|jpeg) mime_type="image/jpeg" ;;
            gif) mime_type="image/gif" ;;
            svg) mime_type="image/svg+xml" ;;
        esac
        
        local content="$(cat "$file_path")"
        send_response "200 OK" "$mime_type" "$content"
    else
        send_response "404 Not Found" "text/html" "<h1>404 Not Found</h1>"
    fi
}

# 启动服务器
start_server() {
    echo "Starting WebUI server on port $PORT" >> "$LOG_FILE"
    echo $$ > "$PID_FILE"
    
    while true; do
        if command -v nc >/dev/null 2>&1; then
            nc -l -p "$PORT" -e handle_request
        elif command -v netcat >/dev/null 2>&1; then
            netcat -l -p "$PORT" -e handle_request
        else
            echo "No suitable network tool found" >> "$LOG_FILE"
            exit 1
        fi
    done
}

start_server
EOF
    
    chmod +x "$server_script"
    
    # 启动自定义服务器
    "$server_script" "$webroot" "$port" "$WEBUI_PID_FILE" "$WEBUI_LOG_FILE" &
    
    return 0
}

# 启动WebUI服务器
start_webui_server() {
    local module_dir="$1"
    local port="$2"
    
    if [ -z "$port" ]; then
        port=$(read_webui_config "$module_dir" "PORT" "$DEFAULT_WEBUI_PORT")
    fi
    
    local webroot="$module_dir/webroot"
    
    # 检查WebUI结构
    if ! check_webui_structure "$module_dir"; then
        return 1
    fi
    
    # 检查端口可用性
    if ! is_port_available "$port"; then
        log_warning "端口 $port 已被占用，查找可用端口..."
        port=$(find_available_port "$port")
        if [ $? -ne 0 ]; then
            return 1
        fi
        log_info "使用端口: $port"
    fi
    
    # 停止已存在的服务器
    stop_webui_server
    
    # 启动服务器
    if ! start_simple_http_server "$webroot" "$port" "$WEBUI_LOG_FILE"; then
        log_warning "简单HTTP服务器启动失败，尝试自定义服务器..."
        generate_custom_http_server "$webroot" "$port"
    fi
    
    # 等待服务器启动
    sleep 2
    
    # 检查服务器状态
    if is_webui_running; then
        log_success "WebUI服务器启动成功"
        log_info "访问地址: http://localhost:$port"
        
        # 更新配置中的端口
        write_config "$module_dir/$WEBUI_CONFIG" "PORT" "$port"
        
        return 0
    else
        log_error "WebUI服务器启动失败"
        return 1
    fi
}

# 停止WebUI服务器
stop_webui_server() {
    if [ -f "$WEBUI_PID_FILE" ]; then
        local pid=$(cat "$WEBUI_PID_FILE")
        if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
            log_info "停止WebUI服务器 (PID: $pid)"
            kill "$pid" 2>/dev/null
            sleep 1
            
            # 强制终止
            if kill -0 "$pid" 2>/dev/null; then
                kill -9 "$pid" 2>/dev/null
            fi
        fi
        rm -f "$WEBUI_PID_FILE"
    fi
    
    # 清理临时文件
    rm -f "/data/local/tmp/webui_server.sh"
    
    log_success "WebUI服务器已停止"
}

# 检查WebUI服务器是否运行
is_webui_running() {
    if [ -f "$WEBUI_PID_FILE" ]; then
        local pid=$(cat "$WEBUI_PID_FILE")
        [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null
    else
        return 1
    fi
}

# 重启WebUI服务器
restart_webui_server() {
    local module_dir="$1"
    local port="$2"
    
    log_info "重启WebUI服务器..."
    stop_webui_server
    sleep 1
    start_webui_server "$module_dir" "$port"
}

# 获取WebUI状态
get_webui_status() {
    local module_dir="$1"
    
    if is_webui_running; then
        local pid=$(cat "$WEBUI_PID_FILE")
        local port=$(read_webui_config "$module_dir" "PORT" "unknown")
        echo "running:$pid:$port"
    else
        echo "stopped"
    fi
}

# 获取WebUI访问URL
get_webui_url() {
    local module_dir="$1"
    local external="${2:-false}"
    
    local port=$(read_webui_config "$module_dir" "PORT" "$DEFAULT_WEBUI_PORT")
    
    if [ "$external" = "true" ]; then
        # 获取设备IP地址
        local ip=$(ip route get 8.8.8.8 2>/dev/null | grep -o 'src [0-9.]*' | cut -d' ' -f2)
        if [ -n "$ip" ]; then
            echo "http://$ip:$port"
        else
            echo "http://localhost:$port"
        fi
    else
        echo "http://localhost:$port"
    fi
}

# 测试WebUI连接
test_webui_connection() {
    local module_dir="$1"
    local timeout="${2:-5}"
    
    local url=$(get_webui_url "$module_dir")
    
    log_info "测试WebUI连接: $url"
    
    if command_exists curl; then
        if curl -s --connect-timeout "$timeout" "$url" >/dev/null 2>&1; then
            log_success "WebUI连接测试成功"
            return 0
        fi
    elif command_exists wget; then
        if wget -q --timeout="$timeout" -O /dev/null "$url" 2>/dev/null; then
            log_success "WebUI连接测试成功"
            return 0
        fi
    fi
    
    log_error "WebUI连接测试失败"
    return 1
}

# 设置WebUI
setup_webui() {
    local module_dir="$1"
    local port="${2:-$DEFAULT_WEBUI_PORT}"
    
    log_info "设置WebUI: $module_dir"
    
    # 检查WebUI结构
    if ! check_webui_structure "$module_dir"; then
        return 1
    fi
    
    # 生成配置
    generate_webui_config "$module_dir" "$port"
    
    # 设置权限
    set_permission_recursive "$module_dir/webroot" "root:root" "755" "644"
    
    log_success "WebUI设置完成"
    return 0
}
    
    log_success "WebUI已配置，端口: $port"
}

start_webui_server() {
    local module_dir="$1"
    local config_file="$module_dir/$WEBUI_CONFIG"
    
    if [[ ! -f "$config_file" ]]; then
        log_error "WebUI配置文件不存在: $config_file"
        return 1
    fi
    
    # 读取配置
    source "$config_file"
    
    if [[ "$ENABLED" != "true" ]]; then
        log_info "WebUI已禁用"
        return 0
    fi
    
    # 检查端口是否已被占用
    if netstat -ln | grep -q ":$PORT "; then
        log_warning "端口 $PORT 已被占用"
        return 1
    fi
    
    # 启动简单HTTP服务器
    if command -v python3 >/dev/null 2>&1; then
        cd "$WEBROOT"
        python3 -m http.server "$PORT" >/dev/null 2>&1 &
        echo $! > "$module_dir/webui.pid"
        log_success "WebUI服务器已启动，端口: $PORT"
    elif command -v busybox >/dev/null 2>&1; then
        # 使用BusyBox的httpd
        busybox httpd -p "$PORT" -h "$WEBROOT" -f &
        echo $! > "$module_dir/webui.pid"
        log_success "WebUI服务器已启动 (BusyBox)，端口: $PORT"
    else
        log_error "未找到合适的HTTP服务器"
        return 1
    fi
}

stop_webui_server() {
    local module_dir="$1"
    local pid_file="$module_dir/webui.pid"
    
    if [[ -f "$pid_file" ]]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid"
            rm -f "$pid_file"
            log_success "WebUI服务器已停止"
        else
            log_info "WebUI服务器未运行"
            rm -f "$pid_file"
        fi
    else
        log_info "未找到WebUI服务器PID文件"
    fi
}

is_webui_running() {
    local module_dir="$1"
    local pid_file="$module_dir/webui.pid"
    
    if [[ -f "$pid_file" ]]; then
        local pid=$(cat "$pid_file")
        kill -0 "$pid" 2>/dev/null
    else
        return 1
    fi
}

get_webui_url() {
    local module_dir="$1"
    local config_file="$module_dir/$WEBUI_CONFIG"
    
    if [[ -f "$config_file" ]]; then
        source "$config_file"
        echo "http://localhost:$PORT"
    fi
}

# WebUI模板生成
generate_webui_template() {
    local webroot="$1"
    local module_name="$2"
    local module_id="$3"
    
    mkdir -p "$webroot"
    
    # 生成基础HTML模板
    cat > "$webroot/$WEBUI_INDEX" << 'EOF'
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>MODULE_NAME - WebUI</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #333;
            min-height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
        }
        
        .container {
            background: rgba(255, 255, 255, 0.95);
            border-radius: 12px;
            padding: 2rem;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
            backdrop-filter: blur(10px);
            max-width: 600px;
            width: 90%;
        }
        
        .header {
            text-align: center;
            margin-bottom: 2rem;
        }
        
        .header h1 {
            color: #2c3e50;
            font-size: 2rem;
            margin-bottom: 0.5rem;
        }
        
        .header p {
            color: #7f8c8d;
            font-size: 1rem;
        }
        
        .status {
            background: #e8f5e8;
            border: 1px solid #27ae60;
            border-radius: 8px;
            padding: 1rem;
            margin-bottom: 2rem;
            text-align: center;
        }
        
        .status.online {
            background: #e8f5e8;
            border-color: #27ae60;
            color: #27ae60;
        }
        
        .controls {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 1rem;
            margin-bottom: 2rem;
        }
        
        .btn {
            background: #3498db;
            color: white;
            border: none;
            border-radius: 8px;
            padding: 0.75rem 1rem;
            cursor: pointer;
            font-size: 0.9rem;
            transition: all 0.3s ease;
        }
        
        .btn:hover {
            background: #2980b9;
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(52, 152, 219, 0.3);
        }
        
        .btn.danger {
            background: #e74c3c;
        }
        
        .btn.danger:hover {
            background: #c0392b;
        }
        
        .info {
            background: #f8f9fa;
            border-radius: 8px;
            padding: 1rem;
        }
        
        .info h3 {
            color: #2c3e50;
            margin-bottom: 0.5rem;
        }
        
        .info-item {
            display: flex;
            justify-content: space-between;
            padding: 0.25rem 0;
            border-bottom: 1px solid #ecf0f1;
        }
        
        .info-item:last-child {
            border-bottom: none;
        }
        
        .footer {
            text-align: center;
            margin-top: 2rem;
            color: #7f8c8d;
            font-size: 0.8rem;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>MODULE_NAME</h1>
            <p>KernelSU 模块管理界面</p>
        </div>
        
        <div class="status online">
            <strong>模块状态: 运行中</strong>
        </div>
        
        <div class="controls">
            <button class="btn" onclick="reloadModule()">重载模块</button>
            <button class="btn" onclick="viewLogs()">查看日志</button>
            <button class="btn" onclick="updateModule()">检查更新</button>
            <button class="btn danger" onclick="restartModule()">重启模块</button>
        </div>
        
        <div class="info">
            <h3>模块信息</h3>
            <div class="info-item">
                <span>模块ID:</span>
                <span id="moduleId">MODULE_ID</span>
            </div>
            <div class="info-item">
                <span>版本:</span>
                <span id="moduleVersion">v1.0.0</span>
            </div>
            <div class="info-item">
                <span>作者:</span>
                <span id="moduleAuthor">Unknown</span>
            </div>
            <div class="info-item">
                <span>安装时间:</span>
                <span id="installTime">-</span>
            </div>
        </div>
        
        <div class="footer">
            <p>Powered by KernelSU Module Framework</p>
        </div>
    </div>
    
    <script>
        // 基础JavaScript功能
        function reloadModule() {
            if (confirm('确定要重载模块吗？')) {
                fetch('/api/reload', { method: 'POST' })
                    .then(response => response.json())
                    .then(data => {
                        alert(data.message || '模块已重载');
                    })
                    .catch(error => {
                        alert('操作失败: ' + error.message);
                    });
            }
        }
        
        function viewLogs() {
            window.open('/logs.html', '_blank');
        }
        
        function updateModule() {
            fetch('/api/update-check')
                .then(response => response.json())
                .then(data => {
                    if (data.hasUpdate) {
                        if (confirm('发现新版本: ' + data.version + '，是否更新？')) {
                            fetch('/api/update', { method: 'POST' })
                                .then(response => response.json())
                                .then(result => {
                                    alert(result.message || '更新完成');
                                });
                        }
                    } else {
                        alert('当前已是最新版本');
                    }
                })
                .catch(error => {
                    alert('检查更新失败: ' + error.message);
                });
        }
        
        function restartModule() {
            if (confirm('确定要重启模块吗？这可能需要重启设备。')) {
                fetch('/api/restart', { method: 'POST' })
                    .then(response => response.json())
                    .then(data => {
                        alert(data.message || '模块重启中...');
                    })
                    .catch(error => {
                        alert('操作失败: ' + error.message);
                    });
            }
        }
        
        // 加载模块信息
        function loadModuleInfo() {
            fetch('/api/info')
                .then(response => response.json())
                .then(data => {
                    document.getElementById('moduleId').textContent = data.id || 'MODULE_ID';
                    document.getElementById('moduleVersion').textContent = data.version || 'v1.0.0';
                    document.getElementById('moduleAuthor').textContent = data.author || 'Unknown';
                    document.getElementById('installTime').textContent = data.installTime || '-';
                })
                .catch(error => {
                    console.error('加载模块信息失败:', error);
                });
        }
        
        // 页面加载完成后初始化
        document.addEventListener('DOMContentLoaded', function() {
            loadModuleInfo();
            
            // 每30秒检查一次状态
            setInterval(function() {
                fetch('/api/status')
                    .then(response => response.json())
                    .then(data => {
                        const statusEl = document.querySelector('.status');
                        if (data.running) {
                            statusEl.className = 'status online';
                            statusEl.innerHTML = '<strong>模块状态: 运行中</strong>';
                        } else {
                            statusEl.className = 'status offline';
                            statusEl.innerHTML = '<strong>模块状态: 已停止</strong>';
                        }
                    })
                    .catch(error => {
                        console.error('状态检查失败:', error);
                    });
            }, 30000);
        });
    </script>
</body>
</html>
EOF
    
    # 替换模板变量
    sed -i "s/MODULE_NAME/$module_name/g" "$webroot/$WEBUI_INDEX"
    sed -i "s/MODULE_ID/$module_id/g" "$webroot/$WEBUI_INDEX"
    
    # 生成日志页面
    cat > "$webroot/logs.html" << 'EOF'
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>模块日志</title>
    <style>
        body {
            font-family: 'Courier New', monospace;
            background: #1e1e1e;
            color: #d4d4d4;
            margin: 0;
            padding: 1rem;
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
        }
        
        .header {
            background: #2d2d30;
            padding: 1rem;
            border-radius: 8px 8px 0 0;
            border-bottom: 1px solid #3e3e42;
        }
        
        .log-content {
            background: #252526;
            border-radius: 0 0 8px 8px;
            padding: 1rem;
            height: 70vh;
            overflow-y: auto;
            white-space: pre-wrap;
            font-size: 0.9rem;
            line-height: 1.4;
        }
        
        .controls {
            margin-bottom: 1rem;
        }
        
        .btn {
            background: #0e639c;
            color: white;
            border: none;
            padding: 0.5rem 1rem;
            border-radius: 4px;
            cursor: pointer;
            margin-right: 0.5rem;
        }
        
        .btn:hover {
            background: #1177bb;
        }
        
        .log-line {
            margin-bottom: 0.25rem;
        }
        
        .log-error {
            color: #f48771;
        }
        
        .log-warning {
            color: #dcdcaa;
        }
        
        .log-info {
            color: #9cdcfe;
        }
        
        .log-success {
            color: #4ec9b0;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="controls">
            <button class="btn" onclick="refreshLogs()">刷新日志</button>
            <button class="btn" onclick="clearLogs()">清空日志</button>
            <button class="btn" onclick="downloadLogs()">下载日志</button>
            <button class="btn" onclick="window.close()">关闭</button>
        </div>
        
        <div class="header">
            <h2>模块运行日志</h2>
        </div>
        
        <div class="log-content" id="logContent">
            正在加载日志...
        </div>
    </div>
    
    <script>
        function refreshLogs() {
            fetch('/api/logs')
                .then(response => response.text())
                .then(data => {
                    const logContent = document.getElementById('logContent');
                    logContent.innerHTML = formatLogs(data);
                    logContent.scrollTop = logContent.scrollHeight;
                })
                .catch(error => {
                    document.getElementById('logContent').textContent = '加载日志失败: ' + error.message;
                });
        }
        
        function formatLogs(logs) {
            return logs.split('\n').map(line => {
                let className = 'log-line';
                if (line.includes('[ERROR]')) className += ' log-error';
                else if (line.includes('[WARNING]')) className += ' log-warning';
                else if (line.includes('[INFO]')) className += ' log-info';
                else if (line.includes('[SUCCESS]')) className += ' log-success';
                
                return `<div class="${className}">${escapeHtml(line)}</div>`;
            }).join('');
        }
        
        function escapeHtml(unsafe) {
            return unsafe
                .replace(/&/g, "&amp;")
                .replace(/</g, "&lt;")
                .replace(/>/g, "&gt;")
                .replace(/"/g, "&quot;")
                .replace(/'/g, "&#039;");
        }
        
        function clearLogs() {
            if (confirm('确定要清空日志吗？')) {
                fetch('/api/logs', { method: 'DELETE' })
                    .then(() => {
                        document.getElementById('logContent').innerHTML = '';
                    });
            }
        }
        
        function downloadLogs() {
            fetch('/api/logs')
                .then(response => response.text())
                .then(data => {
                    const blob = new Blob([data], { type: 'text/plain' });
                    const url = window.URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.href = url;
                    a.download = 'module-logs.txt';
                    a.click();
                    window.URL.revokeObjectURL(url);
                });
        }
        
        // 初始化
        document.addEventListener('DOMContentLoaded', function() {
            refreshLogs();
            
            // 自动刷新日志
            setInterval(refreshLogs, 5000);
        });
    </script>
</body>
</html>
EOF
    
    log_success "WebUI模板已生成: $webroot"
}

# API处理函数
handle_webui_api() {
    local request_uri="$1"
    local request_method="$2"
    local module_dir="$3"
    
    case "$request_uri" in
        "/api/info")
            cat "$module_dir/module.prop" | while IFS='=' read -r key value; do
                echo "\"$key\": \"$value\","
            done | sed '$ s/,$//' | sed '1i{' | sed '$a}'
            ;;
        "/api/status")
            if is_service_running "MODULE_SERVICE"; then
                echo '{"running": true}'
            else
                echo '{"running": false}'
            fi
            ;;
        "/api/logs")
            if [[ "$request_method" == "DELETE" ]]; then
                echo "" > "$module_dir/module.log"
                echo '{"message": "日志已清空"}'
            else
                cat "$module_dir/module.log" 2>/dev/null || echo "暂无日志"
            fi
            ;;
        "/api/reload")
            restart_service "MODULE_SERVICE"
            echo '{"message": "模块已重载"}'
            ;;
        *)
            echo '{"error": "API not found"}'
            ;;
    esac
}

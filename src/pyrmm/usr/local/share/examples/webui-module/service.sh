#!/system/bin/sh
#
# WebUI示例模块服务脚本
# WebUI Example Module Service Script
#

# 模块配置
MODULE_NAME="WebUI Example"
MODULE_DIR="/data/adb/modules/webui_example"
WEBUI_DIR="$MODULE_DIR/webui"
WEBUI_PORT=8080
WEBUI_HOST="0.0.0.0"

# 加载WebUI辅助库
source /usr/lib/webui-helpers.sh

# 日志函数
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $MODULE_NAME: $*" >> /data/adb/modules_log
}

log "Starting $MODULE_NAME service"

# 初始化WebUI
init_webui() {
    log "Initializing WebUI..."
    
    # 创建WebUI目录
    mkdir -p "$WEBUI_DIR"
    
    # 创建配置文件
    cat > "$MODULE_DIR/webui.conf" << EOF
# WebUI配置文件
WEBUI_ENABLED=true
WEBUI_PORT=$WEBUI_PORT
WEBUI_HOST=$WEBUI_HOST
WEBUI_AUTH=false
DEBUG_MODE=false
EOF
    
    # 复制WebUI文件
    setup_webui_files
    
    log "WebUI initialized"
}

# 设置WebUI文件
setup_webui_files() {
    # 创建主页面
    cat > "$WEBUI_DIR/index.html" << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>WebUI Example Module</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div class="container">
        <header>
            <h1>WebUI Example Module</h1>
            <p>KernelSU Module with Web Interface</p>
        </header>
        
        <nav>
            <button onclick="showTab('status')" class="tab-button active">Status</button>
            <button onclick="showTab('settings')" class="tab-button">Settings</button>
            <button onclick="showTab('logs')" class="tab-button">Logs</button>
            <button onclick="showTab('about')" class="tab-button">About</button>
        </nav>
        
        <main>
            <div id="status" class="tab-content active">
                <h2>Module Status</h2>
                <div class="status-grid">
                    <div class="status-item">
                        <span class="status-label">Module State:</span>
                        <span id="module-state" class="status-value">Loading...</span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">WebUI Port:</span>
                        <span class="status-value">8080</span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">Last Update:</span>
                        <span id="last-update" class="status-value">-</span>
                    </div>
                </div>
                
                <div class="actions">
                    <button onclick="refreshStatus()" class="btn btn-primary">Refresh</button>
                    <button onclick="restartModule()" class="btn btn-warning">Restart</button>
                </div>
            </div>
            
            <div id="settings" class="tab-content">
                <h2>Module Settings</h2>
                <form id="settings-form">
                    <div class="form-group">
                        <label for="webui-enabled">Enable WebUI</label>
                        <input type="checkbox" id="webui-enabled" checked>
                    </div>
                    
                    <div class="form-group">
                        <label for="webui-port">WebUI Port</label>
                        <input type="number" id="webui-port" value="8080" min="1024" max="65535">
                    </div>
                    
                    <div class="form-group">
                        <label for="debug-mode">Debug Mode</label>
                        <input type="checkbox" id="debug-mode">
                    </div>
                    
                    <div class="form-actions">
                        <button type="submit" class="btn btn-primary">Save Settings</button>
                        <button type="button" onclick="resetSettings()" class="btn btn-secondary">Reset</button>
                    </div>
                </form>
            </div>
            
            <div id="logs" class="tab-content">
                <h2>Module Logs</h2>
                <div class="log-controls">
                    <button onclick="refreshLogs()" class="btn btn-primary">Refresh</button>
                    <button onclick="clearLogs()" class="btn btn-warning">Clear</button>
                </div>
                <pre id="log-content" class="log-display">Loading logs...</pre>
            </div>
            
            <div id="about" class="tab-content">
                <h2>About</h2>
                <div class="about-info">
                    <p><strong>Module Name:</strong> WebUI Example Module</p>
                    <p><strong>Version:</strong> 1.0.0</p>
                    <p><strong>Author:</strong> KernelSU Team</p>
                    <p><strong>Description:</strong> An example module demonstrating WebUI integration.</p>
                </div>
            </div>
        </main>
    </div>
    
    <script src="script.js"></script>
</body>
</html>
EOF
    
    # 创建CSS样式
    cat > "$WEBUI_DIR/style.css" << 'EOF'
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background-color: #f5f5f5;
    color: #333;
    line-height: 1.6;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
}

header {
    text-align: center;
    margin-bottom: 30px;
    padding: 20px;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    border-radius: 10px;
}

header h1 {
    font-size: 2.5em;
    margin-bottom: 10px;
}

nav {
    display: flex;
    justify-content: center;
    margin-bottom: 30px;
    background: white;
    border-radius: 10px;
    padding: 10px;
    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
}

.tab-button {
    padding: 12px 24px;
    border: none;
    background: transparent;
    cursor: pointer;
    border-radius: 6px;
    margin: 0 5px;
    transition: all 0.3s;
}

.tab-button:hover {
    background-color: #f0f0f0;
}

.tab-button.active {
    background-color: #667eea;
    color: white;
}

.tab-content {
    display: none;
    background: white;
    padding: 30px;
    border-radius: 10px;
    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
}

.tab-content.active {
    display: block;
}

.status-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 20px;
    margin-bottom: 30px;
}

.status-item {
    display: flex;
    justify-content: space-between;
    padding: 15px;
    background-color: #f8f9fa;
    border-radius: 6px;
    border-left: 4px solid #667eea;
}

.status-label {
    font-weight: 600;
}

.status-value {
    color: #667eea;
    font-weight: 500;
}

.form-group {
    margin-bottom: 20px;
}

.form-group label {
    display: block;
    margin-bottom: 5px;
    font-weight: 600;
}

.form-group input {
    width: 100%;
    padding: 10px;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 16px;
}

.form-group input[type="checkbox"] {
    width: auto;
    margin-right: 10px;
}

.form-actions, .actions, .log-controls {
    margin-top: 20px;
}

.btn {
    padding: 12px 24px;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 16px;
    margin-right: 10px;
    transition: all 0.3s;
}

.btn-primary {
    background-color: #667eea;
    color: white;
}

.btn-primary:hover {
    background-color: #5567d3;
}

.btn-secondary {
    background-color: #6c757d;
    color: white;
}

.btn-secondary:hover {
    background-color: #5a6268;
}

.btn-warning {
    background-color: #ffc107;
    color: #212529;
}

.btn-warning:hover {
    background-color: #e0a800;
}

.log-display {
    background-color: #f8f9fa;
    border: 1px solid #e9ecef;
    border-radius: 6px;
    padding: 20px;
    max-height: 400px;
    overflow-y: auto;
    font-family: 'Courier New', monospace;
    font-size: 14px;
    white-space: pre-wrap;
}

.about-info p {
    margin-bottom: 10px;
    padding: 10px;
    background-color: #f8f9fa;
    border-radius: 6px;
}

@media (max-width: 768px) {
    .container {
        padding: 10px;
    }
    
    nav {
        flex-wrap: wrap;
    }
    
    .tab-button {
        flex: 1;
        margin: 2px;
    }
    
    .status-grid {
        grid-template-columns: 1fr;
    }
}
EOF
    
    # 创建JavaScript脚本
    cat > "$WEBUI_DIR/script.js" << 'EOF'
// WebUI JavaScript功能
class ModuleWebUI {
    constructor() {
        this.init();
    }
    
    init() {
        this.loadSettings();
        this.refreshStatus();
        this.setupEventListeners();
        
        // 定期刷新状态
        setInterval(() => this.refreshStatus(), 30000);
    }
    
    setupEventListeners() {
        // 设置表单提交
        document.getElementById('settings-form').addEventListener('submit', (e) => {
            e.preventDefault();
            this.saveSettings();
        });
    }
    
    async refreshStatus() {
        try {
            const response = await fetch('/api/status');
            const data = await response.json();
            
            document.getElementById('module-state').textContent = data.state || 'Active';
            document.getElementById('last-update').textContent = new Date().toLocaleString();
        } catch (error) {
            console.error('Failed to refresh status:', error);
            document.getElementById('module-state').textContent = 'Error';
        }
    }
    
    async loadSettings() {
        try {
            const response = await fetch('/api/settings');
            const settings = await response.json();
            
            document.getElementById('webui-enabled').checked = settings.webui_enabled !== false;
            document.getElementById('webui-port').value = settings.webui_port || 8080;
            document.getElementById('debug-mode').checked = settings.debug_mode === true;
        } catch (error) {
            console.error('Failed to load settings:', error);
        }
    }
    
    async saveSettings() {
        const settings = {
            webui_enabled: document.getElementById('webui-enabled').checked,
            webui_port: parseInt(document.getElementById('webui-port').value),
            debug_mode: document.getElementById('debug-mode').checked
        };
        
        try {
            const response = await fetch('/api/settings', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(settings)
            });
            
            if (response.ok) {
                alert('Settings saved successfully!');
            } else {
                alert('Failed to save settings');
            }
        } catch (error) {
            console.error('Failed to save settings:', error);
            alert('Failed to save settings');
        }
    }
    
    resetSettings() {
        document.getElementById('webui-enabled').checked = true;
        document.getElementById('webui-port').value = 8080;
        document.getElementById('debug-mode').checked = false;
    }
    
    async refreshLogs() {
        try {
            const response = await fetch('/api/logs');
            const logs = await response.text();
            document.getElementById('log-content').textContent = logs;
        } catch (error) {
            console.error('Failed to refresh logs:', error);
            document.getElementById('log-content').textContent = 'Failed to load logs';
        }
    }
    
    async clearLogs() {
        if (confirm('Are you sure you want to clear all logs?')) {
            try {
                await fetch('/api/logs', { method: 'DELETE' });
                document.getElementById('log-content').textContent = 'Logs cleared';
            } catch (error) {
                console.error('Failed to clear logs:', error);
            }
        }
    }
    
    async restartModule() {
        if (confirm('Are you sure you want to restart the module?')) {
            try {
                await fetch('/api/restart', { method: 'POST' });
                alert('Module restart initiated');
                setTimeout(() => this.refreshStatus(), 2000);
            } catch (error) {
                console.error('Failed to restart module:', error);
            }
        }
    }
}

// 标签页功能
function showTab(tabId) {
    // 隐藏所有标签页内容
    document.querySelectorAll('.tab-content').forEach(tab => {
        tab.classList.remove('active');
    });
    
    // 移除所有按钮的活动状态
    document.querySelectorAll('.tab-button').forEach(btn => {
        btn.classList.remove('active');
    });
    
    // 显示选中的标签页
    document.getElementById(tabId).classList.add('active');
    event.target.classList.add('active');
    
    // 特殊处理
    if (tabId === 'logs') {
        webui.refreshLogs();
    }
}

// 全局函数
function refreshStatus() { webui.refreshStatus(); }
function refreshLogs() { webui.refreshLogs(); }
function clearLogs() { webui.clearLogs(); }
function restartModule() { webui.restartModule(); }
function resetSettings() { webui.resetSettings(); }

// 初始化WebUI
const webui = new ModuleWebUI();
EOF
}

# 启动WebUI服务器
start_webui_server() {
    log "Starting WebUI server on port $WEBUI_PORT..."
    
    # 创建简单的HTTP服务器脚本
    cat > "$MODULE_DIR/webui_server.sh" << 'EOF'
#!/system/bin/sh
# 简单的HTTP服务器实现

WEBUI_DIR="/data/adb/modules/webui_example/webui"
WEBUI_PORT=8080

# 启动Python HTTP服务器（如果可用）
if command -v python3 >/dev/null 2>&1; then
    cd "$WEBUI_DIR"
    python3 -m http.server "$WEBUI_PORT" &
    echo $! > /data/adb/modules/webui_example/webui_server.pid
elif command -v python >/dev/null 2>&1; then
    cd "$WEBUI_DIR"
    python -m SimpleHTTPServer "$WEBUI_PORT" &
    echo $! > /data/adb/modules/webui_example/webui_server.pid
else
    echo "Error: Python not available for WebUI server"
    exit 1
fi
EOF
    
    chmod 755 "$MODULE_DIR/webui_server.sh"
    "$MODULE_DIR/webui_server.sh"
    
    log "WebUI server started on http://localhost:$WEBUI_PORT"
}

# 主函数
main() {
    # 初始化WebUI
    init_webui
    
    # 启动WebUI服务器
    start_webui_server
    
    # 创建状态文件
    touch "$MODULE_DIR/webui_active"
    
    log "$MODULE_NAME loaded successfully with WebUI"
}

# 执行主函数
main "$@"

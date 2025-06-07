#!/system/bin/sh

# Service Module Example - Customization Script

# Variables
MODDIR=${0%/*}
MODID="service_example"

# Load common functions
source "$MODDIR/lib/common-functions.sh" 2>/dev/null || {
    echo "Error: Failed to load common functions"
    exit 1
}

log_info "Installing Service Module Example"

# Create necessary directories
mkdir -p "$MODDIR/lib"
mkdir -p "$MODDIR/logs"
mkdir -p "/data/local/tmp"

# Copy library files if they exist
if [ -d "/data/adb/modules/pyrmm/usr/lib" ]; then
    log_info "Copying common library files"
    cp -f "/data/adb/modules/pyrmm/usr/lib/common-functions.sh" "$MODDIR/lib/" 2>/dev/null
    cp -f "/data/adb/modules/pyrmm/usr/lib/module-manager.sh" "$MODDIR/lib/" 2>/dev/null
fi

# Set appropriate permissions
log_info "Setting file permissions"
set_perm_recursive "$MODDIR" 0 0 0755 0644
set_perm "$MODDIR/service.sh" 0 0 0755

# Create configuration file
log_info "Creating service configuration"
cat > "$MODDIR/config.ini" << 'EOF'
[service]
# Service configuration
enabled=true
log_level=info
interval=30
auto_restart=true
priority=normal

[monitoring]
# System monitoring settings
check_memory=true
check_network=true
check_storage=true
alert_threshold=90

[cleanup]
# Cleanup settings
auto_cleanup=true
cleanup_interval=100
log_retention_days=7
cache_cleanup=true

[notifications]
# Notification settings
enable_notifications=false
notification_level=warn
toast_messages=false
EOF

# Set configuration permissions
set_perm "$MODDIR/config.ini" 0 0 0644

# Create a simple control script
log_info "Creating service control script"
cat > "$MODDIR/control.sh" << 'EOF'
#!/system/bin/sh

# Service Control Script
MODDIR=${0%/*}
SERVICE_NAME="example_service"
PID_FILE="/data/local/tmp/$SERVICE_NAME.pid"

case "$1" in
    start)
        if [ -f "$PID_FILE" ]; then
            echo "Service is already running (PID: $(cat $PID_FILE))"
        else
            echo "Starting $SERVICE_NAME..."
            nohup "$MODDIR/service.sh" > /dev/null 2>&1 &
        fi
        ;;
    stop)
        if [ -f "$PID_FILE" ]; then
            local pid=$(cat "$PID_FILE")
            echo "Stopping $SERVICE_NAME (PID: $pid)..."
            kill "$pid" 2>/dev/null
            rm -f "$PID_FILE"
        else
            echo "Service is not running"
        fi
        ;;
    status)
        if [ -f "$PID_FILE" ]; then
            local pid=$(cat "$PID_FILE")
            if kill -0 "$pid" 2>/dev/null; then
                echo "Service is running (PID: $pid)"
            else
                echo "Service is not running (stale PID file)"
                rm -f "$PID_FILE"
            fi
        else
            echo "Service is not running"
        fi
        ;;
    restart)
        $0 stop
        sleep 2
        $0 start
        ;;
    logs)
        tail -f "/data/local/tmp/$SERVICE_NAME.log" 2>/dev/null || echo "No log file found"
        ;;
    *)
        echo "Usage: $0 {start|stop|status|restart|logs}"
        exit 1
        ;;
esac
EOF

# Set control script permissions
set_perm "$MODDIR/control.sh" 0 0 0755

# Test service functionality
log_info "Testing service components"

# Test configuration reading
if command -v config-manager >/dev/null 2>&1; then
    local test_value=$(config-manager get service.enabled "$MODDIR/config.ini" 2>/dev/null)
    if [ "$test_value" = "true" ]; then
        log_info "Configuration test: PASSED"
    else
        log_warn "Configuration test: FAILED"
    fi
else
    log_warn "config-manager not available, manual configuration required"
fi

# Install completion message
log_info "Service Module Example installation completed"
log_info "Use '$MODDIR/control.sh' to manage the service"
log_info "Service will start automatically on next boot"

# Provide usage instructions
cat << 'EOF'

=== Service Module Example ===

This module demonstrates how to create a background service that:
- Runs continuously in the background
- Monitors system resources
- Performs periodic maintenance tasks
- Provides logging and configuration management
- Can be controlled via command line

Control Commands:
- Start service: ./control.sh start
- Stop service: ./control.sh stop
- Check status: ./control.sh status
- View logs: ./control.sh logs
- Restart: ./control.sh restart

Configuration file: config.ini
Log files: /data/local/tmp/example_service.log

EOF

#!/system/bin/sh

# Service Module Example - Service Script
# This script runs as a background service

# Load module library
MODDIR=${0%/*}
source "$MODDIR/lib/common-functions.sh" 2>/dev/null || {
    echo "Failed to load common functions"
    exit 1
}

# Service configuration
SERVICE_NAME="example_service"
PID_FILE="/data/local/tmp/$SERVICE_NAME.pid"
LOG_FILE="/data/local/tmp/$SERVICE_NAME.log"

# Initialize logging
log_info "Starting $SERVICE_NAME"

# Function to cleanup on exit
cleanup() {
    log_info "Cleaning up $SERVICE_NAME"
    rm -f "$PID_FILE"
    exit 0
}

# Set up signal handlers
trap cleanup TERM INT

# Write PID file
echo $$ > "$PID_FILE"

# Main service loop
main_service() {
    local counter=0
    
    while true; do
        counter=$((counter + 1))
        
        # Example service work
        log_debug "Service heartbeat #$counter"
        
        # Check system status
        if check_kernelsu_environment; then
            log_info "KernelSU environment verified"
        else
            log_warn "KernelSU environment check failed"
        fi
        
        # Monitor system resources
        local meminfo=$(cat /proc/meminfo | grep MemAvailable | awk '{print $2}')
        local loadavg=$(cat /proc/loadavg | awk '{print $1}')
        
        log_debug "Available memory: ${meminfo}kB, Load average: $loadavg"
        
        # Example: Clean temporary files every 100 iterations
        if [ $((counter % 100)) -eq 0 ]; then
            log_info "Performing periodic cleanup (iteration $counter)"
            
            # Clean old log files
            find /data/local/tmp -name "*.log" -mtime +7 -delete 2>/dev/null
            
            # Clean temporary cache
            rm -rf /data/local/tmp/module_cache/* 2>/dev/null
        fi
        
        # Example: Check for updates every 1000 iterations
        if [ $((counter % 1000)) -eq 0 ]; then
            log_info "Checking for module updates (iteration $counter)"
            # Add update check logic here
        fi
        
        # Sleep for 30 seconds
        sleep 30
        
        # Check if we should exit
        if [ ! -f "$PID_FILE" ]; then
            log_info "PID file removed, exiting service"
            break
        fi
    done
}

# Function to monitor device state
monitor_device_state() {
    # Monitor screen state
    local screen_state=$(dumpsys power | grep "Display Power" | grep -o "state=[A-Z]*" | cut -d= -f2)
    log_debug "Screen state: $screen_state"
    
    # Monitor charging state
    local charging=$(dumpsys battery | grep "AC powered" | awk '{print $3}')
    log_debug "Charging: $charging"
    
    # Monitor network state
    if ping -c 1 8.8.8.8 >/dev/null 2>&1; then
        log_debug "Network connectivity: OK"
    else
        log_debug "Network connectivity: Failed"
    fi
}

# Function to handle configuration changes
handle_config_change() {
    local config_file="$MODDIR/config.ini"
    
    if [ -f "$config_file" ]; then
        log_info "Reloading configuration from $config_file"
        
        # Read configuration using config manager
        if command -v config-manager >/dev/null 2>&1; then
            local log_level=$(config-manager get service.log_level "$config_file" 2>/dev/null || echo "info")
            set_log_level "$log_level"
            
            local service_interval=$(config-manager get service.interval "$config_file" 2>/dev/null || echo "30")
            log_info "Service interval set to $service_interval seconds"
        fi
    fi
}

# Initialize service
log_info "Initializing $SERVICE_NAME (PID: $$)"

# Load configuration
handle_config_change

# Start monitoring in background
(
    while true; do
        monitor_device_state
        sleep 60
    done
) &

# Start main service
main_service

# Cleanup on exit
cleanup

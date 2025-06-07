# Service Module Example

This example demonstrates how to create a KernelSU module that runs background services. It showcases advanced module development techniques including service management, configuration handling, and system monitoring.

## Features

- **Background Service**: Runs continuously as a daemon process
- **System Monitoring**: Monitors memory, network, and system resources  
- **Configuration Management**: Uses INI-style configuration files
- **Logging System**: Comprehensive logging with different levels
- **Periodic Tasks**: Performs cleanup and maintenance automatically
- **Service Control**: Command-line interface for service management
- **Signal Handling**: Proper cleanup on termination
- **Resource Monitoring**: Tracks system performance metrics

## Files Structure

```
service-module/
├── module.prop          # Module metadata
├── customize.sh         # Installation script  
├── service.sh          # Main service daemon
├── control.sh          # Service control interface
├── config.ini          # Configuration file
└── README.md           # This file
```

## Installation

This module installs automatically when placed in the KernelSU modules directory. The installation process:

1. Creates necessary directories and files
2. Sets appropriate permissions
3. Copies library dependencies
4. Creates configuration files
5. Sets up service control scripts

## Usage

### Service Control

The module provides a control script for managing the service:

```bash
# Start the service
./control.sh start

# Stop the service  
./control.sh stop

# Check service status
./control.sh status

# Restart the service
./control.sh restart

# View live logs
./control.sh logs
```

### Configuration

Edit `config.ini` to customize service behavior:

```ini
[service]
enabled=true
log_level=info
interval=30
auto_restart=true

[monitoring]
check_memory=true
check_network=true
alert_threshold=90

[cleanup]
auto_cleanup=true
cleanup_interval=100
log_retention_days=7
```

### Monitoring

The service provides several monitoring features:

- **Memory Usage**: Tracks available memory
- **Load Average**: Monitors system load
- **Network Connectivity**: Tests internet connection
- **Screen State**: Monitors device screen status
- **Charging State**: Tracks battery charging status

### Periodic Tasks

The service automatically performs:

- **Log Cleanup**: Removes old log files (every 100 iterations)
- **Cache Cleanup**: Clears temporary cache files
- **Update Checks**: Checks for module updates (every 1000 iterations)
- **System Health**: Monitors overall system health

## Logging

The service uses structured logging with multiple levels:

- **DEBUG**: Detailed operational information
- **INFO**: General operational messages
- **WARN**: Warning conditions
- **ERROR**: Error conditions
- **FATAL**: Critical errors

Logs are written to `/data/local/tmp/example_service.log`

## Advanced Features

### Signal Handling

The service properly handles termination signals:
- SIGTERM: Graceful shutdown
- SIGINT: Interrupt handling
- Cleanup: Removes PID files and temporary data

### Configuration Reloading

The service can reload configuration without restart:
- Monitors configuration file changes
- Applies new settings dynamically
- Validates configuration syntax

### Resource Management

Efficient resource usage:
- Minimal CPU usage during idle
- Memory leak prevention
- Proper file handle management
- Automatic cleanup of temporary files

## Development Notes

This example demonstrates several important concepts:

1. **Service Architecture**: Proper daemon design patterns
2. **Error Handling**: Robust error recovery mechanisms
3. **Configuration Management**: Flexible configuration system
4. **Logging Best Practices**: Structured and configurable logging
5. **Process Management**: PID files and signal handling
6. **System Integration**: Interaction with Android/Linux subsystems

## Customization

To adapt this example for your own service:

1. Modify `service.sh` to implement your service logic
2. Update `config.ini` with your configuration options
3. Customize monitoring and cleanup tasks
4. Add your own logging and error handling
5. Update `module.prop` with your module information

## Dependencies

This module requires:
- KernelSU or compatible root solution
- Shell environment with standard utilities
- Common functions library (optional but recommended)
- Configuration manager tool (optional)

## Security Considerations

- Service runs with root privileges
- Proper input validation required
- Secure handling of sensitive data
- Regular security updates recommended

## Troubleshooting

### Service Won't Start

1. Check file permissions: `ls -la service.sh`
2. Verify KernelSU environment: Check logs
3. Test manually: `./service.sh` in terminal

### Configuration Issues

1. Validate INI syntax in `config.ini`
2. Check file permissions on configuration
3. Verify configuration manager availability

### Performance Issues

1. Monitor resource usage: Check logs
2. Adjust service interval in configuration
3. Review cleanup intervals

### Log Analysis

1. Check service logs: `./control.sh logs`
2. Review system logs: `logcat | grep service`
3. Monitor resource usage: `top | grep service`

## License

This example is provided for educational purposes. Adapt according to your project's license requirements.

## Support

For questions and support:
- Check the main project documentation
- Review the API reference
- Examine other example modules
- Consult the development community

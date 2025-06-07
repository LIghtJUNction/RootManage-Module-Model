# KernelSU Module Development Guide

## Introduction

This guide provides comprehensive information for developing KernelSU modules using the provided development tools.

## Getting Started

### Prerequisites
- Basic knowledge of shell scripting
- Understanding of Android system architecture
- Familiarity with KernelSU concepts

### Setting Up Development Environment
1. Install the development tools
2. Configure your environment
3. Set up testing infrastructure

```bash
# Install tools (if not already installed)
pip install pyrmm

# Initialize development environment
pyrmm usr init

# Verify installation
module-builder --help
```

## Module Architecture

### Core Components

#### module.prop
The module.prop file contains essential module information:

```properties
id=com.example.module
name=Example Module
version=v1.0.0
versionCode=1
author=Your Name
description=Example module description
minApi=21
maxApi=35
minMagisk=20000
minKernelSU=10000
```

#### Script Files
- `service.sh`: Runs during system boot
- `post-fs-data.sh`: Runs after /data is mounted
- `boot-completed.sh`: Runs after boot is completed
- `customize.sh`: Runs during module installation
- `uninstall.sh`: Runs during module removal

#### Directory Structure
```
module/
├── META-INF/
│   └── com/
│       └── google/
│           └── android/
│               ├── update-binary
│               └── updater-script
├── module.prop
├── service.sh (optional)
├── post-fs-data.sh (optional)
├── boot-completed.sh (optional)
├── customize.sh (optional)
├── uninstall.sh (optional)
├── system/ (for systemless mods)
├── webroot/ (for WebUI modules)
└── sepolicy.rule (optional)
```

## Development Workflow

### 1. Planning Your Module

Before starting development, consider:
- What functionality will your module provide?
- Does it need system file modifications?
- Will it require a web interface?
- What are the compatibility requirements?

### 2. Creating the Module Structure

Use the module creation tools:

```bash
# Interactive creation
ksm-create

# Or command-line creation
module-builder \
  --id com.yourname.module \
  --name "Your Module Name" \
  --author "Your Name" \
  --description "Module description" \
  --template basic \
  ./your-module
```

### 3. Implementing Functionality

#### Basic Module Example
```bash
#!/system/bin/sh
# service.sh

MODULE_ID="com.example.basic"
MODDIR="${0%/*}"

# Log function
log() {
    echo "[$MODULE_ID] $1" >> /data/local/tmp/kernelsu_modules.log
}

log "Module service started"

# Your module logic here
# ...

log "Module service completed"
```

#### SystemLess Modification Example
```bash
#!/system/bin/sh
# post-fs-data.sh

MODULE_ID="com.example.systemless"
MODDIR="${0%/*}"

# Mount point for system modifications
SYSTEM_MOUNT="$MODDIR/system"

# Create directories
mkdir -p "$SYSTEM_MOUNT/bin"
mkdir -p "$SYSTEM_MOUNT/lib"

# Copy custom files
cp "$MODDIR/files/custom_binary" "$SYSTEM_MOUNT/bin/"
chmod 755 "$SYSTEM_MOUNT/bin/custom_binary"

log "SystemLess modifications applied"
```

#### WebUI Module Example
```bash
#!/system/bin/sh
# service.sh

MODULE_ID="com.example.webui"
MODDIR="${0%/*}"

# Source WebUI helpers
source "$MODDIR/lib/webui-helpers.sh"

# Start WebUI server
if setup_webui "$MODDIR" 8080; then
    start_webui_server "$MODDIR" 8080
    log "WebUI started on port 8080"
else
    log "Failed to start WebUI"
fi
```

### 4. Testing Your Module

Use the testing tools to verify functionality:

```bash
# Basic structure test
ksm-test --test structure ./your-module

# Full test suite
ksm-test --all-tests ./your-module

# Validation
module-validator --strict ./your-module
```

### 5. Packaging for Distribution

Create a distributable package:

```bash
# Basic packaging
module-packager ./your-module

# Advanced packaging with compression and validation
module-packager \
  --validate \
  --compress \
  --format zip \
  --output your-module-v1.0.0.zip \
  ./your-module
```

## Advanced Topics

### SELinux Policy Rules

If your module needs custom SELinux permissions:

```
# sepolicy.rule
allow untrusted_app system_file:file { read execute };
```

### WebUI Development

#### HTML Structure
```html
<!DOCTYPE html>
<html>
<head>
    <title>${MODULE_NAME}</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>
<body>
    <div id="app">
        <h1>${MODULE_NAME}</h1>
        <p>Version: ${MODULE_VERSION}</p>
        <!-- Your UI here -->
    </div>
    
    <script>
        // Module JavaScript
        const moduleAPI = {
            getStatus: () => fetch('/api/status'),
            setConfig: (config) => fetch('/api/config', {
                method: 'POST',
                body: JSON.stringify(config)
            })
        };
    </script>
</body>
</html>
```

#### API Integration
```bash
# In service.sh - Simple API handler
handle_api_request() {
    local endpoint="$1"
    local method="$2"
    local data="$3"
    
    case "$endpoint" in
        "/api/status")
            echo '{"status":"running","version":"1.0.0"}'
            ;;
        "/api/config")
            # Handle configuration updates
            ;;
    esac
}
```

### Performance Optimization

#### Startup Time
- Minimize work in post-fs-data.sh
- Use background processes for heavy tasks
- Cache frequently used data

#### Memory Usage
- Clean up temporary files
- Use efficient data structures
- Monitor resource usage

#### Script Optimization
```bash
# Good: Use built-in commands
if [ -f "$file" ]; then
    # ...
fi

# Avoid: External command overhead
if test -f "$file"; then
    # ...
fi

# Good: Use parameter expansion
filename="${path##*/}"

# Avoid: External command
filename="$(basename "$path")"
```

### Security Best Practices

#### Input Validation
```bash
validate_input() {
    local input="$1"
    
    # Check for directory traversal
    case "$input" in
        *..*)
            log_error "Invalid input: directory traversal detected"
            return 1
            ;;
        */*)
            log_error "Invalid input: path separators not allowed"
            return 1
            ;;
    esac
    
    return 0
}
```

#### Secure File Operations
```bash
# Safe file creation
safe_write() {
    local file="$1"
    local content="$2"
    local temp_file="$file.tmp.$$"
    
    # Write to temporary file first
    echo "$content" > "$temp_file" || return 1
    
    # Atomic move
    mv "$temp_file" "$file" || {
        rm -f "$temp_file"
        return 1
    }
    
    return 0
}
```

#### Permission Management
```bash
# Set secure permissions
set_secure_permissions() {
    local path="$1"
    
    # Remove world write permissions
    chmod -R o-w "$path"
    
    # Set appropriate ownership
    chown -R root:root "$path"
    
    # Set executable permissions for scripts
    find "$path" -name "*.sh" -exec chmod 755 {} \;
}
```

## Debugging and Troubleshooting

### Debug Mode
Enable debug logging:

```bash
#!/system/bin/sh
# Enable debug mode
DEBUG=1
export DEBUG

# Use debug logging
source /path/to/common-functions.sh
log_debug "Debug information"
```

### Common Issues

#### Module Not Loading
1. Check module.prop syntax
2. Verify script permissions (755)
3. Check for shell script errors
4. Review KernelSU logs

#### WebUI Issues
1. Port conflicts
2. File permissions
3. Network connectivity
4. Browser compatibility

#### Performance Problems
1. Heavy operations in critical paths
2. Inefficient loops
3. External command overhead
4. Memory leaks

### Logging Best Practices
```bash
# Structured logging
log_with_context() {
    local level="$1"
    local component="$2"
    local message="$3"
    local timestamp="$(date '+%Y-%m-%d %H:%M:%S')"
    
    echo "[$timestamp] [$level] [$MODULE_ID:$component] $message" >> "$LOG_FILE"
}

# Usage
log_with_context "INFO" "webui" "Server started on port 8080"
log_with_context "ERROR" "config" "Failed to parse configuration file"
```

## Testing Strategies

### Unit Testing
```bash
# test_functions.sh
test_validate_input() {
    source ../common-functions.sh
    
    # Test valid input
    if validate_input "valid_string"; then
        echo "PASS: Valid input accepted"
    else
        echo "FAIL: Valid input rejected"
    fi
    
    # Test invalid input
    if ! validate_input "../malicious"; then
        echo "PASS: Invalid input rejected"
    else
        echo "FAIL: Invalid input accepted"
    fi
}

test_validate_input
```

### Integration Testing
```bash
# Test module installation
test_installation() {
    local test_module="./test_module"
    
    # Create test module
    module-builder --id test.module --name "Test" "$test_module"
    
    # Test packaging
    if module-packager "$test_module"; then
        echo "PASS: Module packaging successful"
    else
        echo "FAIL: Module packaging failed"
    fi
    
    # Cleanup
    rm -rf "$test_module"
}
```

## Deployment and Distribution

### Version Management
- Use semantic versioning (MAJOR.MINOR.PATCH)
- Update versionCode for each release
- Maintain changelog

### Release Process
1. Final testing
2. Version bump
3. Package creation
4. Documentation update
5. Release notes
6. Distribution

### Update Mechanism
```bash
# update_check.sh
check_for_updates() {
    local current_version="$1"
    local update_url="$2"
    
    if command_exists curl; then
        local latest_version="$(curl -s "$update_url/latest")"
        if version_compare "$current_version" "<" "$latest_version"; then
            log_info "Update available: $latest_version"
            return 0
        fi
    fi
    
    return 1
}
```

## Community and Contribution

### Sharing Your Module
- Open source when possible
- Provide clear documentation
- Include examples and tutorials
- Respond to user feedback

### Contributing to Tools
- Report bugs and issues
- Suggest improvements
- Contribute code
- Help with documentation

## Resources

### Documentation
- KernelSU official documentation
- Android system internals
- Shell scripting guides
- Web development resources

### Tools and Utilities
- Development environment setup
- Testing frameworks
- Debugging tools
- Performance profilers

### Community
- Forums and discussion boards
- Chat channels
- Code repositories
- Tutorial sites

---

*Happy module development! 🚀*

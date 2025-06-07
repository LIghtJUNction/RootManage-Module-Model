# KernelSU Module Development Tools Documentation

This directory contains documentation for the KernelSU module development tools.

## Overview

The KernelSU Module Development Tools provide a comprehensive toolkit for creating, testing, and managing KernelSU modules. The tools are designed to streamline the development process and ensure module quality and compatibility.

## Directory Structure

```
usr/
├── bin/                    # Executable tools
│   ├── ksm-build          # Build system integration
│   ├── ksm-create         # Module creation wizard
│   ├── ksm-test           # Module testing tool
│   ├── module-builder     # Advanced module builder
│   ├── module-packager    # Module packaging tool
│   └── module-validator   # Module validation tool
├── etc/                   # Configuration files
│   └── kernelsu-system.conf
├── include/               # Header files
│   ├── kernelsu-module.h
│   └── shell-utils.h
├── lib/                   # Function libraries
│   ├── common-functions.sh
│   ├── module-manager.sh
│   └── webui-helpers.sh
├── local/                 # Local override directory
└── share/                 # Shared data
    ├── doc/               # Documentation
    ├── man/               # Manual pages
    └── templates/         # Module templates
```

## Tools Overview

### Core Tools

#### ksm-create
Interactive module creation wizard that guides you through the process of creating a new KernelSU module.

```bash
ksm-create
ksm-create --template webui --id com.example.module --name "My Module"
```

#### module-builder
Advanced module building tool with extensive customization options.

```bash
module-builder -i com.example.module -n "Test Module" -a "Author" ./output
module-builder --template webui --webui -i com.example.ui -n "UI Module" ./ui-module
```

#### module-packager
Professional module packaging tool that creates distribution-ready packages.

```bash
module-packager ./my_module
module-packager -v -c ./my_module my_module.zip
```

#### module-validator
Comprehensive module validation tool that checks for common issues and best practices.

```bash
module-validator ./my_module
module-validator --strict --report ./my_module
```

#### ksm-test
Module testing framework for development and CI/CD environments.

```bash
ksm-test ./my_module
ksm-test --simulate --test structure ./my_module
```

### Template System

The template system provides pre-configured module structures for different use cases:

- **basic**: Simple module with minimal structure
- **systemless**: Module for system file replacement
- **webui**: Module with web interface
- **service**: Module with background services
- **replacement**: Module for file replacement
- **modification**: Module for system modification
- **addon**: Plugin module for other applications

### Library Functions

#### common-functions.sh
Core utility functions for module scripts:
- Environment detection
- Logging functions
- File operations
- Permission management
- Network utilities
- Configuration management

#### webui-helpers.sh
WebUI development utilities:
- HTTP server management
- Configuration handling
- Port management
- Security helpers

#### module-manager.sh
Module lifecycle management:
- Installation routines
- Update procedures
- Uninstallation cleanup
- Dependency management

## Configuration

### Global Configuration
System-wide configuration is stored in `/usr/etc/kernelsu-system.conf`:

```bash
# Default module template
DEFAULT_TEMPLATE=basic

# Default author for new modules
DEFAULT_AUTHOR=user

# Module repository settings
REPO_URL=https://modules.kernelsu.org
REPO_ENABLED=true

# Development settings
DEBUG_ENABLED=false
LOG_LEVEL=INFO
```

### Module Configuration
Each module can have its own configuration in `module.conf`:

```bash
# Module-specific settings
WEBUI_PORT=8080
AUTO_START=true
UPDATE_CHECK=true
```

## Best Practices

### Module Development
1. Always use meaningful module IDs following reverse domain notation
2. Include proper version information in module.prop
3. Implement proper error handling in scripts
4. Use the provided logging functions
5. Test modules thoroughly before distribution

### Security Considerations
1. Validate all user inputs
2. Use safe file operations
3. Set appropriate permissions
4. Avoid hardcoded credentials
5. Implement proper access controls for WebUI

### Performance Optimization
1. Minimize startup time impact
2. Use efficient scripting practices
3. Implement proper cleanup routines
4. Optimize WebUI resources
5. Use compression for large modules

## Examples

### Creating a Basic Module
```bash
# Interactive creation
ksm-create

# Command-line creation
module-builder \
  --id com.example.basic \
  --name "Basic Module" \
  --author "Your Name" \
  --description "A basic example module" \
  ./basic-module

# Test the module
ksm-test ./basic-module

# Package for distribution
module-packager --validate --compress ./basic-module
```

### Creating a WebUI Module
```bash
# Create module with WebUI
module-builder \
  --template webui \
  --webui \
  --service \
  --id com.example.webui \
  --name "WebUI Module" \
  ./webui-module

# The generated module will include:
# - WebUI interface (webroot/index.html)
# - Service script for HTTP server
# - Configuration files
# - Basic security setup
```

### Testing and Validation
```bash
# Comprehensive testing
ksm-test --all-tests --verbose ./my-module

# Validation with strict checks
module-validator --strict --security --report ./my-module

# Performance testing
ksm-test --test performance ./my-module
```

## Troubleshooting

### Common Issues

#### Module Not Loading
1. Check module.prop format
2. Verify file permissions
3. Check for syntax errors in scripts
4. Review system logs

#### WebUI Not Accessible
1. Check port availability
2. Verify firewall settings
3. Test network connectivity
4. Review WebUI server logs

#### Build Failures
1. Verify all required files are present
2. Check template compatibility
3. Validate configuration syntax
4. Review build logs for details

### Debug Mode
Enable debug mode for detailed logging:

```bash
export DEBUG=1
module-builder --debug [options]
```

### Log Files
Common log locations:
- System logs: `/data/local/tmp/kernelsu_modules.log`
- WebUI logs: `/data/local/tmp/webui.log`
- Build logs: `./build.log`

## Contributing

To contribute to the development tools:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

### Code Style
- Use consistent indentation (4 spaces)
- Add comments for complex logic
- Follow shell scripting best practices
- Include error handling
- Write descriptive commit messages

## License

These tools are provided under the same license as the main project. See LICENSE file for details.

## Support

For help and support:
- Check the documentation
- Search existing issues
- Create a new issue with detailed information
- Join the community discussions

---

*This documentation is maintained by the KernelSU development team and community contributors.*

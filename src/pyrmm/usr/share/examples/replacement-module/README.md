# System Replacement Module Example

This example demonstrates how to create a KernelSU module that replaces system files, applications, and resources. It showcases safe replacement practices, backup management, and rollback capabilities.

## Features

- **System File Replacement**: Replace any system file safely
- **Application Replacement**: Replace system apps with custom versions
- **Media Replacement**: Replace sounds, ringtones, and media files
- **Font Replacement**: Replace system fonts and typography
- **Binary Replacement**: Replace system binaries and executables
- **Automatic Backup**: Backup original files before replacement
- **Verification System**: Validate replacement files before installation
- **Rollback Support**: Restore original files when needed
- **Management Interface**: Command-line tools for replacement management

## Files Structure

```
replacement-module/
├── module.prop                 # Module metadata
├── customize.sh               # Installation script
├── manage_replacements.sh     # Replacement management tool
├── replacement_config.ini     # Configuration file
├── system/                    # System replacement files
│   ├── app/                  # Application replacements
│   ├── media/                # Media file replacements
│   ├── fonts/                # Font replacements
│   ├── bin/                  # Binary replacements
│   └── etc/                  # Configuration replacements
├── vendor/                    # Vendor replacement files
├── backup/                    # Original file backups
└── README.md                 # This file
```

## Installation

This module installs automatically when placed in the KernelSU modules directory. The installation process:

1. Creates replacement directory structure
2. Sets up backup system
3. Configures example replacements (placeholders)
4. Creates management tools
5. Sets appropriate permissions

## Usage

### Replacement Management

Use the management script to control replacements:

```bash
# List active replacements
./manage_replacements.sh list

# List available backups
./manage_replacements.sh backups

# Restore a file from backup
./manage_replacements.sh restore system/app/Calculator/Calculator.apk

# Validate replacement files
./manage_replacements.sh validate
```

### Adding Replacements

To add your own replacement files:

1. **Applications (APKs)**:
   ```bash
   # Place your custom APK
   cp MyCalculator.apk system/app/Calculator/Calculator.apk
   
   # Remove placeholder
   rm system/app/Calculator/placeholder.txt
   ```

2. **System Sounds**:
   ```bash
   # Place your custom sound files
   cp notification.ogg system/media/audio/notifications/
   cp ringtone.ogg system/media/audio/ringtones/
   ```

3. **Fonts**:
   ```bash
   # Place your custom fonts
   cp MyFont-Regular.ttf system/fonts/Roboto-Regular.ttf
   ```

4. **Binaries**:
   ```bash
   # Place your custom binaries
   cp mybusybox system/bin/busybox
   chmod 755 system/bin/busybox
   ```

### Configuration

Edit `replacement_config.ini` to customize behavior:

```ini
[general]
enabled=true
backup_originals=true
verify_before_replace=true
rollback_on_failure=true

[apps]
replace_calculator=true
verify_signatures=true

[media]
replace_sounds=true
audio_quality_check=true

[fonts]
replace_system_fonts=true
font_fallback_check=true
```

## Replacement Types

### Application Replacement

Replace system applications with custom versions:

- **Calculator**: Custom calculator app
- **Gallery**: Alternative gallery app
- **Browser**: Custom browser implementation
- **Keyboard**: Alternative input method

**Requirements**:
- Valid Android APK format
- Appropriate permissions
- Compatible API level
- Proper signatures (for system apps)

### Media File Replacement

Replace system media files:

- **Notification Sounds**: Custom notification tones
- **Ringtones**: Custom phone ringtones
- **UI Sounds**: Button clicks, keyboard sounds
- **Boot Animation**: Custom boot animation

**Supported Formats**:
- Audio: OGG, MP3, WAV, AAC
- Video: MP4, WebM
- Images: PNG, JPEG, WebP

### Font Replacement

Replace system fonts:

- **System Font**: Main UI font (Roboto)
- **Monospace**: Terminal and code font
- **Emoji Font**: Emoji and symbol font
- **Language Fonts**: Specific language fonts

**Requirements**:
- TrueType (.ttf) or OpenType (.otf) format
- Complete character sets
- Proper font metrics
- Unicode compliance

### Binary Replacement

Replace system binaries:

- **BusyBox**: Enhanced shell utilities
- **SQLite**: Database management tools
- **Network Tools**: wget, curl, etc.
- **System Utilities**: Custom system tools

**Requirements**:
- Compiled for Android
- Correct architecture (ARM64, ARM, x86_64, x86)
- Static linking or available dependencies
- Proper permissions (0755 for executables)

## Safety Features

### Automatic Backup

Before replacing any file, the module automatically:

1. Checks if original file exists
2. Creates backup in `backup/` directory
3. Verifies backup integrity
4. Logs backup location

### File Verification

All replacement files are verified:

- **APK Files**: ZIP structure validation
- **Audio Files**: Format and codec verification
- **Fonts**: Font file structure validation
- **Binaries**: ELF format and architecture check

### Rollback System

If problems occur:

1. **Manual Rollback**: Use management script
2. **Automatic Rollback**: On boot failure detection
3. **Selective Rollback**: Restore individual files
4. **Complete Rollback**: Restore all original files

## Advanced Features

### Module Integration

Integration with other modules:

```bash
# Check if replacement module is active
if [ -d "/data/adb/modules/replacement_example" ]; then
    echo "Replacement module detected"
fi

# Use replacement management API
source /data/adb/modules/replacement_example/lib/replacement-api.sh
create_replacement system/app/MyApp/MyApp.apk
```

### Conditional Replacement

Replace files based on conditions:

```bash
# Replace based on Android version
if [ "$API" -ge 30 ]; then
    # Use Android 11+ specific replacement
    cp modern_app.apk system/app/MyApp/MyApp.apk
else
    # Use legacy replacement
    cp legacy_app.apk system/app/MyApp/MyApp.apk
fi
```

### Signature Verification

For system apps requiring signatures:

```bash
# Check app signature
if verify_apk_signature system/app/MyApp/MyApp.apk; then
    echo "Signature valid"
else
    echo "Signature verification failed"
fi
```

## Best Practices

### File Preparation

1. **Test Thoroughly**: Test replacements on different devices
2. **Version Compatibility**: Ensure compatibility across Android versions
3. **Architecture Support**: Provide files for all architectures
4. **Quality Assurance**: Verify file quality and integrity

### Safety Measures

1. **Always Backup**: Never replace without backup
2. **Gradual Deployment**: Test on single device first
3. **Rollback Plan**: Always have rollback procedure
4. **User Communication**: Inform users about changes

### Performance Considerations

1. **File Size**: Keep replacement files reasonable size
2. **Loading Time**: Consider impact on boot time
3. **Memory Usage**: Monitor memory impact
4. **Storage Space**: Consider storage requirements

## Troubleshooting

### Common Issues

1. **Permission Denied**:
   ```bash
   # Fix permissions
   chmod 644 system/app/MyApp/MyApp.apk
   chown root:root system/app/MyApp/MyApp.apk
   ```

2. **App Won't Install**:
   ```bash
   # Check APK validity
   aapt dump badging MyApp.apk
   
   # Verify file integrity
   unzip -t MyApp.apk
   ```

3. **Boot Loop**:
   ```bash
   # Use recovery mode to restore
   ./manage_replacements.sh restore system/framework/framework-res.apk
   ```

4. **Audio Not Working**:
   ```bash
   # Check audio format
   file notification.ogg
   
   # Verify audio codec
   mediainfo notification.ogg
   ```

### Debugging

Enable debug mode for detailed logging:

```ini
[general]
debug_mode=true
verbose_logging=true
log_file=/data/local/tmp/replacement_debug.log
```

### Recovery

If system becomes unstable:

1. **Boot to Recovery**: Use custom recovery
2. **Remove Module**: Delete module directory
3. **Restore Backups**: Manually restore from backup directory
4. **Factory Reset**: Last resort option

## Security Considerations

### File Integrity

- Verify checksums of replacement files
- Use secure sources for replacement content
- Monitor for unauthorized changes

### Permission Management

- Use minimal required permissions
- Avoid world-writable files
- Secure backup directory access

### System Stability

- Test on non-production devices first
- Monitor system performance after replacement
- Have emergency recovery procedures

## Development Notes

This example demonstrates:

1. **Safe Replacement Patterns**: How to safely replace system files
2. **Backup Strategies**: Automatic backup and restore systems
3. **Verification Methods**: File validation and integrity checking
4. **Error Handling**: Robust error recovery mechanisms
5. **User Interface**: Command-line management tools

## License

This example is provided for educational purposes. Ensure compliance with applicable licenses for any replacement content.

## Support

For questions and support:
- Check the main project documentation
- Review the API reference for replacement functions
- Examine the backup and rollback procedures
- Consult the development community

#!/system/bin/sh

# System Replacement Example - Customization Script

# Variables
MODDIR=${0%/*}
MODID="replacement_example"

# Load common functions
source "$MODDIR/lib/common-functions.sh" 2>/dev/null || {
    echo "Error: Failed to load common functions"
    exit 1
}

log_info "Installing System Replacement Example"

# Create replacement directory structure
create_replacement_structure() {
    log_info "Creating replacement directory structure"
    
    # Create system directories
    mkdir -p "$MODDIR/system/app"
    mkdir -p "$MODDIR/system/priv-app"
    mkdir -p "$MODDIR/system/framework"
    mkdir -p "$MODDIR/system/lib"
    mkdir -p "$MODDIR/system/lib64"
    mkdir -p "$MODDIR/system/bin"
    mkdir -p "$MODDIR/system/etc"
    mkdir -p "$MODDIR/system/fonts"
    mkdir -p "$MODDIR/system/media/audio"
    mkdir -p "$MODDIR/system/usr/share"
    
    # Create vendor directories
    mkdir -p "$MODDIR/vendor/lib"
    mkdir -p "$MODDIR/vendor/lib64"
    mkdir -p "$MODDIR/vendor/etc"
    
    # Create backup directory
    mkdir -p "$MODDIR/backup"
    mkdir -p "$MODDIR/backup/system"
    mkdir -p "$MODDIR/backup/vendor"
    
    log_info "Directory structure created successfully"
}

# Function to backup original files
backup_original_file() {
    local source_path="$1"
    local backup_name="$2"
    local backup_path="$MODDIR/backup/$backup_name"
    
    if [ -f "$source_path" ]; then
        log_info "Backing up original file: $source_path"
        
        # Create backup directory if needed
        mkdir -p "$(dirname "$backup_path")"
        
        # Copy original file to backup
        if cp "$source_path" "$backup_path"; then
            log_info "Backup created: $backup_path"
            return 0
        else
            log_error "Failed to backup: $source_path"
            return 1
        fi
    else
        log_warn "Original file not found: $source_path"
        return 1
    fi
}

# Function to verify replacement file
verify_replacement_file() {
    local replacement_path="$1"
    
    if [ ! -f "$replacement_path" ]; then
        log_error "Replacement file not found: $replacement_path"
        return 1
    fi
    
    # Check file size (should not be empty)
    if [ ! -s "$replacement_path" ]; then
        log_error "Replacement file is empty: $replacement_path"
        return 1
    fi
    
    # For APK files, verify it's a valid ZIP
    if echo "$replacement_path" | grep -q "\.apk$"; then
        if ! unzip -t "$replacement_path" >/dev/null 2>&1; then
            log_error "Invalid APK file: $replacement_path"
            return 1
        fi
    fi
    
    # For library files, check if it's a valid ELF
    if echo "$replacement_path" | grep -q "\.so$"; then
        if ! file "$replacement_path" | grep -q "ELF"; then
            log_error "Invalid library file: $replacement_path"
            return 1
        fi
    fi
    
    log_info "Replacement file verified: $replacement_path"
    return 0
}

# Example: Replace system calculator app
replace_calculator_app() {
    log_info "Setting up calculator app replacement"
    
    # Define paths
    local system_calculator="/system/app/Calculator/Calculator.apk"
    local replacement_calculator="$MODDIR/system/app/Calculator/Calculator.apk"
    
    # Create app directory
    mkdir -p "$MODDIR/system/app/Calculator"
    
    # For this example, we'll create a placeholder replacement
    # In a real module, you would include your custom APK file
    cat > "$MODDIR/system/app/Calculator/placeholder.txt" << 'EOF'
# Calculator App Replacement Placeholder
#
# To actually replace the calculator app:
# 1. Place your custom Calculator.apk in this directory
# 2. Remove this placeholder file
# 3. Ensure the APK has proper permissions and signatures
#
# Example replacement APK requirements:
# - Must be a valid Android APK
# - Should have same or compatible permissions
# - Recommended to use system signature if available
EOF
    
    # Backup original if it exists
    if [ -f "$system_calculator" ]; then
        backup_original_file "$system_calculator" "system/app/Calculator/Calculator.apk"
    fi
    
    log_info "Calculator replacement setup completed"
}

# Example: Replace system sounds
replace_system_sounds() {
    log_info "Setting up system sounds replacement"
    
    # Create audio directories
    mkdir -p "$MODDIR/system/media/audio/notifications"
    mkdir -p "$MODDIR/system/media/audio/ringtones"
    mkdir -p "$MODDIR/system/media/audio/ui"
    
    # Create example replacement for notification sound
    cat > "$MODDIR/system/media/audio/notifications/placeholder.txt" << 'EOF'
# System Sounds Replacement Placeholder
#
# To replace system sounds:
# 1. Place your custom audio files (.ogg, .mp3, .wav) in appropriate directories
# 2. Remove placeholder files
# 3. Ensure audio files have correct format and quality
#
# Supported formats:
# - OGG Vorbis (recommended)
# - MP3
# - WAV
# - AAC
#
# Directory structure:
# - notifications/: Notification sounds
# - ringtones/: Phone ringtones  
# - ui/: UI feedback sounds
EOF
    
    # Set up permissions for audio directories
    set_perm_recursive "$MODDIR/system/media" 0 0 0755 0644
    
    log_info "System sounds replacement setup completed"
}

# Example: Replace system fonts
replace_system_fonts() {
    log_info "Setting up system fonts replacement"
    
    # Create fonts directory
    mkdir -p "$MODDIR/system/fonts"
    
    # Create placeholder for font replacement
    cat > "$MODDIR/system/fonts/placeholder.txt" << 'EOF'
# System Fonts Replacement Placeholder
#
# To replace system fonts:
# 1. Place your custom font files (.ttf, .otf) in this directory
# 2. Remove this placeholder file
# 3. Update fonts.xml if necessary
#
# Common replaceable fonts:
# - Roboto-Regular.ttf: Main UI font
# - Roboto-Bold.ttf: Bold text
# - Roboto-Italic.ttf: Italic text
# - DroidSansMono.ttf: Monospace font
#
# Font requirements:
# - Must be valid TrueType (.ttf) or OpenType (.otf)
# - Should include necessary character sets
# - Recommended to maintain similar metrics to originals
EOF
    
    log_info "System fonts replacement setup completed"
}

# Example: Replace system binaries
replace_system_binaries() {
    log_info "Setting up system binaries replacement"
    
    # Create bin directory
    mkdir -p "$MODDIR/system/bin"
    
    # Create placeholder for binary replacement
    cat > "$MODDIR/system/bin/placeholder.txt" << 'EOF'
# System Binaries Replacement Placeholder
#
# To replace system binaries:
# 1. Place your custom binary files in this directory
# 2. Remove this placeholder file
# 3. Ensure binaries have correct permissions (0755)
# 4. Verify architecture compatibility (arm64, arm, x86_64, x86)
#
# Commonly replaced binaries:
# - busybox: Enhanced shell utilities
# - sqlite3: Database management
# - curl: HTTP client
# - wget: File downloader
#
# Important notes:
# - Binaries must be compiled for Android
# - Must match device architecture
# - Should be statically linked or have all dependencies
# - Requires executable permissions
EOF
    
    log_info "System binaries replacement setup completed"
}

# Create configuration file for replacement management
create_replacement_config() {
    log_info "Creating replacement configuration"
    
    cat > "$MODDIR/replacement_config.ini" << 'EOF'
[general]
# General replacement settings
enabled=true
backup_originals=true
verify_before_replace=true
rollback_on_failure=true

[apps]
# Application replacement settings
replace_calculator=false
replace_gallery=false
replace_browser=false
verify_signatures=true

[media]
# Media replacement settings
replace_sounds=false
replace_ringtones=false
replace_notifications=false
audio_quality_check=true

[fonts]
# Font replacement settings
replace_system_fonts=false
replace_emoji_fonts=false
font_fallback_check=true

[binaries]
# Binary replacement settings
replace_busybox=false
replace_sqlite=false
architecture_check=true
dependency_check=true

[backup]
# Backup settings
auto_backup=true
backup_compression=false
backup_verification=true
max_backup_size=100MB

[rollback]
# Rollback settings
auto_rollback_on_boot_failure=true
rollback_timeout=30
keep_rollback_data=true
EOF
    
    set_perm "$MODDIR/replacement_config.ini" 0 0 0644
    log_info "Replacement configuration created"
}

# Create replacement management script
create_replacement_manager() {
    log_info "Creating replacement management script"
    
    cat > "$MODDIR/manage_replacements.sh" << 'EOF'
#!/system/bin/sh

# Replacement Management Script
MODDIR=${0%/*}
CONFIG_FILE="$MODDIR/replacement_config.ini"

# Load common functions
source "$MODDIR/lib/common-functions.sh" 2>/dev/null || {
    echo "Error: Failed to load common functions"
    exit 1
}

# Function to list active replacements
list_replacements() {
    echo "=== Active Replacements ==="
    
    find "$MODDIR/system" -type f ! -name "*.txt" 2>/dev/null | while read file; do
        relative_path=${file#$MODDIR}
        echo "ACTIVE: $relative_path"
    done
    
    find "$MODDIR/vendor" -type f ! -name "*.txt" 2>/dev/null | while read file; do
        relative_path=${file#$MODDIR}
        echo "ACTIVE: $relative_path"
    done
}

# Function to list available backups
list_backups() {
    echo "=== Available Backups ==="
    
    find "$MODDIR/backup" -type f 2>/dev/null | while read file; do
        relative_path=${file#$MODDIR/backup/}
        echo "BACKUP: $relative_path"
    done
}

# Function to restore from backup
restore_backup() {
    local backup_file="$1"
    local backup_path="$MODDIR/backup/$backup_file"
    local target_path="/$backup_file"
    
    if [ -f "$backup_path" ]; then
        echo "Restoring: $backup_file"
        
        # Mount system as read-write if needed
        if ! is_mounted /system; then
            mount -o remount,rw /system
        fi
        
        # Restore file
        if cp "$backup_path" "$target_path"; then
            echo "Restored successfully: $backup_file"
        else
            echo "Failed to restore: $backup_file"
        fi
    else
        echo "Backup not found: $backup_file"
    fi
}

# Function to validate replacements
validate_replacements() {
    echo "=== Validating Replacements ==="
    
    find "$MODDIR/system" "$MODDIR/vendor" -type f ! -name "*.txt" 2>/dev/null | while read file; do
        if verify_replacement_file "$file"; then
            echo "VALID: $file"
        else
            echo "INVALID: $file"
        fi
    done
}

# Main script logic
case "$1" in
    list)
        list_replacements
        ;;
    backups)
        list_backups
        ;;
    restore)
        if [ -n "$2" ]; then
            restore_backup "$2"
        else
            echo "Usage: $0 restore <backup_file>"
        fi
        ;;
    validate)
        validate_replacements
        ;;
    *)
        echo "Usage: $0 {list|backups|restore|validate}"
        echo ""
        echo "Commands:"
        echo "  list     - List active replacements"
        echo "  backups  - List available backups"
        echo "  restore  - Restore file from backup"
        echo "  validate - Validate replacement files"
        exit 1
        ;;
esac
EOF
    
    set_perm "$MODDIR/manage_replacements.sh" 0 0 0755
    log_info "Replacement management script created"
}

# Main installation process
log_info "Starting replacement module installation"

# Create directory structure
create_replacement_structure

# Set up example replacements
replace_calculator_app
replace_system_sounds
replace_system_fonts
replace_system_binaries

# Create configuration and management tools
create_replacement_config
create_replacement_manager

# Set permissions
log_info "Setting file permissions"
set_perm_recursive "$MODDIR" 0 0 0755 0644
set_perm "$MODDIR/manage_replacements.sh" 0 0 0755

# Copy library files if available
if [ -d "/data/adb/modules/pyrmm/usr/lib" ]; then
    log_info "Copying common library files"
    mkdir -p "$MODDIR/lib"
    cp -f "/data/adb/modules/pyrmm/usr/lib/common-functions.sh" "$MODDIR/lib/" 2>/dev/null
    cp -f "/data/adb/modules/pyrmm/usr/lib/module-manager.sh" "$MODDIR/lib/" 2>/dev/null
fi

# Installation completion
log_info "System Replacement Example installation completed"

# Display usage information
cat << 'EOF'

=== System Replacement Module Example ===

This module demonstrates how to replace system files including:
- Applications (APKs)
- System sounds and media
- Fonts
- Binary executables
- Configuration files

Management Commands:
- List active replacements: ./manage_replacements.sh list
- List backups: ./manage_replacements.sh backups
- Restore backup: ./manage_replacements.sh restore <file>
- Validate files: ./manage_replacements.sh validate

Configuration: replacement_config.ini

IMPORTANT: 
- This is an EXAMPLE module with PLACEHOLDER files
- Replace placeholder files with actual content
- Always backup original files before replacement
- Test replacements thoroughly before deployment

EOF

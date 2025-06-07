#ifndef KERNELSU_MODULE_H
#define KERNELSU_MODULE_H

/*
 * KernelSU Module Development Header
 * 
 * This header provides constants and definitions for KernelSU module development.
 * Based on KernelSU documentation: https://kernelsu.org/zh_CN/guide/module.html
 */

/* KernelSU Environment Variables */
#define KSU_ENV_VAR "KSU"
#define KSU_VERSION_VAR "KSU_VER"
#define KSU_VERSION_CODE_VAR "KSU_VER_CODE"
#define KSU_KERNEL_VERSION_CODE_VAR "KSU_KERNEL_VER_CODE"

/* Magisk Compatibility Variables */
#define MAGISK_VER_CODE_VAR "MAGISK_VER_CODE"
#define MAGISK_VER_VAR "MAGISK_VER"
#define MAGISK_COMPAT_VER_CODE "25200"
#define MAGISK_COMPAT_VER "v25.2"

/* Module Paths */
#define MODULES_ROOT "/data/adb/modules"
#define KSU_BIN_PATH "/data/adb/ksu/bin"
#define BUSYBOX_PATH "/data/adb/ksu/bin/busybox"

/* Module Files */
#define MODULE_PROP "module.prop"
#define SYSTEM_PROP "system.prop"
#define SEPOLICY_RULE "sepolicy.rule"

/* Script Files */
#define POST_FS_DATA_SCRIPT "post-fs-data.sh"
#define POST_MOUNT_SCRIPT "post-mount.sh"
#define SERVICE_SCRIPT "service.sh"
#define BOOT_COMPLETED_SCRIPT "boot-completed.sh"
#define UNINSTALL_SCRIPT "uninstall.sh"
#define CUSTOMIZE_SCRIPT "customize.sh"

/* Marker Files */
#define SKIP_MOUNT_MARKER "skip_mount"
#define DISABLE_MARKER "disable"
#define REMOVE_MARKER "remove"

/* Directory Names */
#define SYSTEM_DIR "system"
#define VENDOR_DIR "vendor"
#define PRODUCT_DIR "product"
#define SYSTEM_EXT_DIR "system_ext"
#define WEBROOT_DIR "webroot"
#define META_INF_DIR "META-INF"

/* Install Variables */
#define BOOTMODE_VAR "BOOTMODE"
#define MODPATH_VAR "MODPATH"
#define TMPDIR_VAR "TMPDIR"
#define ZIPFILE_VAR "ZIPFILE"
#define ARCH_VAR "ARCH"
#define IS64BIT_VAR "IS64BIT"
#define API_VAR "API"

/* Architecture Types */
#define ARCH_ARM "arm"
#define ARCH_ARM64 "arm64"
#define ARCH_X86 "x86"
#define ARCH_X64 "x64"

/* Module Property Keys */
#define PROP_ID "id"
#define PROP_NAME "name"
#define PROP_VERSION "version"
#define PROP_VERSION_CODE "versionCode"
#define PROP_AUTHOR "author"
#define PROP_DESCRIPTION "description"

/* BusyBox Environment */
#define ASH_STANDALONE_VAR "ASH_STANDALONE"
#define ASH_STANDALONE_VALUE "1"

/* Install Functions (for shell scripts) */
#define UI_PRINT_FUNC "ui_print"
#define ABORT_FUNC "abort"
#define SET_PERM_FUNC "set_perm"
#define SET_PERM_RECURSIVE_FUNC "set_perm_recursive"

/* Default Permissions */
#define DEFAULT_DIR_PERM "0755"
#define DEFAULT_FILE_PERM "0644"
#define DEFAULT_EXEC_PERM "0755"
#define DEFAULT_CONTEXT "u:object_r:system_file:s0"

/* Common Paths for System Overlay */
#define SYSTEM_BIN_PATH "/system/bin"
#define SYSTEM_LIB_PATH "/system/lib"
#define SYSTEM_LIB64_PATH "/system/lib64"
#define SYSTEM_ETC_PATH "/system/etc"
#define SYSTEM_APP_PATH "/system/app"
#define SYSTEM_PRIV_APP_PATH "/system/priv-app"

/* WebUI Support */
#define WEBUI_INDEX "index.html"
#define WEBUI_PORT_DEFAULT "8080"

/* Script Execution Modes */
typedef enum {
    SCRIPT_MODE_POST_FS_DATA,
    SCRIPT_MODE_POST_MOUNT,
    SCRIPT_MODE_SERVICE,
    SCRIPT_MODE_BOOT_COMPLETED
} script_mode_t;

/* Module Types */
typedef enum {
    MODULE_TYPE_BASIC,
    MODULE_TYPE_SYSTEMLESS,
    MODULE_TYPE_WEBUI,
    MODULE_TYPE_SERVICE
} module_type_t;

/* Helper Macros */
#define MODDIR_VAR "${0%/*}"
#define KSU_CHECK "[ \"$KSU\" = \"true\" ]"
#define MAGISK_CHECK "[ \"$MAGISK_VER_CODE\" != \"\" ]"

#endif /* KERNELSU_MODULE_H */

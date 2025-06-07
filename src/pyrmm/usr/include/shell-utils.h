#ifndef SHELL_UTILS_H
#define SHELL_UTILS_H

/*
 * Shell Utilities for KernelSU Module Development
 * 
 * Common shell script utilities and best practices for KernelSU modules.
 * These definitions can be sourced in shell scripts.
 */

/* Color Definitions for Terminal Output */
#define COLOR_RED '\033[0;31m'
#define COLOR_GREEN '\033[0;32m'
#define COLOR_YELLOW '\033[1;33m'
#define COLOR_BLUE '\033[0;34m'
#define COLOR_PURPLE '\033[0;35m'
#define COLOR_CYAN '\033[0;36m'
#define COLOR_WHITE '\033[1;37m'
#define COLOR_NC '\033[0m'

/* Logging Functions Template */
#define LOG_FUNCTIONS \
"log_info() { echo -e \"\\033[0;34m[INFO]\\033[0m $1\"; }\n" \
"log_success() { echo -e \"\\033[0;32m[SUCCESS]\\033[0m $1\"; }\n" \
"log_warning() { echo -e \"\\033[1;33m[WARNING]\\033[0m $1\"; }\n" \
"log_error() { echo -e \"\\033[0;31m[ERROR]\\033[0m $1\"; }\n"

/* Common Shell Patterns */
#define MODDIR_DETECTION "MODDIR=${0%/*}"
#define BUSYBOX_SETUP "export PATH=\"/data/adb/ksu/bin:$PATH\""
#define ASH_STANDALONE_SETUP "export ASH_STANDALONE=1"

/* File Permission Helpers */
#define SET_EXEC_PERM "chmod 755"
#define SET_READ_PERM "chmod 644"
#define SET_DIR_PERM "chmod 755"

/* Common Check Functions */
#define CHECK_ROOT \
"check_root() {\n" \
"    if [ \"$(id -u)\" != \"0\" ]; then\n" \
"        log_error \"This script must be run as root\"\n" \
"        exit 1\n" \
"    fi\n" \
"}\n"

#define CHECK_KERNELSU \
"check_kernelsu() {\n" \
"    if [ \"$KSU\" != \"true\" ]; then\n" \
"        log_error \"This script requires KernelSU\"\n" \
"        exit 1\n" \
"    fi\n" \
"}\n"

#define WAIT_FOR_BOOT \
"wait_for_boot() {\n" \
"    while [ \"$(getprop sys.boot_completed)\" != \"1\" ]; do\n" \
"        sleep 1\n" \
"    done\n" \
"}\n"

/* Property Manipulation */
#define RESET_PROP "resetprop"
#define GET_PROP "getprop"
#define SET_PROP_SAFE "resetprop -n"

/* File System Operations */
#define MOUNT_RO "mount -o remount,ro"
#define MOUNT_RW "mount -o remount,rw"
#define CREATE_WHITEOUT "mknod"

/* SELinux Helpers */
#define SELINUX_ENFORCING "getenforce"
#define SELINUX_PERMISSIVE "setenforce 0"
#define SELINUX_RESTORE "restorecon"

/* Network Utilities */
#define CHECK_INTERNET \
"check_internet() {\n" \
"    ping -c 1 8.8.8.8 >/dev/null 2>&1\n" \
"}\n"

/* Package Manager Detection */
#define DETECT_PM \
"detect_pm() {\n" \
"    if command -v pm >/dev/null 2>&1; then\n" \
"        echo \"pm\"\n" \
"    elif command -v cmd >/dev/null 2>&1; then\n" \
"        echo \"cmd package\"\n" \
"    else\n" \
"        echo \"unknown\"\n" \
"    fi\n" \
"}\n"

/* Service Management */
#define START_SERVICE "start"
#define STOP_SERVICE "stop"
#define RESTART_SERVICE "restart"

/* Archive Operations */
#define EXTRACT_ZIP "unzip -o"
#define EXTRACT_TAR "tar -xf"
#define CREATE_ZIP "zip -r"

/* Download Utilities */
#define WGET_CMD "wget -O"
#define CURL_CMD "curl -L -o"

/* Text Processing */
#define GREP_QUIET "grep -q"
#define SED_INPLACE "sed -i"
#define AWK_FIELD "awk '{print $1}'"

#endif /* SHELL_UTILS_H */

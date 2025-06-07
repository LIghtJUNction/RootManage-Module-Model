/*
 * KernelSU模块开发本地配置头文件
 * KernelSU Module Development Local Configuration Header
 * 
 * 这个文件包含本地开发环境特定的配置和定义
 * This file contains local development environment specific configurations and definitions
 */

#ifndef _KERNELSU_LOCAL_CONFIG_H
#define _KERNELSU_LOCAL_CONFIG_H

/* 本地开发路径配置 */
/* Local development path configuration */
#define LOCAL_DEV_ROOT          "/usr/local/share/kernelsu-dev"
#define LOCAL_TEMPLATES_DIR     "/usr/local/share/kernelsu-dev/templates"
#define LOCAL_EXAMPLES_DIR      "/usr/local/share/kernelsu-dev/examples"
#define LOCAL_DOCS_DIR          "/usr/local/share/kernelsu-dev/docs"
#define LOCAL_TOOLS_DIR         "/usr/local/bin"
#define LOCAL_CONFIG_DIR        "/usr/local/etc"

/* 本地缓存配置 */
/* Local cache configuration */
#define LOCAL_CACHE_DIR         "${HOME}/.cache/kernelsu-dev"
#define LOCAL_TEMP_DIR          "/tmp/kernelsu-dev"
#define LOCAL_LOG_DIR           "${HOME}/.local/share/kernelsu-dev/logs"

/* 项目配置文件名 */
/* Project configuration file names */
#define PROJECT_CONFIG_FILE     ".kernelsu-project"
#define BUILD_CONFIG_FILE       "build.conf"
#define MODULE_CONFIG_FILE      "module.prop"
#define WEBUI_CONFIG_FILE       "webui.conf"

/* 开发工具配置 */
/* Development tools configuration */
#define EDITOR_CONFIG_FILE      ".editorconfig"
#define VSCODE_CONFIG_DIR       ".vscode"
#define GIT_CONFIG_FILE         ".gitignore"
#define LINT_CONFIG_FILE        ".shellcheckrc"

/* 本地环境变量 */
/* Local environment variables */
#define ENV_KERNELSU_DEV_ROOT   "KERNELSU_DEV_ROOT"
#define ENV_MODULE_DEV_MODE     "MODULE_DEV_MODE"
#define ENV_DEBUG_ENABLED       "DEBUG_ENABLED"
#define ENV_VERBOSE_OUTPUT      "VERBOSE_OUTPUT"

/* 开发模式标志 */
/* Development mode flags */
#define DEV_MODE_STRICT         0x01
#define DEV_MODE_DEBUG          0x02
#define DEV_MODE_VERBOSE        0x04
#define DEV_MODE_LINT           0x08
#define DEV_MODE_TEST           0x10

/* 本地构建配置 */
/* Local build configuration */
#define BUILD_TYPE_DEBUG        "debug"
#define BUILD_TYPE_RELEASE      "release"
#define BUILD_TYPE_TEST         "test"

#define ARCH_ARM                "arm"
#define ARCH_ARM64              "arm64"
#define ARCH_X86                "x86"
#define ARCH_X86_64             "x86_64"

/* 默认编辑器命令 */
/* Default editor commands */
#define EDITOR_VSCODE           "code"
#define EDITOR_VIM              "vim"
#define EDITOR_NANO             "nano"
#define EDITOR_EMACS            "emacs"

/* 本地服务配置 */
/* Local service configuration */
#define WEBUI_DEFAULT_PORT      8080
#define WEBUI_DEFAULT_HOST      "localhost"
#define API_DEFAULT_PORT        8081
#define DOCS_DEFAULT_PORT       8082

/* 文件扩展名 */
/* File extensions */
#define EXT_MODULE              ".zip"
#define EXT_SCRIPT              ".sh"
#define EXT_CONFIG              ".conf"
#define EXT_TEMPLATE            ".template"
#define EXT_BACKUP              ".bak"

/* 权限配置 */
/* Permission configuration */
#define PERM_EXECUTABLE         0755
#define PERM_READABLE           0644
#define PERM_CONFIG             0600
#define PERM_DIRECTORY          0755

#endif /* _KERNELSU_LOCAL_CONFIG_H */

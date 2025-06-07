/*
 * 开发环境实用工具宏定义
 * Development Environment Utility Macros
 * 
 * 这个文件包含开发过程中常用的宏定义和内联函数
 * This file contains commonly used macro definitions and inline functions during development
 */

#ifndef _DEV_UTILS_H
#define _DEV_UTILS_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <time.h>

/* 调试输出宏 */
/* Debug output macros */
#ifdef DEBUG
    #define DEBUG_PRINT(fmt, ...) \
        fprintf(stderr, "[DEBUG] %s:%d: " fmt "\n", __FILE__, __LINE__, ##__VA_ARGS__)
#else
    #define DEBUG_PRINT(fmt, ...) do {} while(0)
#endif

/* 错误处理宏 */
/* Error handling macros */
#define ERROR_PRINT(fmt, ...) \
    fprintf(stderr, "[ERROR] %s:%d: " fmt "\n", __FILE__, __LINE__, ##__VA_ARGS__)

#define WARN_PRINT(fmt, ...) \
    fprintf(stderr, "[WARN] %s:%d: " fmt "\n", __FILE__, __LINE__, ##__VA_ARGS__)

#define INFO_PRINT(fmt, ...) \
    printf("[INFO] " fmt "\n", ##__VA_ARGS__)

/* 内存管理宏 */
/* Memory management macros */
#define SAFE_FREE(ptr) do { \
    if (ptr) { \
        free(ptr); \
        ptr = NULL; \
    } \
} while(0)

#define SAFE_MALLOC(size) ({ \
    void *ptr = malloc(size); \
    if (!ptr) { \
        ERROR_PRINT("Memory allocation failed"); \
        exit(EXIT_FAILURE); \
    } \
    ptr; \
})

#define SAFE_STRDUP(str) ({ \
    char *new_str = strdup(str); \
    if (!new_str) { \
        ERROR_PRINT("String duplication failed"); \
        exit(EXIT_FAILURE); \
    } \
    new_str; \
})

/* 字符串操作宏 */
/* String operation macros */
#define STR_EMPTY(str) (!str || !*str)
#define STR_EQUAL(str1, str2) (strcmp(str1, str2) == 0)
#define STR_START_WITH(str, prefix) (strncmp(str, prefix, strlen(prefix)) == 0)
#define STR_END_WITH(str, suffix) ({ \
    size_t str_len = strlen(str); \
    size_t suffix_len = strlen(suffix); \
    str_len >= suffix_len && strcmp(str + str_len - suffix_len, suffix) == 0; \
})

/* 数组操作宏 */
/* Array operation macros */
#define ARRAY_SIZE(arr) (sizeof(arr) / sizeof((arr)[0]))
#define ARRAY_FOREACH(arr, var) \
    for (var = arr; var < arr + ARRAY_SIZE(arr); var++)

/* 文件操作宏 */
/* File operation macros */
#define FILE_EXISTS(path) (access(path, F_OK) == 0)
#define FILE_READABLE(path) (access(path, R_OK) == 0)
#define FILE_WRITABLE(path) (access(path, W_OK) == 0)
#define FILE_EXECUTABLE(path) (access(path, X_OK) == 0)

/* 时间操作宏 */
/* Time operation macros */
#define TIMESTAMP() ({ \
    time_t t = time(NULL); \
    struct tm *tm = localtime(&t); \
    static char buf[64]; \
    strftime(buf, sizeof(buf), "%Y-%m-%d %H:%M:%S", tm); \
    buf; \
})

/* 颜色输出宏 (终端) */
/* Color output macros (terminal) */
#define COLOR_RESET     "\033[0m"
#define COLOR_RED       "\033[31m"
#define COLOR_GREEN     "\033[32m"
#define COLOR_YELLOW    "\033[33m"
#define COLOR_BLUE      "\033[34m"
#define COLOR_MAGENTA   "\033[35m"
#define COLOR_CYAN      "\033[36m"
#define COLOR_WHITE     "\033[37m"

#define COLORED_PRINT(color, fmt, ...) \
    printf(color fmt COLOR_RESET "\n", ##__VA_ARGS__)

#define SUCCESS_PRINT(fmt, ...) COLORED_PRINT(COLOR_GREEN, fmt, ##__VA_ARGS__)
#define ERROR_COLORED_PRINT(fmt, ...) COLORED_PRINT(COLOR_RED, fmt, ##__VA_ARGS__)
#define WARNING_PRINT(fmt, ...) COLORED_PRINT(COLOR_YELLOW, fmt, ##__VA_ARGS__)
#define INFO_COLORED_PRINT(fmt, ...) COLORED_PRINT(COLOR_BLUE, fmt, ##__VA_ARGS__)

/* 条件编译宏 */
/* Conditional compilation macros */
#ifdef ANDROID
    #define PLATFORM_ANDROID 1
    #define PLATFORM_LINUX 0
#else
    #define PLATFORM_ANDROID 0
    #define PLATFORM_LINUX 1
#endif

/* 实用函数声明 */
/* Utility function declarations */
#ifdef __cplusplus
extern "C" {
#endif

/* 字符串实用函数 */
char* trim_whitespace(char* str);
char* str_replace(const char* str, const char* old, const char* new);
int str_split(const char* str, const char* delim, char*** result);

/* 文件实用函数 */
int file_copy(const char* src, const char* dest);
int file_move(const char* src, const char* dest);
char* file_read_all(const char* path);
int file_write_all(const char* path, const char* content);

/* 目录实用函数 */
int dir_create_recursive(const char* path, mode_t mode);
int dir_remove_recursive(const char* path);
int dir_exists(const char* path);

/* 进程实用函数 */
int exec_command(const char* cmd, char** output);
int exec_command_with_timeout(const char* cmd, int timeout, char** output);

#ifdef __cplusplus
}
#endif

#endif /* _DEV_UTILS_H */

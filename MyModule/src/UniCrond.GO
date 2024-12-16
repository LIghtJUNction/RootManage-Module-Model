// 目标平台 Android
// 架构 Arm64
// 介绍：crond 的改进版本，实时扫描 /data/adb/modules/$moduleid/UniCron/ 下的 $moduleid.cron 文件，合并后统一执行定时任务
// 特点：仅可开启单一进程，无需额外配置，资源占用小
// 基本功能：按照 cron 语法定时执行任务
package main

import (
    "flag"
    "fmt"
    "log"
    "os"
    "path/filepath"
    "strings"
    "sync"
    "os/exec"

    "github.com/robfig/cron/v3"
    "github.com/gofrs/flock"
    "github.com/fsnotify/fsnotify"
)

var (
    runningTasks  = make(map[string]cron.EntryID)
    cronScheduler *cron.Cron
    mutex         sync.Mutex
)

func listRunningTasks() {
    mutex.Lock()
    defer mutex.Unlock()
    if len(runningTasks) == 0 {
        fmt.Println("当前没有正在运行的任务。")
        return
    }
    fmt.Println("正在运行的任务列表：")
    for task, id := range runningTasks {
        fmt.Printf("任务: %s, ID: %d\n", task, id)
    }
}

func runScheduler() {
    mutex.Lock()
    defer mutex.Unlock()
    // 避免调度器重复启动
    if cronScheduler != nil {
        fmt.Println("调度器已在运行。")
        return
    }
    cronScheduler = cron.New(cron.WithLogger(cron.VerbosePrintfLogger(log.New(os.Stdout, "cron: ", log.LstdFlags))))
    cronScheduler.Start()
    fmt.Println("调度器启动成功。")

    // 启动文件监控
    go watchCronFiles()
}

func watchCronFiles() {
    watcher, err := fsnotify.NewWatcher()
    if err != nil {
        log.Fatal(err)
    }
    defer watcher.Close()

    baseDir := "/data/adb/modules"
    err = filepath.Walk(baseDir, func(path string, info os.FileInfo, err error) error {
        if err != nil {
            return err
        }
        if info.IsDir() && filepath.Base(path) == "UniCron" {
            return watcher.Add(path)
        }
        return nil
    })
    if err != nil {
        log.Fatal(err)
    }

    for {
        select {
        case event, ok := <-watcher.Events:
            if !ok {
                return
            }
            if event.Op&fsnotify.Write == fsnotify.Write || event.Op&fsnotify.Create == fsnotify.Create || event.Op&fsnotify.Remove == fsnotify.Remove {
                fmt.Printf("检测到文件变化: %s\n", event.Name)
                loadTasks()
            }
        case err, ok := <-watcher.Errors:
            if !ok {
                return
            }
            fmt.Println("文件监控错误:", err)
        }
    }
}

func runWithoutInotify() {
    fmt.Println("以无 inotify 支持方式运行。")
    loadTasks()
    runScheduler()
}

func loadTasks() {
    lock := flock.New("/tmp/unicrond.lock")
    locked, err := lock.TryLock()
    if err != nil {
        fmt.Printf("无法获取锁: %v\n", err)
        os.Exit(1)
    }
    if !locked {
        fmt.Println("UniCrond 已在运行。")
        os.Exit(1)
    }
    defer lock.Unlock()

    baseDir := "/data/adb/modules"
    err = filepath.Walk(baseDir, func(path string, info os.FileInfo, err error) error {
        if err != nil {
            log.Printf("遍历路径时出错: %v\n", err)
            return err
        }

        if info.IsDir() && filepath.Base(path) == "UniCron" {
            files, err := os.ReadDir(path)
            if err != nil {
                log.Printf("读取目录时出错: %v\n", err)
                return err
            }

            for _, file := range files {
                if strings.HasSuffix(file.Name(), ".cron") {
                    cronFilePath := filepath.Join(path, file.Name())
                    content, err := os.ReadFile(cronFilePath)
                    if err != nil {
                        log.Printf("无法读取 cron 文件 %s: %v\n", cronFilePath, err)
                        continue
                    }

                    // 解析 cron 文件内容
                    lines := strings.Split(string(content), "\n")
                    for _, line := range lines {
                        trimmedLine := strings.TrimSpace(line)
                        if trimmedLine != "" {
                            // 验证 cron 表达式
                            parts := strings.Fields(trimmedLine)
                            if len(parts) < 6 {
                                log.Printf("无效的 cron 表达式: %s\n", trimmedLine)
                                continue
                            }
                            spec := strings.Join(parts[:5], " ")
                            command := strings.Join(parts[5:], " ")

                            _, err := cron.ParseStandard(spec)
                            if err != nil {
                                log.Printf("无效的 cron 格式 '%s' in file %s: %v\n", spec, cronFilePath, err)
                                continue
                            }

                            entryID, err := cronScheduler.AddFunc(spec, func() {
                                addJobToSchedule(command)
                            })
                            if err != nil {
                                log.Printf("添加任务失败 '%s': %v\n", command, err)
                                continue
                            }

                            runningTasks[command] = entryID
                            log.Printf("添加任务到调度：%s\n", command)
                        }
                    }
                }
            }
        }
        return nil
    })

    if err != nil {
        log.Printf("遍历模块目录时出错: %v\n", err)
    }
}

func addJobToSchedule(command string) {
    fmt.Printf("添加任务到调度：%s\n", command)
    // 示例：执行命令
    go func(cmd string) {
        output, err := exec.Command("sh", "-c", cmd).CombinedOutput()
        if err != nil {
            log.Printf("执行命令 '%s' 失败: %v\n", cmd, err)
        }
        fmt.Printf("命令 '%s' 输出: %s\n", cmd, string(output))
    }(command)
}

func printDebugInfo(flag string) {
    switch flag {
    case "tasks":
        fmt.Println("调试信息：当前任务列表")
        listRunningTasks()
    case "scheduler":
        fmt.Println("调试信息：调度器状态")
        if cronScheduler != nil {
            entries := cronScheduler.Entries()
            fmt.Printf("调度器中共有 %d 个任务。\n", len(entries))
            for _, entry := range entries {
                fmt.Printf("任务ID: %d, 下次运行: %v, 上次运行: %v\n", entry.ID, entry.Next, entry.Prev)
            }
        } else {
            fmt.Println("调度器尚未启动。")
        }
    default:
        fmt.Printf("未知的调试标志: %s\n", flag)
    }
}

func printUsage() {
    fmt.Println("UniCrond")
    fmt.Println("Usage:")
    fmt.Println("  unicrond [options]")
    fmt.Println()
    fmt.Println("Options:")
    fmt.Println("  -h         print this message")
    fmt.Println("  -i         daemon runs without inotify support")
    fmt.Println("  -n         run in foreground")
    fmt.Println("  -f         run in foreground, the same as -n")
    fmt.Println("  -l         Lists the tasks that are currently running")
    fmt.Println("  -V         print version and exit")
    fmt.Println("  -x <flag>  print debug information")
}

func main() {
    // 定义命令行参数
    version := flag.Bool("V", false, "print version and exit")
    listTasks := flag.Bool("l", false, "Lists the tasks that are currently running")
    debug := flag.String("x", "", "print debug information")
    foreground := flag.Bool("n", false, "run in foreground")
    foregroundAlt := flag.Bool("f", false, "run in foreground, the same as -n")
    inotify := flag.Bool("i", false, "daemon runs without inotify support")
    help := flag.Bool("h", false, "print this message")

    flag.Parse()

    if *help {
        printUsage()
        return
    }

    if *version {
        fmt.Println("UniCrond version 1.0")
        return
    }

    if *listTasks {
        listRunningTasks()
        return
    }

    if *debug != "" {
        printDebugInfo(*debug)
        return
    }

    if *foreground || *foregroundAlt {
        fmt.Println("运行前台模式")
        runScheduler()
        return
    }

    if *inotify {
        runWithoutInotify()
        return
    }

    fmt.Println("启动 UniCrond")
    loadTasks()
    runScheduler()
}

// UniCrond
// Usage:
//  unicrond [options]
// 
// Options:
//  -h         print this message 
//  -i         daemon runs without inotify support
//  -n         run in foreground
//  -f         run in foreground, the same as -n
//  -l         Lists the tasks that are currently running
//  -V         print version and exit
//  -x <flag>  print debug information


// 示例 
// 有一个模块/data/adb/modules/Demo/UniCron/Demo.cron
// 大多数模块没有UniCron文件夹，忽视即可
// Demo.cron文件内容为 * * * * * /data/adb/modules/Demo/Demo.sh
// UniCrond将识别这个文件，并将其添加到自己的任务集合里定时执行

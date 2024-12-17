
var (
    runningTasks        = make(map[string]cron.EntryID)
    cronScheduler       *cron.Cron
    mutex               sync.Mutex
    lockFilePath        = "/data/local/tmp/unicrond.lock"   // 全局锁文件路径
    logFile             *os.File
    lock                *flock.Flock
    ipcSocketPath       = "/data/local/tmp/unicrond.sock"  // Unix Socket 路径
    unicrontDir         = "/data/adb/modules/UniCron/"      // UniCron 目录
    currentTaskCommands = make(map[string]bool)            // 当前加载的任务命令
    shutdownChan        = make(chan struct{})
)

func init() {
    // 确保 /data/local/tmp 目录存在
    tmpDir := filepath.Dir(lockFilePath)
    if _, err := os.Stat(tmpDir); os.IsNotExist(err) {
        err := os.MkdirAll(tmpDir, 0755)
        if err != nil {
            log.Fatalf("无法创建临时目录: %v", err)
        }
    }
}

func initLogging() {
    disablePath := filepath.Join(unicrontDir, "disable")
    logsDir := filepath.Join(unicrontDir, "logs")
    logFilePath := filepath.Join(logsDir, "UniCron.log")

    if _, err := os.Stat(unicrontDir); !os.IsNotExist(err) {
        if _, err := os.Stat(disablePath); os.IsNotExist(err) {
            if _, err := os.Stat(logsDir); os.IsNotExist(err) {
                err := os.MkdirAll(logsDir, 0755)
                if err != nil {
                    log.Fatalf("无法创建日志目录: %v", err)
                }
            }
            var err error
            // 清空日志文件并设置权限为0666
            logFile, err = os.OpenFile(logFilePath, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, 0666)
            if err != nil {
                log.Fatalf("无法打开日志文件: %v", err)
            }
            log.SetOutput(io.MultiWriter(os.Stdout, logFile))
            log.Println("日志记录已初始化。")
        } else {
            log.Println("UniCron 被禁用，不初始化日志。")
        }
    } else {
        log.Println("UniCron 目录不存在，不初始化日志。")
    }
}

func removeDisabledTasks() {
    baseDir := "/data/adb/modules"
    modules, err := os.ReadDir(baseDir)
    if err != nil {
        log.Printf("读取模块目录时出错: %v\n", err)
        return
    }

    for _, module := range modules {
        if module.IsDir() && module.Name() != "UniCron" { // 排除顶级 UniCron 目录
            modulePath := filepath.Join(baseDir, module.Name())
            disablePath := filepath.Join(modulePath, "disable")
            if _, err := os.Stat(disablePath); err == nil {
                mutex.Lock()
                if entryID, exists := runningTasks[module.Name()]; exists {
                    cronScheduler.Remove(entryID)
                    delete(runningTasks, module.Name())
                    log.Printf("移除被禁用模块的任务: %s\n", module.Name())
                }
                mutex.Unlock()
            }
        }
    }
}

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
    log.Println("调度器启动成功。")

    // 启动文件监控
    go watchCronFiles()
}

func watchCronFiles() {
    watcher, err := fsnotify.NewWatcher()
    if err != nil {
        log.Fatalf("创建文件监视器失败: %v", err)
    }
    defer watcher.Close()

    baseDir := "/data/adb/modules"
    modules, err := os.ReadDir(baseDir)
    if err != nil {
        log.Fatalf("读取模块目录时出错: %v", err)
    }

    for _, module := range modules {
        if module.IsDir() && module.Name() != "UniCron" { // 排除顶级 UniCron 目录
            unicrontPath := filepath.Join(baseDir, module.Name(), "UniCron")
            if _, err := os.Stat(unicrontPath); err == nil {
                log.Printf("监控目录: %s\n", unicrontPath)
                err := watcher.Add(unicrontPath)
                if err != nil {
                    log.Printf("无法监控目录 %s: %v\n", unicrontPath, err)
                }
            }
        }
    }

    for {
        select {
        case event, ok := <-watcher.Events:
            if !ok {
                return
            }
            // 仅监控 .cron 文件的变化
            if strings.HasSuffix(event.Name, ".cron") && (event.Op&fsnotify.Write == fsnotify.Write || event.Op&fsnotify.Create == fsnotify.Create || event.Op&fsnotify.Remove == fsnotify.Remove) {
                log.Printf("检测到文件变化: %s\n", event.Name)
                loadTasks()
            }
        case err, ok := <-watcher.Errors:
            if !ok {
                return
            }
            log.Printf("文件监控错误: %v\n", err)
        }
    }
}

func runWithoutInotify() {
    log.Println("以无 inotify 支持方式运行。")
    runScheduler() // 初始化调度器
    loadTasks()    // 加载任务
}

func loadTasks() {
    mutex.Lock()
    defer mutex.Unlock()

    if cronScheduler == nil {
        log.Println("调度器未初始化，无法加载任务。")
        return
    }

    baseDir := "/data/adb/modules"
    log.Printf("尝试加载任务，遍历目录: %s\n", baseDir)

    modules, err := os.ReadDir(baseDir)
    if err != nil {
        log.Printf("读取模块目录时出错: %v\n", err)
        return
    }

    // 临时存储当前加载的任务命令
    newTaskCommands := make(map[string]bool)

    for _, module := range modules {
        if module.IsDir() && module.Name() != "UniCron" { // 排除顶级 UniCron 目录
            unicrontPath := filepath.Join(baseDir, module.Name(), "UniCron")
            if _, err := os.Stat(unicrontPath); err == nil {
                log.Printf("进入 UniCron 目录: %s\n", unicrontPath)

                files, err := os.ReadDir(unicrontPath)
                if err != nil {
                    log.Printf("读取目录时出错: %v\n", err)
                    continue
                }

                for _, file := range files {
                    if strings.HasSuffix(file.Name(), ".cron") {
                        cronFilePath := filepath.Join(unicrontPath, file.Name())
                        log.Printf("读取 cron 文件: %s\n", cronFilePath)
                        content, err := os.ReadFile(cronFilePath)
                        if err != nil {
                            log.Printf("无法读取 cron 文件 %s: %v\n", cronFilePath, err)
                            continue
                        }

                        // 解析 cron 文件内容
                        lines := strings.Split(string(content), "\n")
                        for _, line := range lines {
                            trimmedLine := strings.TrimSpace(line)
                            if trimmedLine != "" && !strings.HasPrefix(trimmedLine, "#") {
                                // 验证 cron 表达式
                                parts := strings.Fields(trimmedLine)
                                if len(parts) < 6 {
                                    log.Printf("无效的 cron 表达式: %s\n", trimmedLine)
                                    continue
                                }
                                spec := strings.Join(parts[:5], " ")
                                command := strings.Join(parts[5:], " ")

                                newTaskCommands[command] = true

                                if _, exists := runningTasks[command]; exists {
                                    log.Printf("任务已存在，跳过添加：%s\n", command)
                                    continue
                                }

                                _, err := cron.ParseStandard(spec)
                                if err != nil {
                                    log.Printf("无效的 cron 格式 '%s' in file %s: %v\n", spec, cronFilePath, err)
                                    continue
                                }

                                entryID, err := cronScheduler.AddFunc(spec, func(cmd string) func() {
                                    return func() { addJobToSchedule(cmd) }
                                }(command))
                                if err != nil {
                                    log.Printf("添加任务失败: %v\n", err)
                                    continue
                                }

                                runningTasks[command] = entryID
                                log.Printf("添加任务到调度：%s\n", command)
                            }
                        }
                    }
                }
            }
        }
    }

    // 移除不再存在的任务
    for cmd := range runningTasks {
        if !newTaskCommands[cmd] {
            log.Printf("移除不存在的任务: %s\n", cmd)
            cronScheduler.Remove(runningTasks[cmd])
            delete(runningTasks, cmd)
        }
    }

    currentTaskCommands = newTaskCommands
}

func addJobToSchedule(command string) {
    log.Printf("执行任务：%s\n", command)
    // 示例：执行命令
    go func(cmd string) {
        output, err := exec.Command("sh", "-c", cmd).CombinedOutput()
        if err != nil {
            log.Printf("执行命令 '%s' 失败: %v\n", cmd, err)
        }
        log.Printf("命令 '%s' 输出: %s\n", cmd, string(output))
    }(command)
}


func killExistingProcesses() error {
    conn, err := net.Dial("unix", ipcSocketPath)
    if err == nil {
        defer conn.Close()
        _, err = conn.Write([]byte("shutdown"))
        if err != nil {
            return err
        }
        buffer := make([]byte, 1024)
        n, err := conn.Read(buffer)
        if err != nil {
            return err
        }
        log.Print(string(buffer[:n]))
        // 等待进程退出
        time.Sleep(2 * time.Second)
    }
    // 检查锁文件是否存在
    if _, err := os.Stat(lockFilePath); err == nil {
        err := os.Remove(lockFilePath)
        if err != nil {
            log.Printf("无法移除锁文件: %v", err)
        }
    }
    return nil
}

func printDebugInfo(flag string) {
    switch flag {
    case "tasks":
        log.Println("调试信息：当前任务列表")
        listRunningTasks()
    case "scheduler":
        log.Println("调试信息：调度器状态")
        if cronScheduler != nil {
            entries := cronScheduler.Entries()
            log.Printf("调度器中共有 %d 个任务。\n", len(entries))
            for _, entry := range entries {
                log.Printf("任务ID: %d, 下次运行: %v, 上次运行: %v\n", entry.ID, entry.Next, entry.Prev)
            }
        } else {
            log.Println("调度器尚未启动。")
        }
    default:
        log.Printf("未知的调试标志: %s\n", flag)
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
    fmt.Println("  -f         force restart UniCrond")
    fmt.Println("  -b         run in background")
    fmt.Println("  -l         Lists the tasks that are currently running")
    fmt.Println("  -V         print version and exit")
    fmt.Println("  -x <flag>  print debug information")
}

func daemonize(args []string) {
    // 移除 '-b' 参数
    newArgs := []string{}
    for _, arg := range args {
        if arg != "-b" {
            newArgs = append(newArgs, arg)
        }
    }

    cmd := exec.Command(os.Args[0], newArgs...)
    // 设置会话ID，确保子进程与父进程分离
    cmd.SysProcAttr = &syscall.SysProcAttr{
        Setsid: true,
    }

    // 重定向标准输入、输出、错误
    cmd.Stdin = nil
    cmd.Stdout = nil
    cmd.Stderr = nil

    err := cmd.Start()
    if err != nil {
        log.Fatalf("守护进程启动失败: %v", err)
    }
    log.Printf("守护进程已启动，PID: %d\n", cmd.Process.Pid)
    os.Exit(0)
}

func main() {
    // 定义命令行参数
    version := flag.Bool("V", false, "print version and exit")
    listTasksFlag := flag.Bool("l", false, "Lists the tasks that are currently running")
    debug := flag.String("x", "", "print debug information")
    forceRestart := flag.Bool("f", false, "force restart UniCrond")
    foreground := flag.Bool("n", false, "run in foreground")
    background := flag.Bool("b", false, "run in background")
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

    // 如果 -f 标志被设置，尝试杀死已有的 UniCrond 进程
    if *forceRestart {
        log.Println("强制重启 UniCrond，尝试关闭已有的进程")
        err := killExistingProcesses()
        if err != nil {
            log.Printf("无法关闭已有的 UniCrond 进程: %v\n", err)
        } else {
            log.Println("已关闭已有的 UniCrond 进程")
        }
    }

    // 如果 -b 标志被设置，先进行守护进程化，再退出父进程
    if *background {
        log.Println("以后台模式运行。")
        daemonize(os.Args[1:])
    }

    // 尝试获取全局锁，确保仅有一个 UniCrond 实例运行
    lock = flock.New(lockFilePath)
    locked, err := lock.TryLock()
    if err != nil {
        log.Fatalf("无法获取锁: %v", err)
    }
    if !locked {
        log.Fatalf("锁已被其他进程持有，UniCrond 已在运行。")
    }
    defer func() {
        err := lock.Unlock()
        if err != nil {
            log.Printf("无法释放锁: %v", err)
        }
    }()

    // 初始化日志
    initLogging()
    // 启动 IPC 监听器
    startIPC()
    // 移除被禁用的任务
    removeDisabledTasks()

    // 如果 -l 标志被设置，连接到 IPC Socket 查询任务列表
    if *listTasksFlag {
        response, err := sendIPCCommand("list")
        if err != nil {
            fmt.Printf("无法连接到运行中的 UniCrond 实例: %v\n", err)
            os.Exit(1)
        }
        fmt.Print(response)
        return
    }

    // 如果有调试信息请求
    if *debug != "" {
        printDebugInfo(*debug)
        return
    }

    // 处理系统信号以优雅关闭
    go func() {
        sigChan := make(chan os.Signal, 1)
        signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
        sig := <-sigChan
        log.Printf("接收到信号: %v，正在关闭...\n", sig)
        shutdownChan <- struct{}{}
    }()

    if *foreground {
        log.Println("运行前台模式")
        runScheduler()
        loadTasks()
    } else if *inotify {
        runWithoutInotify()
    } else {
        // 默认模式: 初始化调度器后加载任务
        log.Println("启动 UniCrond")
        log.Println("酷安@LIghtJUNction")
        runScheduler() // 初始化调度器
        loadTasks()    // 加载任务
    }

    // 阻塞主线程，保持守护进程运行，直到收到关闭信号
    <-shutdownChan
    log.Println("UniCrond 正在关闭...")
    cronScheduler.Stop()
    os.Exit(0)
}
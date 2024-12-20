package main

import (
    "bufio"
    "fmt"
    "github.com/fsnotify/fsnotify"
    "github.com/robfig/cron/v3"
    "github.com/spf13/cobra"
    "io"
    "log"
    "os"
    "os/exec"
    "os/signal"
    "path/filepath"
    "strings"
    "syscall"
    "time"
)

type daemon struct {
    cron    *cron.Cron
    logger  *log.Logger
    watcher *fsnotify.Watcher
}

const (
    VERSION = "1.0.0@LIghtJUNction-<lightjunction.me@gmail.com>"
    BASE = "/data/adb/modules"
)

var rootCmd = &cobra.Command{
    Use:   "unicrond",
    Short: "UniCrond - 模块定时任务管理",
    RunE: func(cmd *cobra.Command, args []string) error {
        if v, _ := cmd.Flags().GetBool("version"); v {
            fmt.Printf("v%s\n", VERSION)
            return nil
        }
        if s, _ := cmd.Flags().GetBool("stop"); s {
            return stopService()
        }
        if l, _ := cmd.Flags().GetBool("list"); l {
            return listTasks()
        }
        return newDaemon().run()
    },
}

func init() {
    rootCmd.Flags().BoolP("version", "v", false, "显示版本")
    rootCmd.Flags().BoolP("stop", "s", false, "停止服务")
    rootCmd.Flags().BoolP("list", "l", false, "显示任务")
}

func newDaemon() *daemon {
    d := &daemon{cron: cron.New()}
    d.logger = log.New(io.MultiWriter(os.Stdout), "", log.LstdFlags)
    return d
}

func (d *daemon) run() error {
    if err := d.setup(); err != nil {
        return err
    }
    defer d.cleanup()

    d.reloadJobs()
    go d.watch()

    sig := make(chan os.Signal, 1)
    signal.Notify(sig, syscall.SIGINT, syscall.SIGTERM)
    <-sig
    return nil
}

func (d *daemon) setup() error {
    if !createLock() {
        return fmt.Errorf("服务已运行")
    }
    
    var err error
    if d.watcher, err = fsnotify.NewWatcher(); err != nil {
        return err
    }

    return filepath.Walk(BASE, func(path string, info os.FileInfo, err error) error {
        if info != nil && info.IsDir() && (path == BASE || strings.HasSuffix(path, "UniCron")) {
            return d.watcher.Add(path)
        }
        return nil
    })
}

func (d *daemon) watch() {
    debounce := time.NewTicker(500 * time.Millisecond)
    defer debounce.Stop()
    
    var needReload bool
    for {
        select {
        case e := <-d.watcher.Events:
            if filepath.Ext(e.Name) == ".cron" || filepath.Base(e.Name) == "disable" {
                needReload = true
            }
        case <-debounce.C:
            if needReload {
                d.reloadJobs()
                needReload = false
            }
        }
    }
}

func (d *daemon) reloadJobs() {
    newCron := cron.New(cron.WithLogger(&cronLogger{logger: d.logger}))
    modules, _ := os.ReadDir(BASE)
    
    for _, mod := range modules {
        if !mod.IsDir() || isDisabled(mod.Name()) {
            continue
        }
        loadModuleJobs(mod.Name(), newCron, d.logger)
    }

    d.cron.Stop()
    d.cron = newCron
    d.cron.Start()
}

func loadModuleJobs(id string, c *cron.Cron, l *log.Logger) {
    file, err := os.Open(filepath.Join(BASE, id, "UniCron", id+".cron"))
    if err != nil {
        return
    }
    defer file.Close()

    scanner := bufio.NewScanner(file)
    for scanner.Scan() {
        if line := scanner.Text(); line != "" && line[0] != '#' {
            if fields := strings.Fields(line); len(fields) >= 6 {
                sched, cmd := strings.Join(fields[0:5], " "), strings.Join(fields[5:], " ")
                c.AddFunc(sched, func() {
                    if out, err := exec.Command("sh", "-c", cmd).CombinedOutput(); err != nil {
                        l.Printf("[%s] 失败: %v\n%s", id, err, out)
                    }
                })
            }
        }
    }
}

func (d *daemon) cleanup() {
    d.cron.Stop()
    d.watcher.Close()
    os.Remove(filepath.Join(BASE, "UniCron/unicron.lock"))
}

func createLock() bool {
    lock := filepath.Join(BASE, "UniCron/unicron.lock")
    os.MkdirAll(filepath.Dir(lock), 0755)
    if f, err := os.OpenFile(lock, os.O_CREATE|os.O_EXCL|os.O_WRONLY, 0644); err == nil {
        fmt.Fprintf(f, "%d", os.Getpid())
        f.Close()
        return true
    }
    return false
}

func isDisabled(id string) bool {
    _, err := os.Stat(filepath.Join(BASE, id, "disable"))
    return err == nil
}

func stopService() error {
    lock := filepath.Join(BASE, "UniCron/unicron.lock")
    data, err := os.ReadFile(lock)
    if err != nil {
        return fmt.Errorf("服务未运行")
    }

    var pid int
    if _, err := fmt.Sscanf(string(data), "%d", &pid); err != nil {
        return fmt.Errorf("无效的PID")
    }

    if proc, err := os.FindProcess(pid); err != nil {
        return fmt.Errorf("未找到进程")
    } else if err := proc.Signal(syscall.SIGTERM); err != nil {
        return fmt.Errorf("停止失败: %v", err)
    }
    return nil
}

type cronLogger struct {
    logger *log.Logger
}

func (l *cronLogger) Info(msg string, keysAndValues ...interface{}) {
    if !strings.Contains(msg, "<nil>") {
        l.logger.Print(msg)
    }
}

func (l *cronLogger) Error(err error, msg string, keysAndValues ...interface{}) {
    l.logger.Printf("错误: %s - %v", msg, err)
}

func listTasks() error {
    modules, err := os.ReadDir(BASE)
    if err != nil {
        return fmt.Errorf("读取模块目录失败")
    }

    fmt.Println("当前任务列表:")
    fmt.Println("----------------------------------------")

    for _, mod := range modules {
        if !mod.IsDir() || isDisabled(mod.Name()) {
            continue
        }

        if file, err := os.Open(filepath.Join(BASE, mod.Name(), "UniCron", mod.Name()+".cron")); err == nil {
            fmt.Printf("\n模块: %s\n", mod.Name())
            fmt.Println("----------------------------------------")
            
            s := bufio.NewScanner(file)
            for s.Scan() {
                if line := s.Text(); line != "" && line[0] != '#' {
                    if f := strings.Fields(line); len(f) >= 6 {
                        fmt.Printf("计划: %s\n命令: %s\n\n", 
                            strings.Join(f[0:5], " "), 
                            strings.Join(f[5:], " "))
                    }
                }
            }
            file.Close()
        }
    }
    return nil
}

func main() {
    if err := rootCmd.Execute(); err != nil {
        os.Exit(1)
    }
}
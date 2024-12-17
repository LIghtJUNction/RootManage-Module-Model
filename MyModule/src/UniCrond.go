package main

import (
	"flag"
	"fmt"
	"io"
	"log"
	"net"
	"os"
	"os/exec"
	"os/signal"
	"path/filepath"
	"strings"
	"sync"
	"syscall"
	"time"
)

var (
	runningTasks        = make(map[string]*time.Ticker)
	mutex               sync.Mutex
	lockFilePath        = "/data/local/tmp/unicrond.lock"
	logFile             *os.File
	ipcSocketPath       = "/data/local/tmp/unicrond.sock"
	unicrontDir         = "/data/adb/modules/UniCron/"
	shutdownChan        = make(chan struct{})
)

func init() {
	ensureDirExists(filepath.Dir(lockFilePath))
}

func initLogging() {
	disablePath := filepath.Join(unicrontDir, "disable")
	logsDir := filepath.Join(unicrontDir, "logs")
	logFilePath := filepath.Join(logsDir, "UniCron.log")

	if _, err := os.Stat(unicrontDir); os.IsNotExist(err) {
		log.Println("UniCron 目录不存在，不初始化日志。")
		return
	}

	if _, err := os.Stat(disablePath); err == nil {
		log.Println("UniCron 被禁用，不初始化日志。")
		return
	}

	ensureDirExists(logsDir)

	var err error
	logFile, err = os.OpenFile(logFilePath, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, 0666)
	if err != nil {
		log.Fatalf("无法打开日志文件: %v", err)
	}
	log.SetOutput(io.MultiWriter(os.Stdout, logFile))
	log.Println("日志记录已初始化。")
}

func ensureDirExists(path string) {
	if _, err := os.Stat(path); os.IsNotExist(err) {
		if err := os.MkdirAll(path, 0755); err != nil {
			log.Fatalf("无法创建目录 %s: %v", path, err)
		}
	}
}

func removeDisabledTasks() {
	baseDir := "/data/adb/modules"
	modules, err := os.ReadDir(baseDir)
	if err != nil {
		log.Printf("读取模块目录时出错: %v", err)
		return
	}

	mutex.Lock()
	defer mutex.Unlock()
	for _, module := range modules {
		if module.IsDir() && module.Name() != "UniCron" {
			modulePath := filepath.Join(baseDir, module.Name())
			disablePath := filepath.Join(modulePath, "disable")
			if _, err := os.Stat(disablePath); err == nil {
				if ticker, exists := runningTasks[module.Name()]; exists {
					ticker.Stop()
					delete(runningTasks, module.Name())
					log.Printf("移除被禁用模块的任务: %s", module.Name())
				}
			}
		}
	}
}

func listRunningTasks() string {
	mutex.Lock()
	defer mutex.Unlock()
	if len(runningTasks) == 0 {
		return "当前没有正在运行的任务。\n"
	}

	var builder strings.Builder
	builder.WriteString("正在运行的任务列表：\n")
	for task := range runningTasks {
		builder.WriteString(fmt.Sprintf("任务: %s\n", task))
	}
	return builder.String()
}

func loadTasks() {
	baseDir := "/data/adb/modules"
	newTaskCommands := sync.Map{}

	modules, err := os.ReadDir(baseDir)
	if err != nil {
		log.Printf("读取模块目录时出错: %v", err)
		return
	}

	for _, module := range modules {
		if !module.IsDir() || module.Name() == "UniCron" {
			continue
		}

		unicrontPath := filepath.Join(baseDir, module.Name(), "UniCron")
		files, err := os.ReadDir(unicrontPath)
		if err != nil {
			log.Printf("读取目录时出错: %v", err)
			continue
		}

		for _, file := range files {
			if !strings.HasSuffix(file.Name(), ".cron") {
				continue
			}

			loadCronFile(filepath.Join(unicrontPath, file.Name()), &newTaskCommands)
		}
	}

	mutex.Lock()
	defer mutex.Unlock()
	for cmd := range runningTasks {
		if _, ok := newTaskCommands.Load(cmd); !ok {
			log.Printf("移除不存在的任务: %s", cmd)
			runningTasks[cmd].Stop()
			delete(runningTasks, cmd)
		}
	}
}

func loadCronFile(filePath string, newTaskCommands *sync.Map) {
	content, err := os.ReadFile(filePath)
	if err != nil {
		log.Printf("无法读取 cron 文件 %s: %v", filePath, err)
		return
	}

	lines := strings.Split(string(content), "\n")
	for _, line := range lines {
		trimmedLine := strings.TrimSpace(line)
		if trimmedLine == "" || strings.HasPrefix(trimmedLine, "#") {
			continue
		}

		parts := strings.Fields(trimmedLine)
		if len(parts) < 6 {
			log.Printf("无效的 cron 表达式: %s", trimmedLine)
			continue
		}

		command := strings.Join(parts[5:], " ")
		spec := strings.Join(parts[:5], " ")

		newTaskCommands.Store(command, true)
		addTaskToSchedule(command, spec)
	}
}

func addTaskToSchedule(command, spec string) {
	if _, exists := runningTasks[command]; exists {
		log.Printf("任务已存在，跳过添加：%s", command)
		return
	}

	duration, err := time.ParseDuration(spec)
	if err != nil {
		log.Printf("无效的 cron 格式 '%s': %v", spec, err)
		return
	}

	ticker := time.NewTicker(duration)
	mutex.Lock()
	runningTasks[command] = ticker
	mutex.Unlock()

	go func(cmd string) {
		for range ticker.C {
			executeCommand(cmd)
		}
	}(command)

	log.Printf("添加任务到调度：%s", command)
}

func executeCommand(command string) {
	log.Printf("执行任务：%s", command)
	output, err := exec.Command("sh", "-c", command).CombinedOutput()
	if err != nil {
		log.Printf("执行命令 '%s' 失败: %v", command, err)
	}
	log.Printf("命令 '%s' 输出: %s", command, string(output))
}

func startIPC() {
	ensureDirExists(filepath.Dir(ipcSocketPath))
	if _, err := os.Stat(ipcSocketPath); err == nil {
		os.Remove(ipcSocketPath)
	}

	listener, err := net.Listen("unix", ipcSocketPath)
	if err != nil {
		log.Fatalf("无法启动 IPC 监听器: %v", err)
	}
	defer listener.Close()

	log.Println("IPC 监听器已启动。")
	os.Chmod(ipcSocketPath, 0666)

	for {
		conn, err := listener.Accept()
		if err != nil {
			log.Printf("接受连接时出错: %v", err)
			continue
		}
		go handleConnection(conn)
	}
}

func handleConnection(conn net.Conn) {
	defer conn.Close()
	buffer := make([]byte, 1024)
	n, err := conn.Read(buffer)
	if err != nil {
		log.Printf("读取数据时出错: %v", err)
		return
	}

	command := strings.TrimSpace(string(buffer[:n]))
	switch command {
	case "list":
		conn.Write([]byte(listRunningTasks()))
	case "shutdown":
		log.Println("收到关闭命令，正在关闭...")
		conn.Write([]byte("正在关闭 UniCrond\n"))
		shutdownChan <- struct{}{}
	default:
		conn.Write([]byte("未知命令\n"))
	}
}

func main() {
	version := flag.Bool("V", false, "print version and exit")
	listTasksFlag := flag.Bool("l", false, "Lists the tasks that are currently running")
	background := flag.Bool("b", false, "run in background")
	help := flag.Bool("h", false, "print this message")
	flag.Parse()

	if *help {
		fmt.Println("Usage: unicrond [options]")
		return
	}

	if *version {
		fmt.Println("UniCrond version 1.0")
		return
	}

	if *background {
		log.Println("以后台模式运行。")
		os.Exit(0)
	}

	initLogging()
	startIPC()
	loadTasks()

	if *listTasksFlag {
		fmt.Print(listRunningTasks())
		return
	}

	go func() {
		sigChan := make(chan os.Signal, 1)
		signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
		<-sigChan
		shutdownChan <- struct{}{}
	}()

	<-shutdownChan
	log.Println("UniCrond 正在关闭...")
}
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
	runningTasks = sync.Map{}
	mutex        sync.Mutex

	lockFilePath  = "/data/local/tmp/unicrond.lock"
	ipcSocketPath = "/data/local/tmp/unicrond.sock"
	unicrontDir   = "/data/adb/modules/UniCron/"
	shutdownChan  = make(chan struct{})
	logFile       *os.File
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

func acquireLock(lockFilePath string) (*os.File, error) {
	lockFile, err := os.OpenFile(lockFilePath, os.O_CREATE|os.O_RDWR, 0666)
	if err != nil {
		return nil, fmt.Errorf("无法打开锁文件: %v", err)
	}

	err = syscall.Flock(int(lockFile.Fd()), syscall.LOCK_EX|syscall.LOCK_NB)
	if err != nil {
		lockFile.Close()
		return nil, fmt.Errorf("无法获取锁: %v", err)
	}
	return lockFile, nil
}

func loadTasks() {
	baseDir := "/data/adb/modules"
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
			loadCronFile(filepath.Join(unicrontPath, file.Name()))
		}
	}
}

func loadCronFile(filePath string) {
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
		addTaskToSchedule(command)
	}
}

func addTaskToSchedule(command string) {
	if _, exists := runningTasks.Load(command); exists {
		log.Printf("任务已存在，跳过添加：%s", command)
		return
	}

	duration := 10 * time.Second // 示例，需替换为实际解析的调度规则
	ticker := time.NewTicker(duration)
	runningTasks.Store(command, ticker)

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
		response := listRunningTasks()
		conn.Write([]byte(response))
	case "shutdown":
		log.Println("收到关闭命令，正在关闭...")
		conn.Write([]byte("正在关闭 UniCrond\n"))
		shutdownChan <- struct{}{}
	default:
		conn.Write([]byte("未知命令\n"))
	}
}

func listRunningTasks() string {
	var builder strings.Builder
	runningTasks.Range(func(key, value any) bool {
		builder.WriteString(fmt.Sprintf("任务: %s\n", key))
		return true
	})
	return builder.String()
}

func main() {
	version := flag.Bool("V", false, "print version and exit")
	help := flag.Bool("h", false, "print this message")
	background := flag.Bool("b", false, "run in background")
	flag.Parse()

	if *help {
		fmt.Println("Usage: unicrond [options]")
		return
	}

	if *version {
		fmt.Println("UniCrond version 1.0")
		return
	}

	lockFile, err := acquireLock(lockFilePath)
	if err != nil {
		log.Fatalf("无法启动 UniCrond: %v", err)
	}
	defer lockFile.Close()

	initLogging()
	startIPC()
	loadTasks()

	if *background {
		log.Println("以后台模式运行")
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
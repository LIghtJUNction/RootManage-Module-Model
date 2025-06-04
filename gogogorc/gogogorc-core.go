package main

import (
	"fmt"
	"os"
)

func main() {
	// 检查是否以 root 权限运行（在 Windows 上暂时跳过这个检查用于测试）
	if os.Geteuid() != 0 && os.Getenv("OS") != "Windows_NT" {
		fmt.Fprintln(os.Stderr, "This program must run as root")
		os.Exit(1)
	}

	// 获取当前的 PATH
	oldPath := os.Getenv("PATH") // 读取 /data/adb/modules/gogogo/gogogo.dev，默认值为 "0"
	devMode := "0"
	if envPath, err := os.ReadFile("/data/adb/modules/gogogo/gogogo.dev"); err == nil {
		// 手动去除空白字符，避免 strings.TrimSpace
		data := envPath
		for len(data) > 0 && (data[len(data)-1] == '\n' || data[len(data)-1] == '\r' || data[len(data)-1] == ' ' || data[len(data)-1] == '\t') {
			data = data[:len(data)-1]
		}
		for len(data) > 0 && (data[0] == '\n' || data[0] == '\r' || data[0] == ' ' || data[0] == '\t') {
			data = data[1:]
		}
		devMode = string(data)
	}

	// 只输出 PATH
	outputPathOnly(oldPath, devMode)
}

// 输出 PATH 环境变量
func outputPathOnly(oldPath, devMode string) {
	if devMode == "1" {
		// 开发者模式
		addPaths := "/data/adb/modules/gogogo/GOXBIN:/data/adb/modules/gogogo/GOXBIN" // 开发者专属路径
		newPath := sortPath(oldPath, addPaths)
		fmt.Printf("PATH='%s'\n", newPath)
	} else {
		// 默认模式
		addPaths := "/data/adb/modules/gogogo/GOBIN" // 默认路径
		newPath := sortPath(oldPath, addPaths)
		fmt.Printf("PATH='%s'\n", newPath)
	}
}

// 极端性能优化版本 - 目标<5ms执行时间
func sortPath(oldPath, addPaths string) string {
	if oldPath == "" && addPaths == "" {
		return ""
	}

	// 单路径快速处理
	if addPaths != "" && oldPath == "" {
		return addPaths
	}
	if addPaths == "" {
		return oldPath
	}

	// 使用字节 slice 减少内存分配
	pathBytes := make([]byte, 0, len(oldPath)+len(addPaths)+10)
	seen := make(map[string]bool, 64) // 布尔值比空结构体更快

	// 添加新路径（最高优先级）
	addStart := 0
	for i := 0; i <= len(addPaths); i++ {
		if i == len(addPaths) || addPaths[i] == ':' {
			if i > addStart {
				path := addPaths[addStart:i]
				if !seen[path] {
					if len(pathBytes) > 0 {
						pathBytes = append(pathBytes, ':')
					}
					pathBytes = append(pathBytes, path...)
					seen[path] = true
				}
			}
			addStart = i + 1
		}
	}

	// 处理原有路径 - 分两轮避免多次扫描
	// 第一轮：添加非 /0/ 路径
	oldStart := 0
	for i := 0; i <= len(oldPath); i++ {
		if i == len(oldPath) || oldPath[i] == ':' {
			if i > oldStart {
				path := oldPath[oldStart:i]
				if !seen[path] && !(len(path) > 3 && path[1] == '0' && path[2] == '/') {
					if len(pathBytes) > 0 {
						pathBytes = append(pathBytes, ':')
					}
					pathBytes = append(pathBytes, path...)
					seen[path] = true
				}
			}
			oldStart = i + 1
		}
	}

	// 第二轮：添加 /0/ 路径（最低优先级）
	oldStart = 0
	for i := 0; i <= len(oldPath); i++ {
		if i == len(oldPath) || oldPath[i] == ':' {
			if i > oldStart {
				path := oldPath[oldStart:i]
				if !seen[path] && len(path) > 3 && path[1] == '0' && path[2] == '/' {
					if len(pathBytes) > 0 {
						pathBytes = append(pathBytes, ':')
					}
					pathBytes = append(pathBytes, path...)
					seen[path] = true
				}
			}
			oldStart = i + 1
		}
	}

	return string(pathBytes)
}

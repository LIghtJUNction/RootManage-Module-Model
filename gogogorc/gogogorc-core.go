package main

import (
	"fmt"
	"os"
	"strings"
)

func main() {
	// 检查是否以 root 权限运行
	if os.Geteuid() != 0 {
		fmt.Fprintln(os.Stderr, "This program must run as root")
		os.Exit(1)
	}

	// 输出 export 语句供 shell 脚本使用
	fmt.Printf("export GOROOT='%s'\n", "/data/adb/modules/gogogo/GOROOT")
	fmt.Printf("export GOCACHE='%s'\n", "/data/adb/modules/gogogo/GOCACHE")
	fmt.Printf("export GOENV='%s'\n", "/data/adb/modules/gogogo/gogogo.env")
	fmt.Printf("export GOBIN='%s'\n", "/data/adb/modules/gogogo/GOBIN")
	fmt.Printf("export GOTMPDIR='%s'\n", "/data/adb/modules/gogogo/GOTMP")
	fmt.Printf("export GO111MODULE='%s'\n", "on")

	// 获取当前的 PATH
	oldPath := os.Getenv("PATH")

	// 读取 /data/adb/modules/gogogo/gogogo.dev
	if envPath, err := os.ReadFile("/data/adb/modules/gogogo/gogogo.dev"); err == nil {
		// 如果值为 1，则加载开发者 XBIN 路径
		if strings.TrimSpace(string(envPath)) == "1" {
			addPaths := "/data/adb/modules/gogogo/GOXBIN:/data/adb/modules/gogogo/GOXBIN" // 开发者专属路径
			newPath := setupPath(oldPath, addPaths)
			fmt.Printf("export PATH='%s'\n", newPath)
		} else {
			// 如果 gogogo.dev 存在但值不是 1，使用默认 PATH
			addPaths := "/data/adb/modules/gogogo/GOBIN" // 默认路径
			newPath := setupPath(oldPath, addPaths)
			fmt.Printf("export PATH='%s'\n", newPath)
		}
	} else {
		// 如果 gogogo.dev 文件不存在，使用默认 PATH
		fmt.Fprintf(os.Stderr, "Error reading gogogo.dev: %v\n", err)
		addPaths := "/data/adb/modules/gogogo/GOBIN" // 默认路径
		newPath := setupPath(oldPath, addPaths)
		fmt.Printf("export PATH='%s'\n", newPath)
	}

}

// 高效的 PATH 设置函数 - 等效于 shell 脚本中的 setup_path
func setupPath(oldPath, addPaths string) string {
	var newPath string

	// 创建带分隔符的路径字符串用于快速查询
	pathsWithSep := ":" + oldPath + ":"

	// 1. 优先添加 /system/bin (如存在)
	if strings.Contains(pathsWithSep, ":/system/bin:") {
		newPath = "/system/bin"
		// 在已处理列表中标记
		pathsWithSep = strings.Replace(pathsWithSep, ":/system/bin:", ":DONE:", 1)
	}

	// 2. 添加非 /0/ 路径
	paths := strings.Split(oldPath, ":")
	for _, p := range paths {
		// 跳过空路径和已处理路径
		if p == "" {
			continue
		}
		if !strings.Contains(pathsWithSep, ":"+p+":") {
			continue
		}

		// 跳过 /0/ 路径 (稍后处理)
		if strings.Contains(p, "/0/") {
			continue
		}

		// 添加到新路径
		if newPath == "" {
			newPath = p
		} else {
			newPath = newPath + ":" + p
		}
		// 标记为已处理
		pathsWithSep = strings.Replace(pathsWithSep, ":"+p+":", ":DONE:", 1)
	}

	// 3. 添加新路径 (如果不存在)
	addPathsList := strings.Split(addPaths, ":")
	for _, p := range addPathsList {
		if p == "" {
			continue
		}
		if !strings.Contains(pathsWithSep, ":"+p+":") {
			if newPath == "" {
				newPath = p
			} else {
				newPath = newPath + ":" + p
			}
			pathsWithSep = pathsWithSep + ":" + p + ":"
		}
	}

	// 4. 最后添加 /0/ 目录路径
	for _, p := range paths {
		if p == "" {
			continue
		}
		if strings.Contains(p, "/0/") {
			// 检查是否未处理
			if strings.Contains(pathsWithSep, ":"+p+":") && !strings.Contains(pathsWithSep, ":DONE:") {
				if newPath == "" {
					newPath = p
				} else {
					newPath = newPath + ":" + p
				}
			}
		}
	}

	// 清理多余的冒号
	newPath = strings.Trim(newPath, ":")
	return newPath
}

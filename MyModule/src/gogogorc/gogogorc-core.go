package main

import (
	"fmt"
	"os"
	"strings"
)

func main() {
	// 检查环境变量是否已设置，避免重复执行
	if os.Getenv("GOGOGO_ENV_LOADED") == "1" {
		// 环境已加载，直接返回
		// Go无法完全模拟shell的return，但可以通过os.Exit(0)来退出
		os.Exit(0)
	}

	fmt.Println("正在加载 必需 环境变量...")

	// Go 环境变量设置
	os.Setenv("GOROOT", "/data/adb/modules/gogogo/GOROOT")
	os.Setenv("GOPATH", "/data/adb/modules/gogogo/go")
	os.Setenv("GOCACHE", "/data/adb/modules/gogogo/GOCACHE")
	os.Setenv("GOENV", "/data/adb/modules/gogogo/gogogo.env")
	os.Setenv("GOTELEMETRYDIR", "/data/adb/modules/gogogo/GOTELEMETRYDIR")
	os.Setenv("GOTMPDIR", "/data/adb/modules/gogogo/tmp")
	os.Setenv("GOMODCACHE", "/data/adb/modules/gogogo/go/pkg/mod")
	os.Setenv("GO111MODULE", "on")

	// 添加Go相关路径到PATH
	oldPath := os.Getenv("PATH")
	addPaths := "/data/adb/modules/gogogo/GOROOT/bin:/data/adb/modules/gogogo/go/bin"
	newPath := setupPath(oldPath, addPaths)
	os.Setenv("PATH", newPath)

	// 设置标志表明环境已加载
	os.Setenv("GOGOGO_ENV_LOADED", "1")
}

// 高效的 PATH 设置函数 - 等效于shell脚本中的setup_path
func setupPath(oldPath, addPaths string) string {
	var newPath string

	// 创建带分隔符的路径字符串用于快速查询
	pathsWithSep := ":" + oldPath + ":"

	// 1. 优先添加 /system/bin (如存在)
	if !strings.Contains(pathsWithSep, ":/system/bin:") == false {
		newPath = "/system/bin"
		// 在已处理列表中标记
		pathsWithSep = strings.Replace(pathsWithSep, ":/system/bin:", ":DONE:", 1)
	}

	// 2. 添加非/0/路径
	paths := strings.Split(oldPath, ":")
	for _, p := range paths {
		// 跳过空路径和已处理路径
		if p == "" {
			continue
		}
		if strings.Contains(pathsWithSep, ":"+p+":") == false {
			continue
		}

		// 跳过/0/路径(稍后处理)
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

	// 3. 添加新Go路径(如果不存在)
	addPathsList := strings.Split(addPaths, ":")
	for _, p := range addPathsList {
		if p == "" {
			continue
		}
		if strings.Contains(pathsWithSep, ":"+p+":") == true {
			if newPath == "" {
				newPath = p
			} else {
				newPath = newPath + ":" + p
			}
			pathsWithSep = pathsWithSep + p + ":DONE:"
		}
	}

	// 4. 最后添加/0/目录路径
	for _, p := range paths {
		if p == "" {
			continue
		}
		if strings.Contains(p, "/0/") {
			// 检查是否未处理
			if strings.Contains(pathsWithSep, ":"+p+":") == false && strings.Contains(pathsWithSep, ":DONE:") == false {
				if newPath == "" {
					newPath = p
				} else {
					newPath = newPath + ":" + p
				}
			}
		}
	}

	return newPath
}

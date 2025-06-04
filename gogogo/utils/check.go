package utils

import (
	"fmt"
	"log/slog"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/lightjunction/rootmanager-module-model/gogogo/config"
)

// CheckAndroidEnvironment 检查Android环境并设置GOENV
func CheckAndroidEnvironment(logger *slog.Logger) {
	if runtime.GOOS == "android" {
		goenvPath := "/data/adb/modules/gogogo/go.env"
		if _, err := os.Stat(goenvPath); err == nil {
			os.Setenv("GOENV", goenvPath)
			logger.Info("检测到Android环境，已设置GOENV", "path", goenvPath)
		}
	}
}

// CheckGoEnvironment 检查Go环境
func CheckGoEnvironment() error {
	// 检查go命令
	if _, err := exec.LookPath("go"); err != nil {
		return fmt.Errorf("未找到go命令，请确保Go已正确安装并添加到PATH")
	}

	// 获取Go版本
	cmd := exec.Command("go", "version")
	_, err := cmd.Output()
	if err != nil {
		return fmt.Errorf("无法获取Go版本: %v", err)
	}

	return nil
}

// DetectNDKType 检测NDK的类型 (Windows/Linux/Mac)
func DetectNDKType(ndkPath string) string {
	// 首先确认NDK路径是否有效
	if _, err := os.Stat(ndkPath); os.IsNotExist(err) {
		return ""
	}

	// 默认使用当前操作系统类型
	defaultType := runtime.GOOS
	if defaultType == "windows" && !strings.Contains(ndkPath, "windows") {
		// Windows上可能有Linux NDK，此时不应使用windows作为defaultType
		defaultType = ""
	}

	// 检查toolchains目录下的预编译工具目录
	toolchainsPath := filepath.Join(ndkPath, "toolchains", "llvm", "prebuilt")
	if _, err := os.Stat(toolchainsPath); os.IsNotExist(err) {
		// 尝试查找旧的NDK目录结构
		files, err := os.ReadDir(ndkPath)
		if err != nil {
			return defaultType // 如果读取失败，使用默认值
		}

		// 查找含有"windows"、"linux"或"darwin"的目录名
		for _, f := range files {
			if f.IsDir() {
				name := strings.ToLower(f.Name())
				if strings.Contains(name, "windows") {
					return "windows"
				}
				if strings.Contains(name, "linux") {
					return "linux"
				}
				if strings.Contains(name, "darwin") || strings.Contains(name, "mac") {
					return "darwin"
				}
			}
		}
		return defaultType // 找不到匹配的目录名，使用默认值
	}

	// 检查现代NDK结构
	files, err := os.ReadDir(toolchainsPath)
	if err != nil {
		return defaultType // 如果读取失败，使用默认值
	}

	// 查找预编译目录
	for _, f := range files {
		if f.IsDir() {
			name := strings.ToLower(f.Name())
			if strings.Contains(name, "windows") {
				return "windows"
			}
			if strings.Contains(name, "linux") {
				return "linux"
			}
			if strings.Contains(name, "darwin") || strings.Contains(name, "mac") {
				return "darwin"
			}
			// 可能是某些新版本NDK使用不同的命名方式
			if strings.Contains(name, "x86_64") || strings.Contains(name, "x86-64") {
				return runtime.GOOS // 使用当前操作系统类型
			}
		}
	}

	return ""
}

// DetectClangInstallationType 检测clang安装类型
func DetectClangInstallationType(clangPath string) string {
	clangPath = strings.ToLower(clangPath)

	if strings.Contains(clangPath, "xcode") || strings.Contains(clangPath, "/usr/bin/clang") {
		return "Xcode"
	}
	if strings.Contains(clangPath, "llvm") {
		return "LLVM"
	}
	if strings.Contains(clangPath, "mingw") || strings.Contains(clangPath, "msys") {
		return "MinGW"
	}
	if strings.Contains(clangPath, "visual studio") || strings.Contains(clangPath, "microsoft") {
		return "Visual Studio"
	}
	if strings.Contains(clangPath, "git") && strings.Contains(clangPath, "usr/bin") {
		return "Git for Windows"
	}

	return "Unknown"
}

// IsValidNDKDir 检查目录是否是有效的NDK根目录
func IsValidNDKDir(ndkPath string) bool {
	if ndkPath == "" {
		return false
	}

	// 检查NDK根目录是否存在
	if _, err := os.Stat(ndkPath); os.IsNotExist(err) {
		return false
	}

	// 检查关键文件和目录
	requiredPaths := []string{
		filepath.Join(ndkPath, "source.properties"),
		filepath.Join(ndkPath, "toolchains"),
	}

	for _, path := range requiredPaths {
		if _, err := os.Stat(path); os.IsNotExist(err) {
			return false
		}
	}

	return true
}

// IsValidClangInstallation 检查clang安装是否有效
func IsValidClangInstallation(installation config.ClangInstallation) bool {
	// 检查clang可执行文件是否存在
	if _, err := os.Stat(installation.Path); os.IsNotExist(err) {
		return false
	}

	// 检查clang是否可以正常执行
	cmd := exec.Command(installation.Path, "--version")
	if err := cmd.Run(); err != nil {
		return false
	}

	return true
}

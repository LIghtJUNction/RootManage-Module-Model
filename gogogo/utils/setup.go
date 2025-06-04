package utils

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"
)

// SetupNDKEnvironment 为Android NDK设置环境变量
func SetupNDKEnvironment(ndkPath string, arch string, cmdEnv *[]string) error {
	// 检测NDK类型
	ndkType := DetectNDKType(ndkPath)
	if ndkType == "" {
		return fmt.Errorf("无法确定NDK类型")
	}

	// 根据宿主系统类型和NDK类型设置不同的环境变量
	hostOS := runtime.GOOS

	prebuiltPath := GetNDKPrebuiltPath(ndkPath, ndkType)
	if prebuiltPath == "" {
		return fmt.Errorf("无法找到NDK预编译工具路径")
	}

	// NDK基本环境变量
	*cmdEnv = append(*cmdEnv, "ANDROID_NDK_HOME="+ndkPath)
	*cmdEnv = append(*cmdEnv, "ANDROID_NDK_ROOT="+ndkPath)

	// 为不同的宿主系统和NDK类型设置特定的环境变量
	if hostOS == "windows" {
		// Windows宿主
		*cmdEnv = append(*cmdEnv, "CGO_CFLAGS=-I"+filepath.Join(prebuiltPath, "sysroot", "usr", "include"))
		*cmdEnv = append(*cmdEnv, "CGO_LDFLAGS=-L"+filepath.Join(prebuiltPath, "sysroot", "usr", "lib"))
	} else if hostOS == "linux" {
		// Linux宿主
		*cmdEnv = append(*cmdEnv, "CGO_CFLAGS=-I"+filepath.Join(prebuiltPath, "sysroot", "usr", "include"))
		*cmdEnv = append(*cmdEnv, "CGO_LDFLAGS=-L"+filepath.Join(prebuiltPath, "sysroot", "usr", "lib"))
	} else if hostOS == "darwin" {
		// macOS宿主
		*cmdEnv = append(*cmdEnv, "CGO_CFLAGS=-I"+filepath.Join(prebuiltPath, "sysroot", "usr", "include"))
		*cmdEnv = append(*cmdEnv, "CGO_LDFLAGS=-L"+filepath.Join(prebuiltPath, "sysroot", "usr", "lib"))
	}

	// 设置架构特定的环境变量
	switch arch {
	case "arm":
		*cmdEnv = append(*cmdEnv, "CC="+filepath.Join(prebuiltPath, "bin", "armv7a-linux-androideabi21-clang"))
		*cmdEnv = append(*cmdEnv, "CXX="+filepath.Join(prebuiltPath, "bin", "armv7a-linux-androideabi21-clang++"))
	case "arm64":
		*cmdEnv = append(*cmdEnv, "CC="+filepath.Join(prebuiltPath, "bin", "aarch64-linux-android21-clang"))
		*cmdEnv = append(*cmdEnv, "CXX="+filepath.Join(prebuiltPath, "bin", "aarch64-linux-android21-clang++"))
	case "386":
		*cmdEnv = append(*cmdEnv, "CC="+filepath.Join(prebuiltPath, "bin", "i686-linux-android21-clang"))
		*cmdEnv = append(*cmdEnv, "CXX="+filepath.Join(prebuiltPath, "bin", "i686-linux-android21-clang++"))
	case "amd64":
		*cmdEnv = append(*cmdEnv, "CC="+filepath.Join(prebuiltPath, "bin", "x86_64-linux-android21-clang"))
		*cmdEnv = append(*cmdEnv, "CXX="+filepath.Join(prebuiltPath, "bin", "x86_64-linux-android21-clang++"))
	}

	return nil
}

// GetNDKPrebuiltPath 获取NDK预编译工具的路径
func GetNDKPrebuiltPath(ndkPath string, ndkType string) string {
	// 标准路径结构: toolchains/llvm/prebuilt/OS-ARCH
	baseDir := filepath.Join(ndkPath, "toolchains", "llvm", "prebuilt")
	if _, err := os.Stat(baseDir); os.IsNotExist(err) {
		return ""
	}
	files, err := os.ReadDir(baseDir)
	if err != nil {
		return ""
	}

	// 首先尝试查找完全匹配的目录
	for _, f := range files {
		if f.IsDir() {
			name := strings.ToLower(f.Name())
			if strings.HasPrefix(name, ndkType) {
				return filepath.Join(baseDir, f.Name())
			}
		}
	}

	// 如果没有完全匹配，返回任意一个目录
	if len(files) > 0 {
		for _, f := range files {
			if f.IsDir() {
				return filepath.Join(baseDir, f.Name())
			}
		}
	}

	return ""
}

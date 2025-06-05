package utils

import (
	"fmt"
	"log/slog"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/fatih/color"
)

// SetupNDKEnvironment 为Android NDK设置环境变量
func SetupNDKEnvironment(ndkPath string, arch string, cmdEnv *[]string, logger *slog.Logger) error {
	// 定义颜色
	colorTitle := color.New(color.FgHiCyan, color.Bold)
	colorSuccess := color.New(color.FgHiGreen, color.Bold)
	colorWarning := color.New(color.FgHiYellow, color.Bold)
	colorError := color.New(color.FgHiRed, color.Bold)
	colorInfo := color.New(color.FgHiBlue)
	colorPath := color.New(color.FgHiCyan)
	colorArch := color.New(color.FgHiMagenta, color.Bold)

	if logger != nil {
		logger.Debug("开始设置NDK环境", "ndkPath", ndkPath, "arch", arch)
		// 美化输出NDK设置开始信息
		fmt.Print("\n")
		colorTitle.Printf("🔧 设置Android NDK环境\n")
		fmt.Println(strings.Repeat("─", 40))
		colorInfo.Printf("📁 NDK路径: ")
		colorPath.Printf("%s\n", ndkPath)
		colorInfo.Printf("🏗️  目标架构: ")
		colorArch.Printf("%s\n", arch)
		fmt.Println(strings.Repeat("─", 40))
	} // 先清除可能存在的旧的Android相关环境变量，避免冲突
	cleanedEnv := make([]string, 0, len(*cmdEnv))
	removedCount := 0
	for _, env := range *cmdEnv {
		if strings.HasPrefix(env, "ANDROID_") ||
			strings.HasPrefix(env, "CC=") ||
			strings.HasPrefix(env, "CXX=") ||
			strings.HasPrefix(env, "CGO_CFLAGS=") ||
			strings.HasPrefix(env, "CGO_LDFLAGS=") {
			if logger != nil {
				logger.Debug("清理旧的环境变量", "var", env)
			}
			removedCount++
		} else {
			cleanedEnv = append(cleanedEnv, env)
		}
	}
	*cmdEnv = cleanedEnv

	if logger != nil && removedCount > 0 {
		logger.Info("清理了旧的Android/编译器环境变量", "count", removedCount)
		colorInfo.Printf("🧹 清理了 %d 个旧的环境变量\n", removedCount)
	} // 检测NDK类型
	ndkType := DetectNDKType(ndkPath)
	if ndkType == "" {
		// 使用当前操作系统作为默认类型
		ndkType = runtime.GOOS
		if logger != nil {
			logger.Warn("无法检测NDK类型，使用默认值", "type", ndkType)
			colorWarning.Printf("⚠️  无法检测NDK类型，使用默认值: ")
			colorArch.Printf("%s\n", ndkType)
		}
	} else if logger != nil {
		logger.Debug("检测到NDK类型", "type", ndkType)
		colorSuccess.Printf("✓ 检测到NDK类型: ")
		colorArch.Printf("%s\n", ndkType)
	}
	// 根据宿主系统类型和NDK类型设置不同的环境变量
	hostOS := runtime.GOOS
	prebuiltPath := GetNDKPrebuiltPath(ndkPath, ndkType)
	if prebuiltPath == "" {
		// 尝试使用基本路径
		prebuiltPath = filepath.Join(ndkPath, "toolchains", "llvm")
		if _, err := os.Stat(prebuiltPath); os.IsNotExist(err) {
			if logger != nil {
				logger.Error("无法找到NDK预编译工具路径", "path", prebuiltPath)
				colorError.Printf("❌ 无法找到NDK预编译工具路径: ")
				colorPath.Printf("%s\n", prebuiltPath)
			}
			return fmt.Errorf("无法找到NDK预编译工具路径")
		}
	}

	if logger != nil {
		logger.Debug("使用预编译工具路径", "path", prebuiltPath)
		colorSuccess.Printf("✓ 使用预编译工具路径: ")
		colorPath.Printf("%s\n", prebuiltPath)
	}

	// NDK基本环境变量
	*cmdEnv = append(*cmdEnv, "ANDROID_NDK_HOME="+ndkPath)
	*cmdEnv = append(*cmdEnv, "ANDROID_NDK_ROOT="+ndkPath)

	// 检查 sysroot 目录是否存在
	sysrootPath := filepath.Join(prebuiltPath, "sysroot")
	if _, err := os.Stat(sysrootPath); os.IsNotExist(err) {
		// 尝试查找替代的 sysroot 路径
		altSysrootPath := filepath.Join(ndkPath, "sysroot")
		if _, err := os.Stat(altSysrootPath); err == nil {
			sysrootPath = altSysrootPath
		} else {
			// 可能是新版NDK，使用不同的目录结构
			*cmdEnv = append(*cmdEnv, "CGO_CFLAGS=-I"+filepath.Join(ndkPath, "toolchains", "llvm", "prebuilt", hostOS+"-x86_64", "sysroot", "usr", "include"))
			*cmdEnv = append(*cmdEnv, "CGO_LDFLAGS=-L"+filepath.Join(ndkPath, "toolchains", "llvm", "prebuilt", hostOS+"-x86_64", "sysroot", "usr", "lib"))
			goto setup_compilers // 跳到设置编译器部分
		}
	}

	// 为不同的宿主系统和NDK类型设置特定的环境变量
	*cmdEnv = append(*cmdEnv, "CGO_CFLAGS=-I"+filepath.Join(sysrootPath, "usr", "include"))
	*cmdEnv = append(*cmdEnv, "CGO_LDFLAGS=-L"+filepath.Join(sysrootPath, "usr", "lib"))

setup_compilers:
	// 设置架构特定的环境变量
	binDir := filepath.Join(prebuiltPath, "bin")
	apiLevel := "21" // Android 5.0+, 设为最低兼容版本

	// 检查bin目录是否存在
	if _, err := os.Stat(binDir); os.IsNotExist(err) {
		// 尝试查找替代路径
		binDir = filepath.Join(ndkPath, "toolchains", "llvm", "prebuilt", hostOS+"-x86_64", "bin")
		if _, err := os.Stat(binDir); os.IsNotExist(err) {
			// 如果还是找不到，尝试最后一次
			binDir = filepath.Join(ndkPath, "prebuilt", hostOS+"-x86_64", "bin")
			if _, err := os.Stat(binDir); os.IsNotExist(err) {
				// 如果目录仍然不存在，返回错误
				return fmt.Errorf("无法找到编译器目录")
			}
		}
	}

	var ccName, cxxName string
	// 不同架构的编译器名称
	switch arch {
	case "arm":
		ccName = fmt.Sprintf("armv7a-linux-androideabi%s-clang", apiLevel)
		cxxName = fmt.Sprintf("armv7a-linux-androideabi%s-clang++", apiLevel)
	case "arm64":
		ccName = fmt.Sprintf("aarch64-linux-android%s-clang", apiLevel)
		cxxName = fmt.Sprintf("aarch64-linux-android%s-clang++", apiLevel)
	case "386":
		ccName = fmt.Sprintf("i686-linux-android%s-clang", apiLevel)
		cxxName = fmt.Sprintf("i686-linux-android%s-clang++", apiLevel)
	case "amd64":
		ccName = fmt.Sprintf("x86_64-linux-android%s-clang", apiLevel)
		cxxName = fmt.Sprintf("x86_64-linux-android%s-clang++", apiLevel)
	default:
		return fmt.Errorf("不支持的架构: %s", arch)
	}
	if runtime.GOOS == "windows" {
		// 先检查是否存在不带后缀的文件
		ccPath := filepath.Join(binDir, ccName)
		if _, err := os.Stat(ccPath); os.IsNotExist(err) {
			// 检查.cmd后缀
			if _, err := os.Stat(ccPath + ".cmd"); err == nil {
				ccName += ".cmd"
				cxxName += ".cmd"
			} else if _, err := os.Stat(ccPath + ".exe"); err == nil {
				// 检查.exe后缀
				ccName += ".exe"
				cxxName += ".exe"
			}
		}
	}
	ccPath := filepath.Join(binDir, ccName)
	cxxPath := filepath.Join(binDir, cxxName)

	*cmdEnv = append(*cmdEnv, "CC="+ccPath)
	*cmdEnv = append(*cmdEnv, "CXX="+cxxPath)

	if logger != nil {
		logger.Debug("设置编译器路径", "CC", ccPath, "CXX", cxxPath)
		// 美化输出编译器设置信息
		colorInfo.Printf("🔨 设置编译器路径:\n")
		colorInfo.Printf("  • CC: ")
		colorPath.Printf("%s\n", ccPath)
		colorInfo.Printf("  • CXX: ")
		colorPath.Printf("%s\n", cxxPath)

		// 打印最终环境变量，便于调试
		logger.Debug("NDK环境变量设置完成",
			"ANDROID_NDK_HOME", ndkPath,
			"CC", ccPath,
			"CXX", cxxPath,
			"arch", arch)

		// 检查编译器文件是否存在
		if _, err := os.Stat(ccPath); os.IsNotExist(err) {
			logger.Error("CC编译器文件不存在", "path", ccPath)
			colorError.Printf("❌ CC编译器文件不存在: ")
			colorPath.Printf("%s\n", ccPath)
		} else {
			colorSuccess.Printf("✓ CC编译器文件验证通过\n")
		}
		if _, err := os.Stat(cxxPath); os.IsNotExist(err) {
			logger.Error("CXX编译器文件不存在", "path", cxxPath)
			colorError.Printf("❌ CXX编译器文件不存在: ")
			colorPath.Printf("%s\n", cxxPath)
		} else {
			colorSuccess.Printf("✓ CXX编译器文件验证通过\n")
		}

		// 美化输出完成信息
		fmt.Println()
		colorSuccess.Printf("✅ NDK环境变量设置完成!\n")
		fmt.Println(strings.Repeat("─", 40))
	}

	return nil
}

// GetNDKPrebuiltPath 获取NDK预编译工具的路径
func GetNDKPrebuiltPath(ndkPath string, ndkType string) string {
	// 标准路径结构: toolchains/llvm/prebuilt/OS-ARCH
	baseDir := filepath.Join(ndkPath, "toolchains", "llvm", "prebuilt")
	if _, err := os.Stat(baseDir); os.IsNotExist(err) {
		// 尝试查找替代路径
		altPath := filepath.Join(ndkPath, "toolchains", "llvm")
		if _, err := os.Stat(altPath); err == nil {
			return altPath
		}
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

	// 尝试查找部分匹配
	for _, f := range files {
		if f.IsDir() {
			name := strings.ToLower(f.Name())
			switch ndkType {
			case "windows":
				if strings.Contains(name, "win") {
					return filepath.Join(baseDir, f.Name())
				}
			case "linux":
				if strings.Contains(name, "linux") || strings.Contains(name, "gnu") {
					return filepath.Join(baseDir, f.Name())
				}
			case "darwin":
				if strings.Contains(name, "darwin") || strings.Contains(name, "mac") || strings.Contains(name, "apple") {
					return filepath.Join(baseDir, f.Name())
				}
			}
		}
	}

	// 如果没有匹配，返回任意一个目录
	if len(files) > 0 {
		for _, f := range files {
			if f.IsDir() {
				return filepath.Join(baseDir, f.Name())
			}
		}
	}

	return ""
}

// PrintEnvironmentVars 打印环境变量，用于调试
func PrintEnvironmentVars(env []string, prefix string, logger *slog.Logger) {
	if logger == nil {
		return
	}

	// 颜色定义
	colorTitle := color.New(color.FgHiCyan, color.Bold)
	colorCategory := color.New(color.FgHiMagenta, color.Bold)
	colorVar := color.New(color.FgHiBlue)
	colorValue := color.New(color.FgHiGreen)
	colorWarning := color.New(color.FgHiYellow, color.Bold)
	colorError := color.New(color.FgHiRed, color.Bold)
	colorSuccess := color.New(color.FgHiGreen, color.Bold)

	// 将环境变量按类别分组打印
	androidVars := make([]string, 0)
	cgoVars := make([]string, 0)
	goVars := make([]string, 0)
	compilerVars := make([]string, 0)

	for _, e := range env {
		if strings.HasPrefix(e, "ANDROID_") {
			androidVars = append(androidVars, e)
		} else if strings.HasPrefix(e, "CGO_") {
			cgoVars = append(cgoVars, e)
		} else if strings.HasPrefix(e, "GO") {
			goVars = append(goVars, e)
		} else if strings.HasPrefix(e, "CC=") || strings.HasPrefix(e, "CXX=") {
			compilerVars = append(compilerVars, e)
		} else {
			// 忽略非关键环境变量以减少日志量
			continue
		}
	}

	// 美化输出标题
	fmt.Print("\n")
	colorTitle.Printf("🔍 %s\n", prefix)
	fmt.Println(strings.Repeat("─", 50))

	// 只打印有内容的分类
	if len(androidVars) > 0 {
		colorCategory.Printf("📱 Android环境变量:\n")
		for _, v := range androidVars {
			parts := strings.SplitN(v, "=", 2)
			if len(parts) == 2 {
				colorVar.Printf("  • %s", parts[0])
				fmt.Print("=")
				colorValue.Printf("%s\n", parts[1])
			}
		}
		fmt.Println()
		logger.Info(prefix+" Android环境变量", "vars", androidVars)
	}

	if len(cgoVars) > 0 {
		colorCategory.Printf("⚙️ CGO环境变量:\n")
		for _, v := range cgoVars {
			parts := strings.SplitN(v, "=", 2)
			if len(parts) == 2 {
				colorVar.Printf("  • %s", parts[0])
				fmt.Print("=")
				colorValue.Printf("%s\n", parts[1])
			}
		}
		fmt.Println()
		logger.Info(prefix+" CGO环境变量", "vars", cgoVars)
	}

	if len(goVars) > 0 {
		colorCategory.Printf("🐹 Go环境变量:\n")
		for _, v := range goVars {
			parts := strings.SplitN(v, "=", 2)
			if len(parts) == 2 {
				colorVar.Printf("  • %s", parts[0])
				fmt.Print("=")
				colorValue.Printf("%s\n", parts[1])
			}
		}
		fmt.Println()
		logger.Info(prefix+" Go环境变量", "vars", goVars)
	}

	if len(compilerVars) > 0 {
		colorCategory.Printf("🔨 编译器环境变量:\n")
		for _, v := range compilerVars {
			parts := strings.SplitN(v, "=", 2)
			if len(parts) == 2 {
				colorVar.Printf("  • %s", parts[0])
				fmt.Print("=")
				colorValue.Printf("%s\n", parts[1])
			}
		}
		fmt.Println()
		logger.Info(prefix+" 编译器环境变量", "vars", compilerVars)
	}

	// 检查是否有冲突的CGO_ENABLED设置
	hasCGOEnabled0 := false
	hasCGOEnabled1 := false
	cgoEnabledCount := 0

	for _, e := range env {
		if e == "CGO_ENABLED=0" {
			hasCGOEnabled0 = true
			cgoEnabledCount++
		} else if e == "CGO_ENABLED=1" {
			hasCGOEnabled1 = true
			cgoEnabledCount++
		}
	}

	if hasCGOEnabled0 && hasCGOEnabled1 {
		colorError.Printf("❌ 检测到冲突的CGO_ENABLED设置!\n")
		colorWarning.Printf("   同时存在 CGO_ENABLED=0 和 CGO_ENABLED=1\n")
		logger.Error(prefix+" 检测到冲突的CGO_ENABLED设置", "CGO_ENABLED=0", hasCGOEnabled0, "CGO_ENABLED=1", hasCGOEnabled1)
	} else if cgoEnabledCount > 1 {
		colorWarning.Printf("⚠️  检测到多个相同的CGO_ENABLED设置 (数量: %d)\n", cgoEnabledCount)
		logger.Warn(prefix+" 检测到多个相同的CGO_ENABLED设置", "count", cgoEnabledCount)
	}

	// 检查编译器路径是否存在
	for _, e := range compilerVars {
		if strings.HasPrefix(e, "CC=") {
			ccPath := strings.TrimPrefix(e, "CC=")
			if _, err := os.Stat(ccPath); os.IsNotExist(err) {
				colorError.Printf("❌ CC编译器路径不存在: %s\n", ccPath)
				logger.Error(prefix+" CC编译器路径不存在", "path", ccPath)
			} else {
				colorSuccess.Printf("✓ CC编译器路径验证通过: %s\n", ccPath)
				logger.Debug(prefix+" CC编译器路径验证通过", "path", ccPath)
			}
		} else if strings.HasPrefix(e, "CXX=") {
			cxxPath := strings.TrimPrefix(e, "CXX=")
			if _, err := os.Stat(cxxPath); os.IsNotExist(err) {
				colorError.Printf("❌ CXX编译器路径不存在: %s\n", cxxPath)
				logger.Error(prefix+" CXX编译器路径不存在", "path", cxxPath)
			} else {
				colorSuccess.Printf("✓ CXX编译器路径验证通过: %s\n", cxxPath)
				logger.Debug(prefix+" CXX编译器路径验证通过", "path", cxxPath)
			}
		}
	}

	fmt.Println(strings.Repeat("─", 50))
}

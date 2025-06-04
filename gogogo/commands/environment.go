package commands

import (
	"fmt"
	"log/slog"
	"os"
	"os/exec"
	"strings"

	"github.com/fatih/color"
	"github.com/lightjunction/rootmanager-module-model/gogogo/utils"
)

// GetEnvironmentInfo 显示环境信息
func GetEnvironmentInfo(logger *slog.Logger) {
	colorTitle := color.New(color.FgHiCyan, color.Bold)
	colorSuccess := color.New(color.FgHiGreen)
	colorError := color.New(color.FgHiRed)
	colorWarning := color.New(color.FgHiYellow)
	colorBold := color.New(color.Bold)

	colorTitle.Println("🌍 编译环境信息:")

	// Go环境
	fmt.Println()
	colorBold.Println("Go环境:")
	if err := utils.CheckGoEnvironment(); err != nil {
		colorError.Printf("  ❌ %v\n", err)
	} else {
		colorSuccess.Println("  ✓ Go环境正常")

		// 显示Go版本和路径
		if goVersion, err := exec.Command("go", "version").Output(); err == nil {
			fmt.Printf("  版本: %s", strings.TrimSpace(string(goVersion)))
		}
		if goRoot := os.Getenv("GOROOT"); goRoot != "" {
			fmt.Printf("  GOROOT: %s\n", goRoot)
		}
		if goPath := os.Getenv("GOPATH"); goPath != "" {
			fmt.Printf("  GOPATH: %s\n", goPath)
		}
	}

	// NDK环境
	fmt.Println()
	colorBold.Println("Android NDK:")
	ndkPaths := utils.FindSystemNDK(logger)
	if len(ndkPaths) == 0 {
		colorWarning.Println("  ⚠️  未找到系统NDK安装")
	} else {
		colorSuccess.Printf("  ✓ 找到 %d 个NDK安装:\n", len(ndkPaths))
		for i, ndkPath := range ndkPaths {
			if i < 3 { // 只显示前3个
				ndkType := utils.DetectNDKType(ndkPath)
				fmt.Printf("    %d. %s (%s)\n", i+1, ndkPath, ndkType)
			}
		}
		if len(ndkPaths) > 3 {
			fmt.Printf("    ... 还有 %d 个NDK安装\n", len(ndkPaths)-3)
		}
	}

	// Clang环境
	fmt.Println()
	colorBold.Println("Clang编译器:")
	clangInstallations := utils.FindSystemClang(logger)
	if len(clangInstallations) == 0 {
		colorWarning.Println("  ⚠️  未找到Clang安装")
	} else {
		colorSuccess.Printf("  ✓ 找到 %d 个Clang安装:\n", len(clangInstallations))
		for i, installation := range clangInstallations {
			if i < 3 { // 只显示前3个
				fmt.Printf("    %d. %s (%s, v%s)\n", i+1, installation.Path, installation.Type, installation.Version)
			}
		}
		if len(clangInstallations) > 3 {
			fmt.Printf("    ... 还有 %d 个Clang安装\n", len(clangInstallations)-3)
		}
	}

	// 相关环境变量
	fmt.Println()
	colorBold.Println("相关环境变量:")
	envVars := []string{
		"ANDROID_HOME", "ANDROID_SDK_ROOT", "NDK_ROOT",
		"CC", "CXX", "CGO_ENABLED",
		"GOOS", "GOARCH", "CGO_CFLAGS", "CGO_LDFLAGS",
	}

	for _, envVar := range envVars {
		if value := os.Getenv(envVar); value != "" {
			fmt.Printf("  %s: %s\n", envVar, value)
		}
	}
}

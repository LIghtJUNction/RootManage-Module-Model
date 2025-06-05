package commands

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"

	"github.com/fatih/color"

	"github.com/lightjunction/rootmanager-module-model/gogogo/config"
	"github.com/lightjunction/rootmanager-module-model/gogogo/utils"
)

// RunInteractive 运行交互式编译模式
func RunInteractive(cfg *config.Config) error {
	colorTitle := color.New(color.FgHiCyan, color.Bold)
	colorBold := color.New(color.Bold)
	colorInfo := color.New(color.FgHiBlue)
	colorWarning := color.New(color.FgYellow)
	colorSuccess := color.New(color.FgGreen)

	colorTitle.Println("🔍 交互式编译模式")
	scanner := bufio.NewScanner(os.Stdin)

	// 源文件
	if cfg.SourceFile == "" {
		colorBold.Print("请输入源文件路径: ")
		if scanner.Scan() {
			sourceFile := strings.TrimSpace(scanner.Text())
			if sourceFile == "" {
				return fmt.Errorf("源文件路径不能为空")
			}
			cfg.SourceFile = sourceFile
		}
	}

	// 输出目录
	defaultOutput := cfg.OutputDir
	colorBold.Printf("输出目录 [%s]: ", defaultOutput)
	if scanner.Scan() {
		outputDir := strings.TrimSpace(scanner.Text())
		if outputDir != "" {
			cfg.OutputDir = outputDir
		}
	}

	// 二进制名称
	defaultName := cfg.BinaryName
	if defaultName == "" {
		defaultName = strings.TrimSuffix(filepath.Base(cfg.SourceFile), filepath.Ext(cfg.SourceFile))
	}
	colorBold.Printf("二进制名称 [%s]: ", defaultName)
	if scanner.Scan() {
		binaryName := strings.TrimSpace(scanner.Text())
		if binaryName != "" {
			cfg.BinaryName = binaryName
		} else {
			cfg.BinaryName = defaultName
		}
	} else {
		cfg.BinaryName = defaultName
	}

	// 选择平台
	fmt.Println()
	colorTitle.Println("📋 选择目标平台:")
	fmt.Println("  1) default (默认桌面平台)")
	fmt.Println("  2) desktop (所有桌面平台)")
	fmt.Println("  3) server (服务器平台)")
	fmt.Println("  4) mobile (移动平台)")
	fmt.Println("  5) web (WebAssembly)")
	fmt.Println("  6) embedded (嵌入式平台)")
	fmt.Println("  7) all (所有支持的平台)")
	fmt.Println("  8) 自定义平台组合")
	fmt.Println("  9) 指定单个操作系统 (如 'windows', 'linux', 'darwin')")

	platformChoice := "1"
	colorBold.Print("\n请选择平台 [1]: ")
	if scanner.Scan() {
		choice := strings.TrimSpace(scanner.Text())
		if choice != "" {
			platformChoice = choice
		}
	}

	switch platformChoice {
	case "1":
		cfg.Platforms = []string{"default"}
	case "2":
		cfg.Platforms = []string{"desktop"}
	case "3":
		cfg.Platforms = []string{"server"}
	case "4":
		cfg.Platforms = []string{"mobile"}
	case "5":
		cfg.Platforms = []string{"web"}
	case "6":
		cfg.Platforms = []string{"embedded"}
	case "7":
		cfg.Platforms = []string{"all"}
	case "8":
		colorBold.Print("请输入自定义平台组合 (如 windows/amd64,linux/arm64): ")
		if scanner.Scan() {
			platforms := strings.TrimSpace(scanner.Text())
			if platforms != "" {
				cfg.Platforms = []string{platforms}
			} else {
				cfg.Platforms = []string{"default"}
			}
		}
	case "9":
		colorBold.Print("请输入操作系统名称 (如 windows, linux, darwin): ")
		if scanner.Scan() {
			osName := strings.TrimSpace(scanner.Text())
			if osName != "" {
				cfg.Platforms = []string{osName}
				// 询问是否编译所有架构
				colorBold.Print("是否编译该操作系统的所有架构? (y/N): ")
				if scanner.Scan() {
					response := strings.ToLower(strings.TrimSpace(scanner.Text()))
					cfg.All = (response == "y" || response == "yes")
				}
			} else {
				cfg.Platforms = []string{"default"}
			}
		}
	default:
		colorInfo.Println("无效选择，使用默认平台")
		cfg.Platforms = []string{"default"}
	}

	// 编译选项
	fmt.Println()
	colorTitle.Println("🔧 编译选项:")

	// 并行编译
	colorBold.Printf("并行编译? (Y/n): ")
	if scanner.Scan() {
		response := strings.ToLower(strings.TrimSpace(scanner.Text()))
		if response != "" {
			cfg.Parallel = !(response == "n" || response == "no")
		}
	}

	// 压缩
	colorBold.Printf("压缩二进制文件? (y/N): ")
	if scanner.Scan() {
		response := strings.ToLower(strings.TrimSpace(scanner.Text()))
		if response != "" {
			cfg.Compress = (response == "y" || response == "yes")
		}
	}

	// 清理输出目录
	colorBold.Printf("清理输出目录? (y/N): ")
	if scanner.Scan() {
		response := strings.ToLower(strings.TrimSpace(scanner.Text()))
		if response != "" {
			cfg.Clean = (response == "y" || response == "yes")
		}
	}

	// 跳过CGO平台
	colorBold.Printf("跳过需要CGO支持的平台? (y/N): ")
	if scanner.Scan() {
		response := strings.ToLower(strings.TrimSpace(scanner.Text()))
		if response != "" {
			cfg.SkipCGO = (response == "y" || response == "yes")
		}
	}

	// 详细程度
	colorBold.Printf("详细程度 (0-3) [1]: ")
	if scanner.Scan() {
		verboseStr := strings.TrimSpace(scanner.Text())
		if verboseStr != "" {
			verbose, err := strconv.Atoi(verboseStr)
			if err == nil && verbose >= 0 && verbose <= 3 {
				cfg.Verbose = verbose
			}
		}
	}

	// 高级选项
	fmt.Println()
	colorTitle.Println("⚙️ 高级选项:")

	// Android NDK路径
	colorBold.Printf("Android NDK路径 (留空使用环境变量): ")
	if scanner.Scan() {
		ndkPath := strings.TrimSpace(scanner.Text())
		if ndkPath != "" {
			// 验证NDK路径
			if _, err := os.Stat(ndkPath); os.IsNotExist(err) {
				colorWarning.Printf("⚠️  警告: 指定的NDK路径不存在: %s\n", ndkPath)
				if utils.AskUserConfirm("是否仍然使用此路径?", false) {
					cfg.NDKPath = ndkPath
				}
			} else {
				// 检查NDK目录结构
				if utils.IsValidNDKDir(ndkPath) {
					cfg.NDKPath = ndkPath
					ndkType := utils.DetectNDKType(ndkPath)
					if ndkType != "" {
						colorSuccess.Printf("✓ 检测到NDK类型: %s\n", ndkType)
					}
				} else {
					colorWarning.Printf("⚠️  警告: 指定的路径可能不是有效的NDK根目录\n")
					if utils.AskUserConfirm("是否仍然使用此路径?", false) {
						cfg.NDKPath = ndkPath
					}
				}
			}
		}
	}

	// 链接器标志
	colorBold.Printf("链接器标志 (如 -s -w): ")
	if scanner.Scan() {
		ldflags := strings.TrimSpace(scanner.Text())
		cfg.LDFlags = ldflags
	}

	// 构建标签
	colorBold.Printf("构建标签: ")
	if scanner.Scan() {
		tags := strings.TrimSpace(scanner.Text())
		cfg.Tags = tags
	}

	// 强制编译
	colorBold.Printf("强制编译所有平台? (y/N): ")
	if scanner.Scan() {
		response := strings.ToLower(strings.TrimSpace(scanner.Text()))
		if response != "" {
			cfg.Force = (response == "y" || response == "yes")
		}
	}

	// 确认配置
	fmt.Println()
	colorTitle.Println("📝 配置摘要:")
	fmt.Printf("  • 源文件: %s\n", cfg.SourceFile)
	fmt.Printf("  • 输出目录: %s\n", cfg.OutputDir)
	fmt.Printf("  • 二进制名称: %s\n", cfg.BinaryName)
	fmt.Printf("  • 目标平台: %s\n", strings.Join(cfg.Platforms, ","))
	fmt.Printf("  • 并行编译: %v\n", cfg.Parallel)
	fmt.Printf("  • 压缩二进制: %v\n", cfg.Compress)
	fmt.Printf("  • 清理输出目录: %v\n", cfg.Clean)
	fmt.Printf("  • 跳过CGO平台: %v\n", cfg.SkipCGO)
	fmt.Printf("  • 详细程度: %d\n", cfg.Verbose)
	if cfg.NDKPath != "" {
		fmt.Printf("  • Android NDK路径: %s\n", cfg.NDKPath)
	}
	if cfg.LDFlags != "" {
		fmt.Printf("  • 链接器标志: %s\n", cfg.LDFlags)
	}
	if cfg.Tags != "" {
		fmt.Printf("  • 构建标签: %s\n", cfg.Tags)
	}
	fmt.Printf("  • 强制编译: %v\n", cfg.Force)

	fmt.Println()
	if !utils.AskUserConfirm("开始编译?", true) {
		return fmt.Errorf("用户取消编译")
	}

	// 设置为非交互模式以继续执行
	cfg.Interactive = false
	return nil
}

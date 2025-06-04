package commands

import (
	"bufio"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"

	"github.com/fatih/color"
)

// Config 配置结构体（需要和 main.go 中的 Config 保持一致）
type Config struct {
	SourceFile  string
	OutputDir   string
	BinaryName  string
	Platforms   []string
	Verbose     int
	Parallel    bool
	Compress    bool
	Clean       bool
	Retry       bool
	MaxRetries  int
	Progress    bool
	LDFlags     string
	Tags        string
	SkipTests   bool
	SkipCGO     bool
	Force       bool
	NoPrompt    bool
	All         bool
	Interactive bool
	NoCGO       bool
	NDKPath     string
}

// RunInteractive 运行交互式编译模式
func RunInteractive(config *Config) error {
	colorTitle := color.New(color.FgHiCyan, color.Bold)
	colorBold := color.New(color.Bold)
	colorInfo := color.New(color.FgHiBlue)

	colorTitle.Println("🔍 交互式编译模式")
	scanner := bufio.NewScanner(os.Stdin)

	// 源文件
	if config.SourceFile == "" {
		colorBold.Print("请输入源文件路径: ")
		if scanner.Scan() {
			sourceFile := strings.TrimSpace(scanner.Text())
			if sourceFile == "" {
				return fmt.Errorf("源文件路径不能为空")
			}
			config.SourceFile = sourceFile
		}
	}

	// 输出目录
	defaultOutput := config.OutputDir
	colorBold.Printf("输出目录 [%s]: ", defaultOutput)
	if scanner.Scan() {
		outputDir := strings.TrimSpace(scanner.Text())
		if outputDir != "" {
			config.OutputDir = outputDir
		}
	}

	// 二进制名称
	defaultName := config.BinaryName
	if defaultName == "" {
		defaultName = strings.TrimSuffix(filepath.Base(config.SourceFile), filepath.Ext(config.SourceFile))
	}
	colorBold.Printf("二进制名称 [%s]: ", defaultName)
	if scanner.Scan() {
		binaryName := strings.TrimSpace(scanner.Text())
		if binaryName != "" {
			config.BinaryName = binaryName
		} else {
			config.BinaryName = defaultName
		}
	} else {
		config.BinaryName = defaultName
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
	fmt.Println("  8) 自定义 (手动输入)")

	colorBold.Print("请选择 [1]: ")
	if scanner.Scan() {
		choice := strings.TrimSpace(scanner.Text())
		if choice == "" {
			choice = "1"
		}

		switch choice {
		case "1":
			config.Platforms = []string{"default"}
		case "2":
			config.Platforms = []string{"desktop"}
		case "3":
			config.Platforms = []string{"server"}
		case "4":
			config.Platforms = []string{"mobile"}
		case "5":
			config.Platforms = []string{"web"}
		case "6":
			config.Platforms = []string{"embedded"}
		case "7":
			config.Platforms = []string{"all"}
		case "8":
			colorBold.Print("请输入平台列表 (用逗号分隔): ")
			if scanner.Scan() {
				platforms := strings.TrimSpace(scanner.Text())
				if platforms != "" {
					config.Platforms = strings.Split(platforms, ",")
					for i := range config.Platforms {
						config.Platforms[i] = strings.TrimSpace(config.Platforms[i])
					}
				}
			}
		default:
			colorInfo.Println("无效选择，使用默认平台")
			config.Platforms = []string{"default"}
		}
	}

	// 详细程度
	fmt.Println()
	colorBold.Printf("详细程度 (0=安静, 1=正常, 2=详细) [%d]: ", config.Verbose)
	if scanner.Scan() {
		verboseStr := strings.TrimSpace(scanner.Text())
		if verboseStr != "" {
			if verbose, err := strconv.Atoi(verboseStr); err == nil && verbose >= 0 && verbose <= 2 {
				config.Verbose = verbose
			}
		}
	}

	// 编译选项
	fmt.Println()
	colorTitle.Println("🔧 编译选项:")

	// 并行编译
	defaultParallel := "y"
	if !config.Parallel {
		defaultParallel = "n"
	}
	colorBold.Printf("并行编译 (y/n) [%s]: ", defaultParallel)
	if scanner.Scan() {
		parallel := strings.ToLower(strings.TrimSpace(scanner.Text()))
		if parallel == "" {
			parallel = defaultParallel
		}
		config.Parallel = parallel == "y" || parallel == "yes"
	}

	// 压缩
	defaultCompress := "n"
	if config.Compress {
		defaultCompress = "y"
	}
	colorBold.Printf("压缩二进制文件 (y/n) [%s]: ", defaultCompress)
	if scanner.Scan() {
		compress := strings.ToLower(strings.TrimSpace(scanner.Text()))
		if compress == "" {
			compress = defaultCompress
		}
		config.Compress = compress == "y" || compress == "yes"
	}

	// 清理
	defaultClean := "n"
	if config.Clean {
		defaultClean = "y"
	}
	colorBold.Printf("编译前清理输出目录 (y/n) [%s]: ", defaultClean)
	if scanner.Scan() {
		clean := strings.ToLower(strings.TrimSpace(scanner.Text()))
		if clean == "" {
			clean = defaultClean
		}
		config.Clean = clean == "y" || clean == "yes"
	}

	// ldflags
	colorBold.Printf("链接器标志 (如: \"-s -w\") [%s]: ", config.LDFlags)
	if scanner.Scan() {
		ldflags := strings.TrimSpace(scanner.Text())
		if ldflags != "" {
			config.LDFlags = ldflags
		}
	}

	fmt.Println()
	colorTitle.Println("✅ 配置完成，开始编译...")
	config.Interactive = false // 设置为非交互模式以继续执行
	
	return nil
}
		binaryName := strings.TrimSpace(scanner.Text())
		if binaryName != "" {
			config.BinaryName = binaryName
		} else {
			config.BinaryName = defaultName
		}
	} else {
		config.BinaryName = defaultName
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
		config.Platforms = []string{"default"}
	case "2":
		config.Platforms = []string{"desktop"}
	case "3":
		config.Platforms = []string{"server"}
	case "4":
		config.Platforms = []string{"mobile"}
	case "5":
		config.Platforms = []string{"web"}
	case "6":
		config.Platforms = []string{"embedded"}
	case "7":
		config.Platforms = []string{"all"}
	case "8":
		colorBold.Print("请输入自定义平台组合 (如 windows/amd64,linux/arm64): ")
		if scanner.Scan() {
			platforms := strings.TrimSpace(scanner.Text())
			if platforms != "" {
				config.Platforms = []string{platforms}
			} else {
				config.Platforms = []string{"default"}
			}
		}
	case "9":
		colorBold.Print("请输入操作系统名称 (如 windows, linux, darwin): ")
		if scanner.Scan() {
			os := strings.TrimSpace(scanner.Text())
			if os != "" {
				config.Platforms = []string{os}
				// 询问是否编译所有架构
				colorBold.Print("是否编译该操作系统的所有架构? (y/N): ")
				if scanner.Scan() {
					response := strings.ToLower(strings.TrimSpace(scanner.Text()))
					config.All = (response == "y" || response == "yes")
				}
			} else {
				config.Platforms = []string{"default"}
			}
		}
	default:
		config.Platforms = []string{"default"}
	}

	// 编译选项
	fmt.Println()
	colorTitle.Println("🔧 编译选项:")

	// 并行编译
	colorBold.Printf("并行编译? (Y/n): ")
	if scanner.Scan() {
		response := strings.ToLower(strings.TrimSpace(scanner.Text()))
		if response != "" {
			config.Parallel = !(response == "n" || response == "no")
		}
	}

	// 压缩
	colorBold.Printf("压缩二进制文件? (y/N): ")
	if scanner.Scan() {
		response := strings.ToLower(strings.TrimSpace(scanner.Text()))
		if response != "" {
			config.Compress = (response == "y" || response == "yes")
		}
	}

	// 清理输出目录
	colorBold.Printf("清理输出目录? (y/N): ")
	if scanner.Scan() {
		response := strings.ToLower(strings.TrimSpace(scanner.Text()))
		if response != "" {
			config.Clean = (response == "y" || response == "yes")
		}
	}

	// 跳过CGO平台
	colorBold.Printf("跳过需要CGO支持的平台? (y/N): ")
	if scanner.Scan() {
		response := strings.ToLower(strings.TrimSpace(scanner.Text()))
		if response != "" {
			config.SkipCGO = (response == "y" || response == "yes")
		}
	}

	// 详细程度
	colorBold.Printf("详细程度 (0-3) [1]: ")
	if scanner.Scan() {
		verboseStr := strings.TrimSpace(scanner.Text())
		if verboseStr != "" {
			verbose, err := strconv.Atoi(verboseStr)
			if err == nil && verbose >= 0 && verbose <= 3 {
				config.Verbose = verbose
			}
		}
	}

	// 高级选项
	fmt.Println()
	colorTitle.Println("⚙️ 高级选项:")

	// Android NDK路径	colorBold.Printf("Android NDK路径 (留空使用环境变量): ")
	if scanner.Scan() {
		ndkPath := strings.TrimSpace(scanner.Text())
		if ndkPath != "" {
			// 验证NDK路径			if _, err := os.Stat(ndkPath); os.IsNotExist(err) {
				colorWarning.Printf("⚠️  警告: 指定的NDK路径不存在: %s\n", ndkPath)
				if utils.AskUserConfirm("是否仍然使用此路径?", false) {
					config.NDKPath = ndkPath
				}
			} else {
				// 检查NDK目录结构
				if utils.IsValidNDKDir(ndkPath) {
					config.NDKPath = ndkPath
					ndkType := utils.DetectNDKType(ndkPath)
					if ndkType != "" {
						colorSuccess.Printf("✓ 检测到NDK类型: %s\n", ndkType)
					}
				} else {
					colorWarning.Printf("⚠️  警告: 指定的路径可能不是有效的NDK根目录\n")
					if utils.AskUserConfirm("是否仍然使用此路径?", false) {
						config.NDKPath = ndkPath
					}
				}
			}
			}
		}
	

	// 链接器标志
	colorBold.Printf("链接器标志 (如 -s -w): ")
	if scanner.Scan() {
		ldflags := strings.TrimSpace(scanner.Text())
		config.LDFlags = ldflags
	}

	// 构建标签
	colorBold.Printf("构建标签: ")
	if scanner.Scan() {
		tags := strings.TrimSpace(scanner.Text())
		config.Tags = tags
	}

	// 强制编译
	colorBold.Printf("强制编译所有平台? (y/N): ")
	if scanner.Scan() {
		response := strings.ToLower(strings.TrimSpace(scanner.Text()))
		if response != "" {
			config.Force = (response == "y" || response == "yes")
		}
	}

	// 确认配置
	fmt.Println()
	colorTitle.Println("📝 配置摘要:")
	fmt.Printf("  • 源文件: %s\n", config.SourceFile)
	fmt.Printf("  • 输出目录: %s\n", config.OutputDir)
	fmt.Printf("  • 二进制名称: %s\n", config.BinaryName)
	fmt.Printf("  • 目标平台: %s\n", strings.Join(config.Platforms, ","))
	fmt.Printf("  • 并行编译: %v\n", config.Parallel)
	fmt.Printf("  • 压缩二进制: %v\n", config.Compress)
	fmt.Printf("  • 清理输出目录: %v\n", config.Clean)
	fmt.Printf("  • 跳过CGO平台: %v\n", config.SkipCGO)
	fmt.Printf("  • 详细程度: %d\n", config.Verbose)
	if config.NDKPath != "" {
		fmt.Printf("  • Android NDK路径: %s\n", config.NDKPath)
	}
	if config.LDFlags != "" {
		fmt.Printf("  • 链接器标志: %s\n", config.LDFlags)
	}
	if config.Tags != "" {
		fmt.Printf("  • 构建标签: %s\n", config.Tags)
	}	fmt.Printf("  • 强制编译: %v\n", config.Force)
	fmt.Println()
	if !utils.AskUserConfirm("开始编译?", false) {
		return fmt.Errorf("用户取消编译")
	}

	return nil
}

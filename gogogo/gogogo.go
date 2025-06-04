	package main

	import (
		"bufio"
		"errors"
		"fmt"
		"log/slog"
		"os"
		"os/exec"
		"path/filepath"
		"runtime"
		"strconv"
		"strings"

		"github.com/fatih/color"
		"github.com/lightjunction/rootmanager-module-model/gogogo/build"
		"github.com/lightjunction/rootmanager-module-model/gogogo/utils"
		"github.com/lightjunction/rootmanager-module-model/gogogo/commands"
		"github.com/spf13/cobra"
	)

	// 特殊错误类型
	var ErrSkipped = errors.New("跳过编译")

	// 全局变量
	var (
		config Config
		logger *slog.Logger

		// 颜色定义
		colorTitle   = color.New(color.FgHiCyan, color.Bold)
		colorSuccess = color.New(color.FgHiGreen)
		colorError   = color.New(color.FgHiRed)
		colorWarning = color.New(color.FgHiYellow)
		colorInfo    = color.New(color.FgHiBlue)
		colorBold    = color.New(color.Bold)
	)

	// Config 主配置结构
	type Config struct {
		// 基本编译参数
		SourceFile string
		OutputDir  string
		BinaryName string
		Platforms  []string

		// 编译控制选项
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
		All         bool // 编译指定OS的所有架构（否则仅编译本机架构）
		Interactive bool // 交互式模式
		NoCGO       bool // 完全禁用CGO（无论是否是CGO相关平台）

		// Android平台特有配置
		NDKPath string // Android NDK路径，优先级高于环境变量
	}

	// PlatformGroups 预设平台组合
	var PlatformGroups = map[string][]string{
		"default": {
			"windows/amd64", "windows/386", "windows/arm64",
			"linux/amd64", "linux/386", "linux/arm64", "linux/arm",
			"darwin/amd64", "darwin/arm64",
			"android/arm64", // 只包含最主要的Android平台
		},
		"desktop": {
			"windows/amd64", "windows/386", "windows/arm64",
			"linux/amd64", "linux/386", "linux/arm64", "linux/arm",
			"darwin/amd64", "darwin/arm64",
		},
		"server": {
			"linux/amd64", "linux/arm64",
			"freebsd/amd64", "freebsd/arm64",
		},
		"mobile": {
			"android/arm64", "android/arm",
			"ios/amd64", "ios/arm64",
		},
		"web": {
			"js/wasm",
		},
		"embedded": {
			"linux/arm", "linux/arm64",
			"linux/mips", "linux/mips64",
			"linux/riscv64",
		},
		// "all" 组合将通过 getAllSupportedPlatforms() 动态获取
	}

	func init() {
		// 设置默认配置
		config = Config{
			OutputDir:   "./build",
			Parallel:    true,
			Progress:    true,
			Verbose:     1,
			MaxRetries:  3,
			Retry:       true,
			Interactive: false,
		}

		// 初始化日志
		handler := slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{
			Level: slog.LevelInfo,
		})
		logger = slog.New(handler)

		// 检查Android环境
		utils.CheckAndroidEnvironment(logger)	}

func main() {
	var rootCmd = &cobra.Command{
		Use:   "gogogo",
		Short: "Go跨平台编译工具",
		Long: `gogogo v3.0.0 - 一个强大的Go跨平台编译工具

特性:
  ✓ 支持多平台并行编译
  ✓ 智能重试机制  
  ✓ 进度条显示
  ✓ 文件压缩
  ✓ Android环境支持
  ✓ 详细的日志输出
  ✓ 支持单个OS名称编译

预设平台组合:
  default    默认平台 (桌面 + Android/arm64)
  desktop    桌面平台 (Windows, Linux, macOS)
  server     服务器平台 (Linux, FreeBSD)  
  mobile     移动平台 (Android, iOS) - 需要特殊工具链
  web        Web平台 (WebAssembly)
  embedded   嵌入式平台 (ARM, MIPS, RISC-V)
  all        所有支持的平台 (动态从 'go tool dist list' 获取)

单个操作系统编译:
  • 指定OS名称 (如 'illumos', 'freebsd', 'openbsd')
  • 使用 --all 参数编译指定OS的所有架构（否则仅编译本机架构）
  • 支持自动检测NDK和交叉编译工具链

使用 --no-prompt 参数可以跳过所有用户确认提示。
使用 --all 参数编译指定OS的所有架构（否则仅编译本机架构）。`,
		Example: `  # 交互式模式
  gogogo -i

  # 编译桌面平台
  gogogo -s main.go

  # 编译指定平台
  gogogo -s main.go -p windows/amd64,linux/amd64

  # 编译单个OS的本机架构
  gogogo -s main.go -p illumos

  # 编译单个OS的所有架构
  gogogo -s main.go -p illumos --all

  # 详细输出并压缩
  gogogo -s main.go -v 2 -c

  # 编译所有平台，清理输出目录
  gogogo -s main.go -p all --clean`,		RunE: func(cmd *cobra.Command, args []string) error {
			// 如果是交互式模式，运行交互式编译
			if config.Interactive {
				if err := commands.RunInteractive(&config); err != nil {
					return err
				}
			}

			// 检查必需参数
			if config.SourceFile == "" {
				return fmt.Errorf("请指定源文件 (-s)，使用 'gogogo --help' 查看帮助")
			}

			// 设置日志级别
			var logLevel slog.Level
			switch config.Verbose {
			case 0:
				logLevel = slog.LevelError
			case 1:
				logLevel = slog.LevelInfo
			case 2:
				logLevel = slog.LevelDebug
			default:
				logLevel = slog.LevelDebug
			}

			handler := slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{
				Level: logLevel,
			})
			logger = slog.New(handler)

			// 检查Go环境
			if err := utils.CheckGoEnvironment(); err != nil {
				return err
			}			// 创建输出目录
			if err := os.MkdirAll(config.OutputDir, 0755); err != nil {
				return fmt.Errorf("创建输出目录失败: %v", err)
			}

			// 清理输出目录
			if config.Clean {
				if err := utils.CleanOutputDir(config.OutputDir, config.Verbose, logger); err != nil {
					return fmt.Errorf("清理输出目录失败: %v", err)
				}
			}			// 构建配置
			buildConfig := build.BuildConfig{
				SkipCGO:  config.SkipCGO,
				Verbose:  config.Verbose,
				LDFlags:  config.LDFlags,
				Tags:     config.Tags,
				Force:    config.Force,
				NoPrompt: config.NoPrompt,
				NoCGO:    config.NoCGO,
				NDKPath:  config.NDKPath,
				Compress: config.Compress,
			}

			utilsConfig := &utils.Config{
				All:      config.All,
				Verbose:  config.Verbose,
				NoPrompt: config.NoPrompt,
			}			// 解析目标平台
			targets := utils.ParsePlatforms(strings.Join(config.Platforms, ","), *utilsConfig, logger)
			if len(targets) == 0 {
				return fmt.Errorf("没有找到有效的目标平台")
			}

			// 执行编译
			progressConfig := build.ProgressConfig{
				Progress:   config.Progress,
				Parallel:   config.Parallel,
				Verbose:    config.Verbose,
				MaxRetries: 1, // 设置默认重试次数
			}
			return build.BuildWithProgress(targets, config.SourceFile, config.OutputDir, config.BinaryName, buildConfig, progressConfig, logger)
		},
	}

	// 添加子命令	var listCmd = &cobra.Command{
		Use:   "list",
		Short: "列出所有支持的平台",
		Long:  "列出Go工具链支持的所有目标平台",
		Run: func(cmd *cobra.Command, args []string) {
			commands.ListPlatforms()
		},
	}
	var groupsCmd = &cobra.Command{
		Use:   "groups",
		Short: "列出所有平台组合",
		Long:  "列出预设的平台组合，可以直接使用这些组合名称",
		Run: func(cmd *cobra.Command, args []string) {
			commands.ListGroups(PlatformGroups)
		},
	}
	var versionCmd = &cobra.Command{
		Use:   "version",
		Short: "显示版本信息",
		Long:  "显示gogogo的版本信息和环境信息",
		Run: func(cmd *cobra.Command, args []string) {
			commands.ShowVersion()
		},
	}
	var examplesCmd = &cobra.Command{
		Use:   "examples",
		Short: "显示使用示例",
		Long:  "显示详细的使用示例和常见用法",
		Run: func(cmd *cobra.Command, args []string) {
			commands.ShowExamples()
		},
	}
	var envCmd = &cobra.Command{
		Use:   "env",
		Short: "显示编译环境信息",
		Long:  "显示Go编译环境、Android NDK、交叉编译等相关环境变量信息",
		Run: func(cmd *cobra.Command, args []string) {
			commands.GetEnvironmentInfo(logger)
		},
	}

	// 添加子命令到根命令
	rootCmd.AddCommand(listCmd, groupsCmd, versionCmd, examplesCmd, envCmd)

	// 添加主要的命令行参数
	rootCmd.Flags().StringVarP(&config.SourceFile, "source", "s", "", "源Go文件路径 (必需)")
	rootCmd.Flags().StringVarP(&config.OutputDir, "output", "o", "./build", "输出目录")
	rootCmd.Flags().StringVarP(&config.BinaryName, "name", "n", "", "二进制文件名 (默认: 源文件名)")
	rootCmd.Flags().StringSliceVarP(&config.Platforms, "platforms", "p", []string{"default"}, "目标平台 (可使用预设组合或具体平台)")

	// 构建选项
	rootCmd.Flags().IntVarP(&config.Verbose, "verbose", "v", 1, "详细程度 (0=安静, 1=正常, 2=详细, 3=调试)")
	rootCmd.Flags().BoolVar(&config.Parallel, "parallel", true, "并行编译")
	rootCmd.Flags().BoolVarP(&config.Compress, "compress", "c", false, "压缩二进制文件")
	rootCmd.Flags().BoolVar(&config.Clean, "clean", false, "编译前清理输出目录")
	rootCmd.Flags().BoolVar(&config.Progress, "progress", true, "显示进度条")
	rootCmd.Flags().StringVar(&config.LDFlags, "ldflags", "", "链接器标志 (如: \"-s -w\")")
	rootCmd.Flags().StringVar(&config.Tags, "tags", "", "构建标签")
	rootCmd.Flags().BoolVar(&config.Retry, "retry", true, "编译失败时重试")
	rootCmd.Flags().IntVar(&config.MaxRetries, "max-retries", 3, "最大重试次数")
	rootCmd.Flags().BoolVar(&config.SkipTests, "skip-tests", false, "跳过测试")
	rootCmd.Flags().BoolVar(&config.SkipCGO, "skip-cgo", false, "跳过需要CGO支持的平台")
	rootCmd.Flags().BoolVar(&config.Force, "force", false, "强制编译所有平台（包括在非macOS上编译iOS）")
	rootCmd.Flags().BoolVar(&config.NoPrompt, "no-prompt", false, "跳过所有用户确认提示")
	rootCmd.Flags().BoolVarP(&config.Interactive, "interactive", "i", false, "交互式模式")
	rootCmd.Flags().BoolVar(&config.NoCGO, "no-cgo", false, "完全禁用CGO（无论是否是CGO相关平台）")
	rootCmd.Flags().BoolVar(&config.All, "all", false, "编译指定OS的所有架构（否则仅编译本机架构）")
	rootCmd.Flags().StringVar(&config.NDKPath, "ndk-path", "", "Android NDK路径（优先级高于环境变量）")

	// 设置帮助模板
	rootCmd.SetHelpTemplate(`{{.Long}}

用法:
  {{.UseLine}}{{if .HasAvailableSubCommands}}
  {{.CommandPath}} [command]{{end}}{{if gt (len .Aliases) 0}}

别名:
  {{.NameAndAliases}}{{end}}{{if .HasExample}}

示例:
{{.Example}}{{end}}{{if .HasAvailableSubCommands}}

可用命令:{{range .Commands}}{{if (or .IsAvailableCommand (eq .Name "help"))}}
  {{rpad .Name .NamePadding }} {{.Short}}{{end}}{{end}}{{end}}{{if .HasAvailableLocalFlags}}

选项:
{{.LocalFlags.FlagUsages}}{{end}}{{if .HasAvailableInheritedFlags}}

全局选项:
{{.InheritedFlags.FlagUsages}}{{end}}{{if .HasHelpSubCommands}}

其他帮助主题:{{range .Commands}}{{if .IsAdditionalHelpTopicCommand}}
  {{rpad .Name .NamePadding }} {{.Short}}{{end}}{{end}}{{end}}{{if .HasAvailableSubCommands}}

使用 "{{.CommandPath}} [command] --help" 获取更多关于命令的信息。{{end}}
`)

	// 执行命令
	if err := rootCmd.Execute(); err != nil {
		colorError.Printf("❌ 错误: %v\n", err)
		os.Exit(1)
	}
}

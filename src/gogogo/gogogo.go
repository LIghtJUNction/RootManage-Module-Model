package main

import (
	"bufio"
	"errors"
	"fmt"
	"io/ioutil"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/fatih/color"
	"github.com/klauspost/compress/gzip"
	"github.com/schollz/progressbar/v3"
	"github.com/sirupsen/logrus"
	"github.com/spf13/cobra"
)

// 特殊错误类型
var ErrSkipped = errors.New("跳过编译")

// BuildTarget 构建目标
type BuildTarget struct {
	GOOS   string
	GOARCH string
	Name   string
}

// #region 基础配置结构
// Config 配置结构
type Config struct {
	// #region 基本编译参数
	SourceFile string
	OutputDir  string
	BinaryName string
	Platforms  []string
	// #endregion

	// #region 编译控制选项
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
	// #endregion

	// #region Android平台特有配置
	NDKPath string // Android NDK路径，优先级高于环境变量
	// #endregion
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
	}, "embedded": {
		"linux/arm", "linux/arm64",
		"linux/mips", "linux/mips64",
		"linux/riscv64",
	},
	// "all" 组合将通过 getAllSupportedPlatforms() 动态获取
}

var (
	// 颜色配置
	colorTitle   = color.New(color.FgCyan, color.Bold)
	colorSuccess = color.New(color.FgGreen, color.Bold)
	colorError   = color.New(color.FgRed, color.Bold)
	colorWarning = color.New(color.FgYellow, color.Bold)
	colorInfo    = color.New(color.FgBlue)
	colorBold    = color.New(color.Bold)

	// 全局配置
	config Config
	logger *logrus.Logger
)

func init() {
	// 初始化日志
	logger = logrus.New()
	logger.SetFormatter(&logrus.TextFormatter{
		DisableColors: false,
		FullTimestamp: true,
	})

	// 检查Android环境
	checkAndroidEnvironment()
}

// checkAndroidEnvironment 检查Android环境并设置GOENV
func checkAndroidEnvironment() {
	if runtime.GOOS == "android" {
		goenvPath := "/data/adb/modules/gogogo/go.env"
		if _, err := os.Stat(goenvPath); err == nil {
			os.Setenv("GOENV", goenvPath)
			logger.Info("检测到Android环境，已设置GOENV:", goenvPath)
		}
	}
}

// checkGoEnvironment 检查Go环境
func checkGoEnvironment() error {
	colorInfo.Print("🔍 检查Go环境...")

	// 检查go命令
	if _, err := exec.LookPath("go"); err != nil {
		return fmt.Errorf("未找到go命令，请确保Go已正确安装并添加到PATH")
	}

	// 获取Go版本
	cmd := exec.Command("go", "version")
	output, err := cmd.Output()
	if err != nil {
		return fmt.Errorf("无法获取Go版本: %v", err)
	}
	colorSuccess.Printf(" ✓ %s\n", strings.TrimSpace(string(output)))
	return nil
}

// detectNDKType 检测NDK的类型 (Windows/Linux/Mac)
func detectNDKType(ndkPath string) string {
	// 检查toolchains目录下的预编译工具目录
	toolchainsPath := filepath.Join(ndkPath, "toolchains", "llvm", "prebuilt")
	if _, err := os.Stat(toolchainsPath); os.IsNotExist(err) {
		// 尝试查找旧的NDK目录结构
		files, err := ioutil.ReadDir(ndkPath)
		if err != nil {
			return ""
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
		return ""
	}

	// 检查现代NDK结构
	files, err := ioutil.ReadDir(toolchainsPath)
	if err != nil {
		return ""
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
		}
	}

	return ""
}

// getNDKPrebuiltPath 获取NDK预编译工具的路径
func getNDKPrebuiltPath(ndkPath string, ndkType string) string {
	// 标准路径结构: toolchains/llvm/prebuilt/OS-ARCH
	baseDir := filepath.Join(ndkPath, "toolchains", "llvm", "prebuilt")
	if _, err := os.Stat(baseDir); os.IsNotExist(err) {
		return ""
	}

	files, err := ioutil.ReadDir(baseDir)
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

// setupNDKEnvironment 为Android NDK设置环境变量
func setupNDKEnvironment(ndkPath string, arch string, cmdEnv *[]string) error {
	// 检测NDK类型
	ndkType := detectNDKType(ndkPath)
	if ndkType == "" {
		return fmt.Errorf("无法确定NDK类型")
	}

	// 根据宿主系统类型和NDK类型设置不同的环境变量
	hostOS := runtime.GOOS
	if config.Verbose >= 2 {
		colorInfo.Printf("✓ 检测到NDK类型: %s, 宿主系统: %s\n", ndkType, hostOS)
	}

	prebuiltPath := getNDKPrebuiltPath(ndkPath, ndkType)
	if prebuiltPath == "" {
		return fmt.Errorf("无法找到NDK预编译工具路径")
	}

	// NDK基本环境变量
	*cmdEnv = append(*cmdEnv, "ANDROID_NDK_HOME="+ndkPath)
	*cmdEnv = append(*cmdEnv, "ANDROID_NDK_ROOT="+ndkPath)

	// 为不同的宿主系统和NDK类型设置特定的环境变量
	if hostOS == "windows" {
		// Windows宿主
		if ndkType == "windows" {
			// Windows NDK
			*cmdEnv = append(*cmdEnv, "CGO_CFLAGS=-I"+filepath.Join(prebuiltPath, "sysroot", "usr", "include"))
			*cmdEnv = append(*cmdEnv, "CGO_LDFLAGS=-L"+filepath.Join(prebuiltPath, "sysroot", "usr", "lib"))
		} else {
			// 非Windows NDK在Windows上使用
			colorWarning.Printf("⚠️  在Windows上使用非Windows NDK可能会有兼容性问题\n")
			*cmdEnv = append(*cmdEnv, "CGO_CFLAGS=-I"+filepath.Join(prebuiltPath, "sysroot", "usr", "include"))
			*cmdEnv = append(*cmdEnv, "CGO_LDFLAGS=-L"+filepath.Join(prebuiltPath, "sysroot", "usr", "lib"))
		}
	} else if hostOS == "linux" {
		// Linux宿主
		if ndkType == "linux" {
			// Linux NDK
			*cmdEnv = append(*cmdEnv, "CGO_CFLAGS=-I"+filepath.Join(prebuiltPath, "sysroot", "usr", "include"))
			*cmdEnv = append(*cmdEnv, "CGO_LDFLAGS=-L"+filepath.Join(prebuiltPath, "sysroot", "usr", "lib"))
		} else {
			// 非Linux NDK在Linux上使用
			colorWarning.Printf("⚠️  在Linux上使用非Linux NDK可能需要额外的兼容层\n")
			if ndkType == "windows" {
				colorInfo.Printf("💡 在Linux上使用Windows NDK可能需要Wine支持\n")
			}
		}
	} else if hostOS == "darwin" {
		// Mac宿主
		if ndkType == "darwin" {
			// Mac NDK
			*cmdEnv = append(*cmdEnv, "CGO_CFLAGS=-I"+filepath.Join(prebuiltPath, "sysroot", "usr", "include"))
			*cmdEnv = append(*cmdEnv, "CGO_LDFLAGS=-L"+filepath.Join(prebuiltPath, "sysroot", "usr", "lib"))
		} else {
			// 非Mac NDK在Mac上使用
			colorWarning.Printf("⚠️  在macOS上使用非macOS NDK可能会有兼容性问题\n")
		}
	}

	// 为特定架构设置额外的环境变量
	if arch == "arm64" {
		*cmdEnv = append(*cmdEnv, "CC="+filepath.Join(prebuiltPath, "bin", "aarch64-linux-android21-clang"))
	} else if arch == "arm" {
		*cmdEnv = append(*cmdEnv, "CC="+filepath.Join(prebuiltPath, "bin", "armv7a-linux-androideabi21-clang"))
	}

	return nil
}

// getAllSupportedPlatforms 获取Go支持的所有平台
func getAllSupportedPlatforms() ([]string, error) {
	cmd := exec.Command("go", "tool", "dist", "list")
	output, err := cmd.Output()
	if err != nil {
		return nil, fmt.Errorf("获取平台列表失败: %v", err)
	}

	platforms := strings.Split(strings.TrimSpace(string(output)), "\n")
	var validPlatforms []string
	for _, platform := range platforms {
		platform = strings.TrimSpace(platform)
		if platform != "" && strings.Contains(platform, "/") {
			validPlatforms = append(validPlatforms, platform)
		}
	}

	return validPlatforms, nil
}

// getArchsForOS 获取指定操作系统支持的架构列表
func getArchsForOS(targetOS string) ([]string, error) {
	allPlatforms, err := getAllSupportedPlatforms()
	if err != nil {
		return nil, err
	}

	var archs []string
	for _, platform := range allPlatforms {
		parts := strings.Split(platform, "/")
		if len(parts) == 2 && parts[0] == targetOS {
			archs = append(archs, parts[1])
		}
	}

	return archs, nil
}

// getNativeArch 获取本机架构
func getNativeArch() string {
	return runtime.GOARCH
}

// parsePlatforms 解析平台字符串
func parsePlatforms(platformStr string) []BuildTarget {
	var targets []BuildTarget
	platforms := strings.Split(platformStr, ",")
	for _, platform := range platforms {
		platform = strings.TrimSpace(platform)

		// 特殊处理 "all" 平台组合
		if platform == "all" {
			allPlatforms, err := getAllSupportedPlatforms()
			if err != nil {
				if config.Verbose >= 1 {
					colorError.Printf("⚠️  获取所有平台失败，使用静态列表: %v\n", err)
				}
				// 如果获取失败，使用静态的备用列表
				fallbackAll := []string{
					"windows/amd64", "windows/386", "windows/arm64",
					"linux/amd64", "linux/386", "linux/arm64", "linux/arm",
					"darwin/amd64", "darwin/arm64",
					"freebsd/amd64", "freebsd/arm64",
					"android/arm64", "android/arm",
					"ios/amd64", "ios/arm64",
					"js/wasm",
					"linux/mips", "linux/mips64",
					"linux/riscv64",
					"openbsd/amd64", "netbsd/amd64",
					"dragonfly/amd64", "solaris/amd64",
				}
				allPlatforms = fallbackAll
			}

			for _, p := range allPlatforms {
				parts := strings.Split(p, "/")
				if len(parts) == 2 {
					targets = append(targets, BuildTarget{
						GOOS:   parts[0],
						GOARCH: parts[1],
						Name:   p,
					})
				}
			}
		} else if group, exists := PlatformGroups[platform]; exists {
			// 检查是否是其他预设组合
			for _, p := range group {
				parts := strings.Split(p, "/")
				if len(parts) == 2 {
					targets = append(targets, BuildTarget{
						GOOS:   parts[0],
						GOARCH: parts[1],
						Name:   p,
					})
				}
			}
		} else if strings.Contains(platform, "/") {
			// 包含斜杠的为完整平台格式 (OS/ARCH)
			parts := strings.Split(platform, "/")
			if len(parts) == 2 {
				targets = append(targets, BuildTarget{
					GOOS:   parts[0],
					GOARCH: parts[1],
					Name:   platform,
				})
			}
		} else {
			// 单个操作系统名称，需要根据 -all 标志决定架构
			var archs []string
			var err error

			if config.All {
				// 获取该OS支持的所有架构
				archs, err = getArchsForOS(platform)
				if err != nil {
					if config.Verbose >= 1 {
						colorError.Printf("⚠️  获取 %s 支持的架构失败: %v\n", platform, err)
					}
					continue
				}
				if len(archs) == 0 {
					if config.Verbose >= 1 {
						colorWarning.Printf("⚠️  操作系统 %s 不支持或未找到\n", platform)
					}
					continue
				}
			} else {
				// 仅使用本机架构
				nativeArch := getNativeArch()
				// 验证该OS是否支持本机架构
				supportedArchs, err := getArchsForOS(platform)
				if err != nil {
					if config.Verbose >= 1 {
						colorError.Printf("⚠️  获取 %s 支持的架构失败: %v\n", platform, err)
					}
					continue
				}

				// 检查本机架构是否在支持列表中
				found := false
				for _, arch := range supportedArchs {
					if arch == nativeArch {
						found = true
						break
					}
				}

				if found {
					archs = []string{nativeArch}
				} else {
					if config.Verbose >= 1 {
						colorWarning.Printf("⚠️  操作系统 %s 不支持本机架构 %s，支持的架构: %s\n",
							platform, nativeArch, strings.Join(supportedArchs, ", "))
						colorInfo.Printf("💡 可以使用 --all 标志编译该OS的所有架构\n")
					}
					continue
				}
			}

			// 添加目标平台
			for _, arch := range archs {
				targets = append(targets, BuildTarget{
					GOOS:   platform,
					GOARCH: arch,
					Name:   platform + "/" + arch,
				})
			}
		}
	}
	return targets
}

// askUserConfirm 询问用户确认
func askUserConfirm(prompt string) bool {
	if config.NoPrompt {
		return true
	}

	colorWarning.Printf("%s (y/N): ", prompt)
	scanner := bufio.NewScanner(os.Stdin)
	if scanner.Scan() {
		response := strings.ToLower(strings.TrimSpace(scanner.Text()))
		return response == "y" || response == "yes"
	}
	return false
}

// buildSingle 编译单个目标
func buildSingle(target BuildTarget, sourceFile, outputDir, binaryName string) error { // 跳过CGO相关平台
	if config.SkipCGO && (target.GOOS == "android" || target.GOOS == "ios") {
		if config.Verbose >= 1 {
			colorWarning.Printf("⚠️  跳过需要CGO支持的平台: %s (使用 --skip-cgo=false 强制编译)\n", target.Name)
		}
		return ErrSkipped
	}

	// 构建输出文件名
	filename := binaryName
	if target.GOOS == "windows" {
		filename += ".exe"
	}

	outputPath := filepath.Join(outputDir, target.Name, filename)

	// 确保输出目录存在
	if err := os.MkdirAll(filepath.Dir(outputPath), 0755); err != nil {
		return fmt.Errorf("创建输出目录失败: %v", err)
	}

	// 构建命令
	args := []string{"build"}

	if config.LDFlags != "" {
		args = append(args, "-ldflags", config.LDFlags)
	}

	if config.Tags != "" {
		args = append(args, "-tags", config.Tags)
	}

	args = append(args, "-o", outputPath, sourceFile)

	// 设置环境变量
	cmd := exec.Command("go", args...)
	cmd.Env = append(os.Environ(),
		"GOOS="+target.GOOS,
		"GOARCH="+target.GOARCH,
	)
	// 特殊平台的CGO设置
	if target.GOOS == "js" {
		// WebAssembly需要禁用CGO
		cmd.Env = append(cmd.Env, "CGO_ENABLED=0")
	} else if target.GOOS == "ios" {
		// iOS平台特殊处理
		if runtime.GOOS != "darwin" {
			if !config.Force {
				if config.Verbose >= 1 {
					colorWarning.Printf("⚠️  跳过iOS平台: 只能在macOS上编译 (使用 --force 强制尝试)\n")
				}
				return ErrSkipped
			} else {
				colorError.Printf("⚠️  警告: 在非macOS系统上强制编译iOS，可能会失败!\n")
			}
		}

		// 检查是否安装了Xcode (仅在macOS上)
		if runtime.GOOS == "darwin" {
			if _, err := exec.LookPath("xcodebuild"); err != nil {
				return fmt.Errorf("iOS编译需要安装Xcode和Command Line Tools")
			}
		}

		cmd.Env = append(cmd.Env, "CGO_ENABLED=1")
		if config.Verbose >= 1 {
			colorWarning.Printf("⚠️  iOS平台需要Xcode和iOS SDK，建议使用gomobile工具\n")
			colorInfo.Printf("💡 安装gomobile: go install golang.org/x/mobile/cmd/gomobile@latest\n")
			colorInfo.Printf("💡 初始化gomobile: gomobile init\n")
			colorInfo.Printf("💡 构建iOS应用: gomobile build -target=ios .\n")
		}
	} else if target.GOOS == "android" {
		// #region Android平台处理
		if config.Verbose >= 1 {
			colorWarning.Printf("⚠️  Android平台建议使用gomobile工具进行构建\n")
			colorInfo.Printf("💡 安装gomobile: go install golang.org/x/mobile/cmd/gomobile@latest\n")
			colorInfo.Printf("💡 构建Android应用: gomobile build -target=android .\n")
			colorInfo.Printf("✓ 自动继续使用标准Go工具链编译Android平台\n")
		}

		// 处理Android编译选项
		var ndkHome string

		// 使用NoCGO标志完全禁用CGO（适用于纯Go代码）
		if config.NoCGO {
			if config.Verbose >= 1 {
				colorInfo.Printf("💡 使用--no-cgo标志，禁用Android的CGO编译\n")
			}
			cmd.Env = append(cmd.Env, "CGO_ENABLED=0")
			// 不再提前返回，让编译继续进行
		} else if runtime.GOOS != "android" { // 仅在交叉编译时检查NDK环境
			// 优先使用命令行指定的NDK路径
			if config.NDKPath != "" {
				ndkHome = config.NDKPath
				if config.Verbose >= 1 {
					colorInfo.Printf("💡 使用命令行指定的NDK路径: %s\n", ndkHome)
				}
			} else {
				// 其次检查是否配置了Android NDK环境变量
				ndkHome = os.Getenv("ANDROID_NDK_HOME")
				if ndkHome == "" {
					ndkHome = os.Getenv("ANDROID_NDK_ROOT")
				}
			}

			if ndkHome == "" {
				if !config.Force && !config.NoPrompt {
					if config.Verbose >= 1 {
						colorError.Printf("⚠️  编译Android平台需要设置Android NDK环境\n")
						colorInfo.Printf("💡 未检测到NDK路径或环境变量\n")

						// 询问用户是否要提供NDK路径
						if askUserConfirm("是否手动提供Android NDK路径?") {
							colorBold.Print("请输入Android NDK根目录路径: ")
							scanner := bufio.NewScanner(os.Stdin)
							if scanner.Scan() {
								ndkPath := strings.TrimSpace(scanner.Text())
								if ndkPath != "" {
									// 检查路径是否存在
									if _, err := os.Stat(ndkPath); os.IsNotExist(err) {
										colorError.Printf("❌ 指定的NDK路径不存在: %s\n", ndkPath)
										return ErrSkipped
									}

									// 检查该目录是否包含一些NDK的典型文件夹
									possibleDirs := []string{"toolchains", "platforms", "sources", "sysroot"}
									validNDK := false
									for _, dir := range possibleDirs {
										if _, err := os.Stat(filepath.Join(ndkPath, dir)); !os.IsNotExist(err) {
											validNDK = true
											break
										}
									}

									if !validNDK {
										colorWarning.Printf("⚠️  指定的路径可能不是有效的NDK根目录，缺少关键文件夹\n")
										if !askUserConfirm("是否继续使用此路径?") {
											return ErrSkipped
										}
									}

									// 使用用户提供的NDK路径
									ndkHome = ndkPath
									colorSuccess.Printf("✓ 已设置临时NDK路径: %s\n", ndkHome)

									// 显示永久设置环境变量的指导
									colorInfo.Printf("\n📝 如需永久配置NDK环境，请设置系统环境变量:\n")
									if runtime.GOOS == "windows" {
										colorInfo.Printf("  • PowerShell: $env:ANDROID_NDK_HOME = \"%s\"\n", ndkPath)
										colorInfo.Printf("  • CMD: set ANDROID_NDK_HOME=%s\n", ndkPath)
										colorInfo.Printf("  • 系统环境变量: 右键\"此电脑\" -> 属性 -> 高级系统设置 -> 环境变量\n")
									} else {
										colorInfo.Printf("  • Bash/Zsh: export ANDROID_NDK_HOME=\"%s\"\n", ndkPath)
										colorInfo.Printf("  • 永久配置: 添加到 ~/.bashrc 或 ~/.zshrc 文件\n")
									}
									colorInfo.Printf("\n")
								} else {
									colorWarning.Printf("⚠️  未提供NDK路径，跳过编译\n")
									return ErrSkipped
								}
							} else {
								colorWarning.Printf("⚠️  读取输入失败，跳过编译\n")
								return ErrSkipped
							}
						} else {
							colorInfo.Printf("💡 跳过Android编译。您可以使用以下选项之一:\n")
							colorInfo.Printf("  1. 使用 --ndk-path 参数指定NDK路径\n")
							colorInfo.Printf("  2. 设置ANDROID_NDK_HOME环境变量指向NDK根目录\n")
							colorInfo.Printf("  3. 使用 --force 参数强制尝试编译\n")
							colorInfo.Printf("  4. 使用 --no-cgo 参数禁用CGO编译（仅适用于纯Go代码）\n")
							return ErrSkipped
						}
					} else {
						return ErrSkipped
					}
				} else if config.Force {
					colorError.Printf("⚠️  警告: 未设置NDK路径，强制尝试编译可能会失败！\n")
				} else {
					// 静默模式，没有force标志，直接跳过
					return ErrSkipped
				}
			} else {
				// 使用智能环境变量设置
				if err := setupNDKEnvironment(ndkHome, target.GOARCH, &cmd.Env); err != nil {
					if config.Verbose >= 1 {
						colorWarning.Printf("⚠️  设置NDK环境变量失败: %v\n", err)
						colorInfo.Printf("💡 将使用传统方式设置NDK环境\n")
					}
					// 如果智能设置失败，回退到简单的环境变量设置
					cmd.Env = append(cmd.Env,
						"ANDROID_NDK_HOME="+ndkHome,
						"CGO_CFLAGS=-I"+filepath.Join(ndkHome, "toolchains", "llvm", "prebuilt", runtime.GOOS+"-x86_64", "sysroot", "usr", "include"))
				} else if config.Verbose >= 2 {
					colorSuccess.Printf("✓ 已根据NDK类型和宿主系统智能配置环境变量\n")
				}
			}
		}

		cmd.Env = append(cmd.Env, "CGO_ENABLED=1")

		if config.Verbose >= 1 && runtime.GOOS == "windows" {
			colorInfo.Printf("💡 Windows上可以直接编译Android/arm64平台\n")
		}
		// #endregion

		// 为Android设置编译标志，尝试静态链接
		if config.LDFlags == "" {
			// 尝试静态链接，如果失败会降级到动态链接
			newLDFlags := "-linkmode=external -extldflags=-static"
			for i, arg := range args {
				if arg == "-o" {
					// 在-o参数前插入ldflags
					newArgs := make([]string, 0, len(args)+2)
					newArgs = append(newArgs, args[:i]...)
					newArgs = append(newArgs, "-ldflags", newLDFlags)
					newArgs = append(newArgs, args[i:]...)
					args = newArgs
					break
				}
			}
		}
	} else {
		// 其他平台通常禁用CGO以避免交叉编译问题
		cmd.Env = append(cmd.Env, "CGO_ENABLED=0")
	}

	if config.Verbose >= 2 {
		logger.Infof("执行命令: %s", strings.Join(cmd.Args, " "))
		logger.Infof("环境变量: GOOS=%s GOARCH=%s", target.GOOS, target.GOARCH)
	}

	// 执行编译
	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("编译失败 [%s]: %v\n输出: %s", target.Name, err, string(output))
	}

	// 压缩文件
	if config.Compress {
		if err := compressFile(outputPath); err != nil {
			logger.Warnf("压缩文件失败 [%s]: %v", target.Name, err)
		}
	}

	return nil
}

// compressFile 压缩文件
func compressFile(filePath string) error {
	// 读取原文件
	input, err := os.ReadFile(filePath)
	if err != nil {
		return err
	}

	// 创建压缩文件
	compressedPath := filePath + ".gz"
	output, err := os.Create(compressedPath)
	if err != nil {
		return err
	}
	defer output.Close()

	// 使用gzip压缩
	writer := gzip.NewWriter(output)
	defer writer.Close()

	_, err = writer.Write(input)
	if err != nil {
		return err
	}

	// 删除原文件
	os.Remove(filePath)

	return nil
}

// buildWithProgress 带进度条的编译
func buildWithProgress(targets []BuildTarget, sourceFile, outputDir, binaryName string) error {
	if config.Verbose >= 1 {
		colorInfo.Printf("🚀 开始编译 %d 个目标平台\n", len(targets))
	}

	var bar *progressbar.ProgressBar
	if config.Progress && config.Verbose >= 1 {
		bar = progressbar.NewOptions(len(targets),
			progressbar.OptionSetDescription("编译进度"),
			progressbar.OptionSetTheme(progressbar.Theme{
				Saucer:        "█",
				SaucerPadding: "░",
				BarStart:      "[",
				BarEnd:        "]",
			}),
			progressbar.OptionShowCount(),
			progressbar.OptionShowIts(),
		)
	}
	var wg sync.WaitGroup
	var mu sync.Mutex
	var errs []error
	var skipped []string
	var successful []string

	// 控制并发数
	maxWorkers := runtime.NumCPU()
	if !config.Parallel {
		maxWorkers = 1
	}

	semaphore := make(chan struct{}, maxWorkers)

	for _, target := range targets {
		wg.Add(1)
		go func(t BuildTarget) {
			defer wg.Done()

			semaphore <- struct{}{}
			defer func() { <-semaphore }()

			// 重试逻辑
			var err error
			for attempt := 0; attempt <= config.MaxRetries; attempt++ {
				err = buildSingle(t, sourceFile, outputDir, binaryName)
				if err == nil {
					break
				}

				if attempt < config.MaxRetries && config.Retry {
					if config.Verbose >= 2 {
						logger.Warnf("编译失败，正在重试 [%s] (第%d次): %v", t.Name, attempt+1, err)
					}
					time.Sleep(time.Second * time.Duration(attempt+1))
				}
			}

			mu.Lock()
			if err != nil {
				if errors.Is(err, ErrSkipped) {
					// 跳过的平台不计入错误
					skipped = append(skipped, t.Name)
					if config.Verbose >= 1 {
						colorWarning.Printf("⏭️ %s (跳过)\n", t.Name)
					}
				} else {
					errs = append(errs, fmt.Errorf("[%s] %v", t.Name, err))
				}
			} else {
				successful = append(successful, t.Name)
				if config.Verbose >= 1 {
					colorSuccess.Printf("✓ %s\n", t.Name)
				}
			}

			if bar != nil {
				bar.Add(1)
			}
			mu.Unlock()
		}(target)
	}
	wg.Wait()
	if len(errs) > 0 {
		colorError.Println("\n❌ 编译过程中出现错误:")
		for _, err := range errs {
			colorError.Printf("  • %v\n", err)
		}
		return fmt.Errorf("编译失败: %d个目标出现错误", len(errs))
	}

	if config.Verbose >= 1 {
		if len(successful) > 0 {
			colorSuccess.Printf("\n🎉 编译完成! 共编译 %d 个目标平台\n", len(successful))
		}
		if len(skipped) > 0 {
			colorWarning.Printf("⏭️ 跳过 %d 个目标平台: %s\n", len(skipped), strings.Join(skipped, ", "))
		}
		if len(successful) == 0 && len(skipped) > 0 {
			colorInfo.Printf("💡 所有平台都被跳过，没有实际编译任何目标\n")
		}
	}

	return nil
}

// listPlatforms 列出所有支持的平台
func listPlatforms() {
	colorTitle.Println("📋 支持的平台:")

	// 获取所有平台
	cmd := exec.Command("go", "tool", "dist", "list")
	output, err := cmd.Output()
	if err != nil {
		colorError.Printf("获取平台列表失败: %v\n", err)
		return
	}

	platforms := strings.Split(strings.TrimSpace(string(output)), "\n")

	// 按OS分组显示
	osGroups := make(map[string][]string)
	for _, platform := range platforms {
		parts := strings.Split(platform, "/")
		if len(parts) == 2 {
			osGroups[parts[0]] = append(osGroups[parts[0]], parts[1])
		}
	}

	for os, archs := range osGroups {
		colorBold.Printf("  %s: ", os)
		fmt.Printf("%s\n", strings.Join(archs, ", "))
	}
}

// listGroups 列出平台组合
func listGroups() {
	colorTitle.Println("📦 平台组合:")

	// 显示静态预设组合
	for group, platforms := range PlatformGroups {
		colorBold.Printf("  %s:\n", group)
		for _, platform := range platforms {
			fmt.Printf("    • %s\n", platform)
		}
		fmt.Println()
	}

	// 动态显示 "all" 组合
	colorBold.Printf("  all (动态获取):\n")
	allPlatforms, err := getAllSupportedPlatforms()
	if err != nil {
		colorError.Printf("    ❌ 获取失败: %v\n", err)
		fmt.Printf("    💡 将使用静态备用列表\n")
	} else {
		colorInfo.Printf("    💡 共 %d 个平台，动态从 'go tool dist list' 获取\n", len(allPlatforms))
		// 显示前几个平台作为示例
		maxShow := 10
		for i, platform := range allPlatforms {
			if i >= maxShow {
				fmt.Printf("    • ... 还有 %d 个平台\n", len(allPlatforms)-maxShow)
				break
			}
			fmt.Printf("    • %s\n", platform)
		}
	}
	fmt.Println()
}

// cleanOutputDir 清理输出目录
func cleanOutputDir(outputDir string) error {
	if _, err := os.Stat(outputDir); err == nil {
		if config.Verbose >= 1 {
			colorInfo.Printf("🧹 清理输出目录: %s\n", outputDir)
		}
		return os.RemoveAll(outputDir)
	}
	return nil
}

// showVersion 显示版本信息
func showVersion() {
	fmt.Printf(`%s%sgogogo v2.0.0 - Go跨平台编译工具%s

%s特性:%s
  ✓ 支持多平台并行编译
  ✓ 智能重试机制
  ✓ 进度条显示
  ✓ 文件压缩
  ✓ Android环境支持
  ✓ 详细的日志输出

%s环境信息:%s
  Go版本: %s
  运行平台: %s/%s
  CPU核心: %d

`,
		colorTitle.Sprint(""), colorBold.Sprint(""), color.Reset,
		colorBold.Sprint(""), color.Reset,
		colorBold.Sprint(""), color.Reset,
		runtime.Version(),
		runtime.GOOS, runtime.GOARCH,
		runtime.NumCPU(),
	)
}

// showExamples 显示使用示例
func showExamples() {
	colorTitle.Println("📚 使用示例:")
	examples := []struct {
		desc string
		cmd  string
	}{{"交互式模式", "gogogo -i"},
		{"编译桌面平台", "gogogo -s main.go"},
		{"编译指定平台", "gogogo -s main.go -p windows/amd64,linux/amd64"},
		{"详细输出并压缩", "gogogo -s main.go -v 2 -c"},
		{"编译所有平台，清理输出目录", "gogogo -s main.go -p all --clean"},
		{"编译单个OS的本机架构", "gogogo -s main.go -p illumos"},
		{"编译单个OS的所有架构", "gogogo -s main.go -p illumos --all"},
		{"在Android设备上编译", "gogogo -s main.go -p android/arm64,android/arm"},
		{"强制编译iOS（在Windows上）", "gogogo -s main.go -p ios/arm64 --force"}, {"跳过所有确认提示", "gogogo -s main.go -p mobile --no-prompt"},
		{"安静模式编译", "gogogo -s main.go -v 0"},
		{"使用自定义ldflags", "gogogo -s main.go --ldflags \"-s -w\""},
		{"跳过CGO平台", "gogogo -s main.go -p all --skip-cgo"},
		{"指定NDK路径", "gogogo -s main.go -p android/arm64 --ndk-path \"C:\\Android\\sdk\\ndk\\25.2.9519653\""},
	}

	for _, example := range examples {
		colorBold.Printf("  • %s:\n", example.desc)
		colorInfo.Printf("    %s\n\n", example.cmd)
	}
}

// runInteractive 运行交互式编译模式
func runInteractive() error {
	colorTitle.Println("🔍 交互式编译模式")
	fmt.Println()

	scanner := bufio.NewScanner(os.Stdin)

	// 源文件
	if config.SourceFile == "" {
		colorBold.Print("请输入源文件路径: ")
		if scanner.Scan() {
			config.SourceFile = strings.TrimSpace(scanner.Text())
			if config.SourceFile == "" {
				return fmt.Errorf("源文件路径不能为空")
			}
			if _, err := os.Stat(config.SourceFile); os.IsNotExist(err) {
				return fmt.Errorf("源文件不存在: %s", config.SourceFile)
			}
		}
	} else {
		colorBold.Printf("源文件: %s\n", config.SourceFile)
	}

	// 输出目录
	colorBold.Printf("输出目录 [%s]: ", config.OutputDir)
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

	// #region Android NDK路径
	colorBold.Printf("Android NDK路径 (留空使用环境变量): ")
	if scanner.Scan() {
		ndkPath := strings.TrimSpace(scanner.Text())
		if ndkPath != "" {
			// 验证NDK路径
			if _, err := os.Stat(ndkPath); os.IsNotExist(err) {
				colorWarning.Printf("⚠️  警告: 指定的NDK路径不存在: %s\n", ndkPath)
				if askUserConfirm("是否仍然使用此路径?") {
					config.NDKPath = ndkPath
				}
			} else {
				// 检查NDK目录结构
				possibleDirs := []string{"toolchains", "platforms", "sources", "sysroot"}
				validNDK := false
				for _, dir := range possibleDirs {
					if _, err := os.Stat(filepath.Join(ndkPath, dir)); !os.IsNotExist(err) {
						validNDK = true
						break
					}
				}

				if !validNDK {
					colorWarning.Printf("⚠️  警告: 指定的路径可能不是有效的NDK根目录，缺少关键文件夹\n")
					if askUserConfirm("是否仍然使用此路径?") {
						config.NDKPath = ndkPath
					}
				} else {
					config.NDKPath = ndkPath
					ndkType := detectNDKType(ndkPath)
					if ndkType != "" {
						colorSuccess.Printf("✓ 检测到NDK类型: %s\n", ndkType)
					}
				}
			}
		}
	}
	// #endregion

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
	}
	fmt.Printf("  • 强制编译: %v\n", config.Force)

	// 确认开始编译
	fmt.Println()
	colorBold.Print("开始编译? (Y/n): ")
	if scanner.Scan() {
		response := strings.ToLower(strings.TrimSpace(scanner.Text()))
		if response == "n" || response == "no" {
			return fmt.Errorf("用户取消编译")
		}
	}

	// 禁用提示（因为已经在交互式模式中完成了选择）
	config.NoPrompt = true

	return nil
}

func main() {
	var rootCmd = &cobra.Command{
		Use: "gogogo", Short: "Go跨平台编译工具", Long: `gogogo v2.0.0 - 一个强大的Go跨平台编译工具

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
  • 默认仅编译本机架构 (如在amd64上仅编译amd64)
  • 使用 --all 标志编译该OS支持的所有架构

平台编译说明:
  • 桌面平台：支持直接编译
  • Android：推荐使用gomobile工具，或在Android环境中编译
  • iOS：仅支持在macOS上编译，需要Xcode和gomobile工具
  • WebAssembly：支持直接编译
  • 其他平台：大部分支持直接跨平台编译

注意: 如果遇到CGO相关错误，请使用 --skip-cgo 参数跳过问题平台。
使用 --force 参数可以强制尝试编译iOS平台（即使不在macOS上）。
使用 --no-prompt 参数可以跳过所有用户确认提示。
使用 --all 参数编译指定OS的所有架构（否则仅编译本机架构）。`, Example: `  # 编译桌面平台
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
  gogogo -s main.go -p all --clean`, RunE: func(cmd *cobra.Command, args []string) error {
			// 如果是交互式模式，运行交互式编译
			if config.Interactive {
				if err := runInteractive(); err != nil {
					return err
				}
			}

			// 检查必需参数
			if config.SourceFile == "" {
				return fmt.Errorf("请指定源文件 (-s)，使用 'gogogo --help' 查看帮助")
			}

			// 设置日志级别
			switch config.Verbose {
			case 0:
				logger.SetLevel(logrus.ErrorLevel)
			case 1:
				logger.SetLevel(logrus.InfoLevel)
			case 2:
				logger.SetLevel(logrus.DebugLevel)
			case 3:
				logger.SetLevel(logrus.TraceLevel)
			}

			// 检查Go环境
			if err := checkGoEnvironment(); err != nil {
				return err
			}

			// 检查源文件
			if _, err := os.Stat(config.SourceFile); err != nil {
				return fmt.Errorf("源文件不存在: %s", config.SourceFile)
			}

			// 设置默认二进制名称
			if config.BinaryName == "" {
				config.BinaryName = strings.TrimSuffix(filepath.Base(config.SourceFile), filepath.Ext(config.SourceFile))
			}

			// 清理输出目录
			if config.Clean {
				if err := cleanOutputDir(config.OutputDir); err != nil {
					return fmt.Errorf("清理输出目录失败: %v", err)
				}
			}

			// 解析目标平台
			targets := parsePlatforms(strings.Join(config.Platforms, ","))
			if len(targets) == 0 {
				return fmt.Errorf("没有找到有效的目标平台")
			}

			if config.Interactive {
				// 运行交互式编译模式
				if err := runInteractive(); err != nil {
					return err
				}
				targets = parsePlatforms(strings.Join(config.Platforms, ","))
			}

			// 执行编译
			return buildWithProgress(targets, config.SourceFile, config.OutputDir, config.BinaryName)
		},
	}

	// 添加子命令
	var listCmd = &cobra.Command{
		Use:   "list",
		Short: "列出所有支持的平台",
		Long:  "列出Go工具链支持的所有目标平台",
		Run: func(cmd *cobra.Command, args []string) {
			listPlatforms()
		},
	}

	var groupsCmd = &cobra.Command{
		Use:   "groups",
		Short: "列出所有平台组合",
		Long:  "列出预设的平台组合，可以直接使用这些组合名称",
		Run: func(cmd *cobra.Command, args []string) {
			listGroups()
		},
	}

	var versionCmd = &cobra.Command{
		Use:   "version",
		Short: "显示版本信息",
		Long:  "显示gogogo的版本信息和环境信息",
		Run: func(cmd *cobra.Command, args []string) {
			showVersion()
		},
	}

	var examplesCmd = &cobra.Command{
		Use:   "examples",
		Short: "显示使用示例",
		Long:  "显示详细的使用示例和常见用法",
		Run: func(cmd *cobra.Command, args []string) {
			showExamples()
		},
	}

	// 添加子命令到根命令
	rootCmd.AddCommand(listCmd, groupsCmd, versionCmd, examplesCmd)

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
	rootCmd.Flags().BoolVar(&config.Retry, "retry", true, "失败时重试")
	rootCmd.Flags().IntVar(&config.MaxRetries, "max-retries", 2, "最大重试次数")
	rootCmd.Flags().BoolVar(&config.Progress, "progress", true, "显示进度条")
	rootCmd.Flags().BoolVar(&config.All, "all", false, "编译指定OS的所有架构（否则仅编译本机架构）") // 高级选项
	rootCmd.Flags().StringVar(&config.LDFlags, "ldflags", "", "链接器标志 (如: \"-s -w\")")
	rootCmd.Flags().StringVar(&config.Tags, "tags", "", "构建标签")
	rootCmd.Flags().BoolVar(&config.SkipTests, "skip-tests", false, "跳过测试")
	rootCmd.Flags().BoolVar(&config.SkipCGO, "skip-cgo", false, "跳过需要CGO支持的平台")
	rootCmd.Flags().BoolVar(&config.Force, "force", false, "强制编译所有平台（包括在非macOS上编译iOS）")
	rootCmd.Flags().BoolVar(&config.NoPrompt, "no-prompt", false, "跳过所有用户确认提示")
	rootCmd.Flags().BoolVarP(&config.Interactive, "interactive", "i", false, "交互式模式")
	rootCmd.Flags().BoolVar(&config.NoCGO, "no-cgo", false, "完全禁用CGO（无论是否是CGO相关平台）")
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

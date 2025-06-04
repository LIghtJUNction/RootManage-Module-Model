package build

import (
	"bufio"
	"fmt"
	"log/slog"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/fatih/color"
	"github.com/lightjunction/rootmanager-module-model/gogogo/utils"
)

var (
	colorWarning = color.New(color.FgYellow, color.Bold)
	colorError   = color.New(color.FgRed, color.Bold)
	colorInfo    = color.New(color.FgBlue)
	colorSuccess = color.New(color.FgGreen, color.Bold)
	colorBold    = color.New(color.Bold)
)

// BuildConfig represents the configuration needed for building
type BuildConfig struct {
	SkipCGO  bool
	Verbose  int
	LDFlags  string
	Tags     string
	Force    bool
	NoPrompt bool
	NoCGO    bool
	NDKPath  string
	Compress bool
}

// BuildSingle 编译单个目标
func BuildSingle(target utils.BuildTarget, sourceFile, outputDir, binaryName string, config BuildConfig, logger *slog.Logger) error {
	// 跳过CGO相关平台
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

		// 处理iOS平台的CGO设置
		if config.NoCGO {
			if config.Verbose >= 1 {
				colorInfo.Printf("💡 使用--no-cgo标志，禁用iOS的CGO编译\n")
			}
			cmd.Env = append(cmd.Env, "CGO_ENABLED=0")
		} else {
			// 启用CGO并尝试配置clang环境		cmd.Env = append(cmd.Env, "CGO_ENABLED=1")

			// 查找系统中的clang安装
			clangInstallations := utils.FindSystemClang(logger)
			bestClang := utils.GetBestClangForTarget(target.Name, clangInstallations, logger)

			if bestClang.Path != "" {
				// 使用找到的clang配置编译环境
				if err := utils.SetupClangEnvironment(bestClang, logger); err != nil {
					if config.Verbose >= 1 {
						colorWarning.Printf("⚠️  配置clang环境失败: %v\n", err)
					}
				} else if config.Verbose >= 1 {
					colorSuccess.Printf("✓ 使用clang: %s (%s)\n", bestClang.Path, bestClang.Type)
				}
			} else {
				// 未找到clang，尝试传统方式
				if runtime.GOOS == "darwin" {
					if _, err := exec.LookPath("xcodebuild"); err != nil {
						colorWarning.Printf("⚠️  未找到clang安装，且Xcode不可用: %v\n", err)
						if config.Verbose >= 1 {
							colorInfo.Printf("💡 建议安装Xcode Command Line Tools: xcode-select --install\n")
						}
					}
				} else {
					colorWarning.Printf("⚠️  未找到适用的clang安装\n")
				}
			}

			if config.Verbose >= 1 {
				colorInfo.Printf("💡 iOS编译提示:\n")
				colorInfo.Printf("   • 推荐使用gomobile: go install golang.org/x/mobile/cmd/gomobile@latest\n")
				colorInfo.Printf("   • 初始化gomobile: gomobile init\n")
				colorInfo.Printf("   • 构建iOS应用: gomobile build -target=ios .\n")
				if runtime.GOOS != "darwin" {
					colorInfo.Printf("   • 跨平台iOS编译需要合适的clang工具链\n")
				}
			}
		}
	} else if target.GOOS == "android" {
		// Android平台处理
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
				if ndkHome == "" {
					ndkHome = os.Getenv("NDK_ROOT")
				}

				// 如果环境变量都没有设置，尝试自动查找系统NDK
				if ndkHome == "" {
					if config.Verbose >= 1 {
						colorInfo.Printf("💡 未设置NDK环境变量，尝试自动查找系统NDK...\n")
					}
					ndkPaths := utils.FindSystemNDK(logger)
					if len(ndkPaths) > 0 {
						ndkHome = ndkPaths[0] // 使用第一个找到的NDK路径
						colorSuccess.Printf("✓ 自动找到NDK路径: %s\n", ndkHome)

						// 显示如何永久设置环境变量的提示
						if config.Verbose >= 1 {
							colorInfo.Printf("💡 建议设置环境变量以避免每次自动搜索:\n")
							switch runtime.GOOS {
							case "windows":
								colorInfo.Printf("  • PowerShell: $env:ANDROID_NDK_HOME = \"%s\"\n", ndkHome)
								colorInfo.Printf("  • CMD: set ANDROID_NDK_HOME=%s\n", ndkHome)
								colorInfo.Printf("  • 系统环境变量: 右键\"此电脑\" -> 属性 -> 高级系统设置 -> 环境变量\n")
							default:
								colorInfo.Printf("  • Bash/Zsh: export ANDROID_NDK_HOME=\"%s\"\n", ndkHome)
								colorInfo.Printf("  • 永久配置: 添加到 ~/.bashrc 或 ~/.zshrc 文件\n")
							}
						}
					}
				}
			}

			if ndkHome == "" {
				if !config.Force && !config.NoPrompt {
					if config.Verbose >= 1 {
						colorError.Printf("⚠️  编译Android平台需要设置Android NDK环境\n")
						colorInfo.Printf("💡 未检测到NDK路径或环境变量\n")

						// 询问用户是否要提供NDK路径
						if utils.AskUserConfirm("是否手动提供Android NDK路径?", config.NoPrompt) {
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
										if !utils.AskUserConfirm("是否继续使用此路径?", config.NoPrompt) {
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
				if err := utils.SetupNDKEnvironment(ndkHome, target.GOARCH, &cmd.Env); err != nil {
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
		logger.Info("执行命令", "cmd", strings.Join(cmd.Args, " "))
		logger.Info("环境变量", "GOOS", target.GOOS, "GOARCH", target.GOARCH)
	}

	// 执行编译
	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("编译失败 [%s]: %v\n输出: %s", target.Name, err, string(output))
	}

	// 压缩文件
	if config.Compress {
		if err := utils.CompressFile(outputPath); err != nil {
			logger.Warn("压缩文件失败", "target", target.Name, "error", err)
		}
	}

	return nil
}

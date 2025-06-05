package gobuild

import (
	"bufio"
	"fmt"
	"log/slog"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/lightjunction/rootmanager-module-model/gogogo/config"
	"github.com/lightjunction/rootmanager-module-model/gogogo/utils"
)

// BuildSingle 编译单个目标
func BuildSingle(target config.BuildTarget, sourceFile, outputDir, binaryName string, buildConfig config.BuildConfig, logger *slog.Logger) error { // 获取颜色函数
	_, colorSuccess, colorError, colorWarning, colorInfo, colorBold := config.GetColors()
	colorEmoji, colorCommand, _, _, colorPlatform, colorProgress, colorSubtle, colorHighlight := config.GetEnhancedColors()

	// 显示开始编译的美化信息
	if buildConfig.Verbose >= 1 {
		colorProgress.Printf("\n" + strings.Repeat("─", 60) + "\n")
		colorEmoji.Print("🎯 ")
		colorHighlight.Printf("开始编译目标: ")
		colorPlatform.Printf("%s\n", target.Name)
		colorProgress.Printf(strings.Repeat("─", 60) + "\n")
	}
	// 跳过CGO相关平台
	if buildConfig.SkipCGO && (target.GOOS == "android" || target.GOOS == "ios") {
		if buildConfig.Verbose >= 1 {
			colorEmoji.Print("⚠️  ")
			colorWarning.Printf("跳过需要CGO支持的平台: ")
			colorPlatform.Printf("%s ", target.Name)
			colorSubtle.Printf("(使用 ")
			colorCommand.Printf("--skip-cgo=false")
			colorSubtle.Printf(" 强制编译)\n")
		}
		return config.ErrSkipped
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

	if buildConfig.LDFlags != "" {
		args = append(args, "-ldflags", buildConfig.LDFlags)
	}

	if buildConfig.Tags != "" {
		args = append(args, "-tags", buildConfig.Tags)
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
	} else if target.GOOS == "ios" { // iOS平台特殊处理
		if runtime.GOOS != "darwin" {
			if !buildConfig.Force {
				if buildConfig.Verbose >= 1 {
					colorEmoji.Print("⚠️  ")
					colorWarning.Printf("跳过iOS平台: ")
					colorSubtle.Printf("只能在macOS上编译 (使用 ")
					colorCommand.Printf("--force")
					colorSubtle.Printf(" 强制尝试)\n")
				}
				return config.ErrSkipped
			} else {
				colorEmoji.Print("⚠️  ")
				colorError.Printf("警告: 在非macOS系统上强制编译iOS，可能会失败!\n")
			}
		}

		// 处理iOS平台的CGO设置
		if buildConfig.NoCGO {
			if buildConfig.Verbose >= 1 {
				colorInfo.Printf("💡 使用--no-cgo标志，禁用iOS的CGO编译\n")
			}
			cmd.Env = append(cmd.Env, "CGO_ENABLED=0")
		} else {
			// 启用CGO并尝试配置clang环境
			cmd.Env = append(cmd.Env, "CGO_ENABLED=1") // 查找系统中的clang安装
			clangInstallations := utils.FindSystemClang(logger)
			bestClang := utils.GetBestClangForTarget(target.Name, clangInstallations, logger)

			if bestClang.Path != "" {
				// 使用找到的clang配置编译环境
				if err := utils.SetupClangEnvironment(bestClang, logger); err != nil {
					if buildConfig.Verbose >= 1 {
						colorWarning.Printf("⚠️  配置clang环境失败: %v\n", err)
					}
				} else if buildConfig.Verbose >= 1 {
					colorSuccess.Printf("✓ 使用clang: %s (%s)\n", bestClang.Path, bestClang.Type)
				}
			} else {
				// 未找到clang，尝试传统方式
				if runtime.GOOS == "darwin" {
					if _, err := exec.LookPath("xcodebuild"); err != nil {
						colorWarning.Printf("⚠️  未找到clang安装，且Xcode不可用: %v\n", err)
						if buildConfig.Verbose >= 1 {
							colorInfo.Printf("💡 建议安装Xcode Command Line Tools: xcode-select --install\n")
						}
					}
				} else {
					colorWarning.Printf("⚠️  未找到适用的clang安装\n")
				}
			}

			if buildConfig.Verbose >= 1 {
				colorInfo.Printf("💡 iOS编译提示:\n")
				colorInfo.Printf("   • 推荐使用gomobile: go install golang.org/x/mobile/cmd/gomobile@latest\n")
				colorInfo.Printf("   • 初始化gomobile: gomobile init\n")
				colorInfo.Printf("   • 构建iOS应用: gomobile build -target=ios .\n")
				if runtime.GOOS != "darwin" {
					colorInfo.Printf("   • 跨平台iOS编译需要合适的clang工具链\n")
				}
			}
		}
	} else if target.GOOS == "android" { // Android平台处理
		if buildConfig.Verbose >= 1 && logger != nil {
			logger.Info("Android平台建议", "message", "建议使用gomobile工具进行构建")
			if buildConfig.Verbose >= 2 {
				colorWarning.Printf("⚠️  Android平台建议使用gomobile工具进行构建\n")
				colorInfo.Printf("💡 安装gomobile: go install golang.org/x/mobile/cmd/gomobile@latest\n")
				colorInfo.Printf("💡 构建Android应用: gomobile build -target=android .\n")
				colorInfo.Printf("✓ 自动继续使用标准Go工具链编译Android平台\n")
			}
		}

		// 处理Android编译选项
		var ndkHome string

		// 使用NoCGO标志完全禁用CGO（适用于纯Go代码）
		if buildConfig.NoCGO {
			if buildConfig.Verbose >= 1 && logger != nil {
				logger.Info("禁用CGO", "platform", target.Name)
				if buildConfig.Verbose >= 2 {
					colorInfo.Printf("💡 使用--no-cgo标志，禁用Android的CGO编译\n")
				}
			}
			cmd.Env = append(cmd.Env, "CGO_ENABLED=0")
			// 不再提前返回，让编译继续进行
		} else if runtime.GOOS != "android" { // 仅在交叉编译时检查NDK环境
			// 优先使用命令行指定的NDK路径
			if buildConfig.NDKPath != "" {
				ndkHome = buildConfig.NDKPath
				if buildConfig.Verbose >= 1 {
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
				} // 验证环境变量指定的NDK路径是否有效
				if ndkHome != "" {
					if _, err := os.Stat(ndkHome); os.IsNotExist(err) {
						colorWarning.Printf("⚠️  环境变量指定的NDK路径不存在: %s\n", ndkHome)
						colorInfo.Printf("📝 请检查以下环境变量设置:\n")
						if env := os.Getenv("ANDROID_NDK_HOME"); env != "" {
							colorInfo.Printf("  • ANDROID_NDK_HOME = %s\n", env)
						}
						if env := os.Getenv("ANDROID_NDK_ROOT"); env != "" {
							colorInfo.Printf("  • ANDROID_NDK_ROOT = %s\n", env)
						}
						if env := os.Getenv("NDK_ROOT"); env != "" {
							colorInfo.Printf("  • NDK_ROOT = %s\n", env)
						}
						colorInfo.Printf("💡 将回退到智能发现模式...\n")
						ndkHome = "" // 清空无效路径，触发智能发现
					} else {
						// 检查路径是否包含NDK的关键目录
						requiredDirs := []string{"toolchains", "platforms", "sources"}
						missingDirs := []string{}
						for _, dir := range requiredDirs {
							if _, err := os.Stat(filepath.Join(ndkHome, dir)); os.IsNotExist(err) {
								missingDirs = append(missingDirs, dir)
							}
						}

						if len(missingDirs) > 0 {
							colorWarning.Printf("⚠️  环境变量指定的路径缺少NDK关键目录: %v\n", missingDirs)
							colorInfo.Printf("📁 当前路径: %s\n", ndkHome)
							colorInfo.Printf("🔍 请确认这是正确的NDK根目录\n")
							colorInfo.Printf("💡 将回退到智能发现模式...\n")
							ndkHome = "" // 清空无效路径，触发智能发现
						} else {
							colorSuccess.Printf("✓ 环境变量NDK路径验证通过: %s\n", ndkHome)
						}
					}
				}

				// 如果环境变量都没有设置或验证失败，尝试自动查找系统NDK
				if ndkHome == "" {
					if buildConfig.Verbose >= 1 {
						colorInfo.Printf("💡 未设置NDK环境变量，尝试自动查找系统NDK...\n")
					}
					ndkPaths := utils.FindSystemNDK(logger)
					if len(ndkPaths) > 0 {
						ndkHome = ndkPaths[0]                             // 使用找到的第一个NDK路径
						colorSuccess.Printf("✓ 自动找到NDK路径: %s\n", ndkHome) // 显示如何永久设置环境变量的提示
						if buildConfig.Verbose >= 1 {
							colorInfo.Printf("💡 建议设置正确的环境变量以避免每次自动搜索:\n")
							switch runtime.GOOS {
							case "windows":
								colorInfo.Printf("  • PowerShell: $env:ANDROID_NDK_HOME = \"%s\"\n", ndkHome)
								colorInfo.Printf("  • CMD: set ANDROID_NDK_HOME=%s\n", ndkHome)
								colorInfo.Printf("  • 系统环境变量: 右键\"此电脑\" -> 属性 -> 高级系统设置 -> 环境变量\n")
								colorInfo.Printf("  • 验证设置: Get-ChildItem Env: | Where-Object { $_.Name -like \"*NDK*\" }\n")
							default:
								colorInfo.Printf("  • Bash/Zsh: export ANDROID_NDK_HOME=\"%s\"\n", ndkHome)
								colorInfo.Printf("  • 永久配置: 添加到 ~/.bashrc 或 ~/.zshrc 文件\n")
								colorInfo.Printf("  • 验证设置: echo $ANDROID_NDK_HOME\n")
							}
						}
					}
				}
			}

			if ndkHome == "" {
				if !buildConfig.Force && !buildConfig.NoPrompt {
					if buildConfig.Verbose >= 1 {
						colorError.Printf("⚠️  编译Android平台需要设置Android NDK环境\n")
						colorInfo.Printf("💡 未检测到NDK路径或环境变量\n")

						// 询问用户是否要提供NDK路径
						if utils.AskUserConfirm("是否手动提供Android NDK路径?", buildConfig.NoPrompt) {
							colorBold.Print("请输入Android NDK根目录路径: ")
							scanner := bufio.NewScanner(os.Stdin)
							if scanner.Scan() {
								ndkPath := strings.TrimSpace(scanner.Text())
								if ndkPath != "" {
									// 检查路径是否存在
									if _, err := os.Stat(ndkPath); os.IsNotExist(err) {
										colorError.Printf("❌ 指定的NDK路径不存在: %s\n", ndkPath)
										return config.ErrSkipped
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
										if !utils.AskUserConfirm("是否继续使用此路径?", buildConfig.NoPrompt) {
											return config.ErrSkipped
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
									return config.ErrSkipped
								}
							} else {
								colorWarning.Printf("⚠️  读取输入失败，跳过编译\n")
								return config.ErrSkipped
							}
						} else {
							colorInfo.Printf("💡 跳过Android编译。您可以使用以下选项之一:\n")
							colorInfo.Printf("  1. 使用 --ndk-path 参数指定NDK路径\n")
							colorInfo.Printf("  2. 设置ANDROID_NDK_HOME环境变量指向NDK根目录\n")
							colorInfo.Printf("  3. 使用 --force 参数强制尝试编译\n")
							colorInfo.Printf("  4. 使用 --no-cgo 参数禁用CGO编译（仅适用于纯Go代码）\n")
							return config.ErrSkipped
						}
					} else {
						return config.ErrSkipped
					}
				} else if buildConfig.Force {
					colorError.Printf("⚠️  警告: 未设置NDK路径，强制尝试编译可能会失败！\n")
				} else {
					// 静默模式，没有force标志，直接跳过
					return config.ErrSkipped
				}
			} else { // 使用智能环境变量设置
				if buildConfig.Verbose >= 3 && logger != nil {
					logger.Debug("开始设置NDK环境变量", "path", ndkHome, "arch", target.GOARCH)
				} // 在使用NDK环境前记录当前环境变量
				if buildConfig.Verbose >= 3 && logger != nil {
					utils.PrintEnvironmentVars(cmd.Env, "NDK设置前", logger)
				}

				if err := utils.SetupNDKEnvironment(ndkHome, target.GOARCH, &cmd.Env, logger); err != nil {
					if buildConfig.Verbose >= 1 && logger != nil {
						logger.Warn("NDK环境设置失败", "error", err.Error(), "ndkPath", ndkHome)
						if buildConfig.Verbose >= 2 {
							colorWarning.Printf("⚠️  设置NDK环境变量失败: %v\n", err)
							colorInfo.Printf("💡 将使用传统方式设置NDK环境\n")
						}
					}
					// 如果智能设置失败，回退到简单的环境变量设置
					ccPath := filepath.Join(ndkHome, "toolchains", "llvm", "prebuilt", runtime.GOOS+"-x86_64", "bin", "clang")
					cxxPath := filepath.Join(ndkHome, "toolchains", "llvm", "prebuilt", runtime.GOOS+"-x86_64", "bin", "clang++")
					includePath := filepath.Join(ndkHome, "toolchains", "llvm", "prebuilt", runtime.GOOS+"-x86_64", "sysroot", "usr", "include")

					// 在Windows上尝试添加.cmd或.exe后缀
					if runtime.GOOS == "windows" {
						if _, err := os.Stat(ccPath); os.IsNotExist(err) {
							if _, err := os.Stat(ccPath + ".cmd"); err == nil {
								ccPath += ".cmd"
								cxxPath += ".cmd"
							} else if _, err := os.Stat(ccPath + ".exe"); err == nil {
								ccPath += ".exe"
								cxxPath += ".exe"
							}
						}
					}

					cmd.Env = append(cmd.Env,
						"ANDROID_NDK_HOME="+ndkHome,
						"ANDROID_NDK_ROOT="+ndkHome,
						"CGO_ENABLED=1",
						"CC="+ccPath,
						"CXX="+cxxPath,
						"CGO_CFLAGS=-I"+includePath)

					// 记录回退环境变量
					if buildConfig.Verbose >= 3 && logger != nil {
						logger.Debug("使用回退的环境变量设置",
							"CC", ccPath,
							"CXX", cxxPath,
							"CGO_CFLAGS", "-I"+includePath)

						// 检查编译器文件是否存在
						if _, err := os.Stat(ccPath); os.IsNotExist(err) {
							logger.Error("回退的CC编译器文件不存在", "path", ccPath)
						}
						if _, err := os.Stat(cxxPath); os.IsNotExist(err) {
							logger.Error("回退的CXX编译器文件不存在", "path", cxxPath)
						}
					}
				} else if buildConfig.Verbose >= 2 && logger != nil {
					logger.Info("NDK环境设置成功", "path", ndkHome)
					if buildConfig.Verbose >= 3 {
						utils.PrintEnvironmentVars(cmd.Env, "NDK设置后", logger)
					}
					colorSuccess.Printf("✓ 已根据NDK类型和宿主系统智能配置环境变量\n")
				}
			}
		} // 设置CGO_ENABLED=1
		cmd.Env = append(cmd.Env, "CGO_ENABLED=1")

		// 在设置CGO_ENABLED后检查环境变量
		if buildConfig.Verbose >= 3 && logger != nil {
			utils.PrintEnvironmentVars(cmd.Env, "Android设置CGO_ENABLED=1后", logger)
		}

		// 不再重复打印提示
		if buildConfig.Verbose >= 2 && runtime.GOOS == "windows" && logger != nil {
			colorInfo.Printf("💡 Windows上可以直接编译Android/arm64平台\n")
		}

		// 为Android设置编译标志，尝试静态链接
		if buildConfig.LDFlags == "" {
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

		// 在设置CGO_ENABLED后检查环境变量
		if buildConfig.Verbose >= 3 && logger != nil {
			utils.PrintEnvironmentVars(cmd.Env, "非Android平台设置CGO_ENABLED=0后", logger)
		}
	}
	if buildConfig.Verbose >= 2 && logger != nil {
		// 使用颜色输出执行命令
		colorInfo.Printf("🔧 执行命令: %s\n", strings.Join(cmd.Args, " "))
		colorInfo.Printf("🎯 目标平台: %s/%s\n", target.GOOS, target.GOARCH)

		// 同时记录到日志
		logger.Info("执行命令", "cmd", strings.Join(cmd.Args, " "))
		logger.Info("环境变量", "GOOS", target.GOOS, "GOARCH", target.GOARCH)
	}

	// 在详细模式下打印完整的环境变量，方便调试
	if buildConfig.Verbose >= 3 && logger != nil {
		// 打印所有环境变量
		utils.PrintEnvironmentVars(cmd.Env, "编译前最终环境变量", logger)
	}
	// 执行编译
	output, err := cmd.CombinedOutput()
	if err != nil {
		return fmt.Errorf("编译失败 [%s]: %v\n输出: %s", target.Name, err, string(output))
	}

	// 压缩文件
	if buildConfig.Compress {
		if err := compressExecutable(outputPath, buildConfig.Verbose); err != nil {
			logger.Warn("压缩文件失败", "target", target.Name, "error", err)
		}
	}

	return nil
}

// setupAndroidEnvironment 设置Android编译环境
func setupAndroidEnvironment(target config.BuildTarget, buildConfig config.BuildConfig, cmd *exec.Cmd) error {
	// 查找NDK路径
	ndkPath := buildConfig.NDKPath
	if ndkPath == "" {
		ndkPath = os.Getenv("ANDROID_NDK_ROOT")
		if ndkPath == "" {
			ndkPath = os.Getenv("ANDROID_NDK_HOME")
		}
	}

	if ndkPath == "" {
		return fmt.Errorf("未找到Android NDK，请设置ANDROID_NDK_ROOT环境变量或使用--ndk-path参数")
	}

	// 确定工具链前缀
	var toolchainPrefix string
	switch target.GOARCH {
	case "arm":
		toolchainPrefix = "arm-linux-androideabi"
	case "arm64":
		toolchainPrefix = "aarch64-linux-android"
	case "386":
		toolchainPrefix = "i686-linux-android"
	case "amd64":
		toolchainPrefix = "x86_64-linux-android"
	default:
		return fmt.Errorf("不支持的Android架构: %s", target.GOARCH)
	}

	// API级别
	apiLevel := "21" // Android 5.0+

	// 工具链路径
	var toolchainDir string
	if runtime.GOOS == "windows" {
		toolchainDir = filepath.Join(ndkPath, "toolchains", "llvm", "prebuilt", "windows-x86_64")
	} else if runtime.GOOS == "darwin" {
		toolchainDir = filepath.Join(ndkPath, "toolchains", "llvm", "prebuilt", "darwin-x86_64")
	} else {
		toolchainDir = filepath.Join(ndkPath, "toolchains", "llvm", "prebuilt", "linux-x86_64")
	}

	// 设置编译器路径
	ccPath := filepath.Join(toolchainDir, "bin", toolchainPrefix+apiLevel+"-clang")
	cxxPath := filepath.Join(toolchainDir, "bin", toolchainPrefix+apiLevel+"-clang++")

	if runtime.GOOS == "windows" {
		ccPath += ".cmd"
		cxxPath += ".cmd"
	}

	// 验证编译器是否存在
	if _, err := os.Stat(ccPath); os.IsNotExist(err) {
		return fmt.Errorf("Android编译器不存在: %s", ccPath)
	}

	// 设置环境变量
	cmd.Env = append(cmd.Env, "CC="+ccPath)
	cmd.Env = append(cmd.Env, "CXX="+cxxPath)

	return nil
}

// compressExecutable 压缩可执行文件
func compressExecutable(path string, verbose int) error {
	// 获取颜色函数
	_, colorSuccess, _, colorWarning, colorInfo, _ := config.GetColors()
	colorEmoji, _, colorPath, colorSize, _, _, _, _ := config.GetEnhancedColors()

	if verbose >= 1 {
		colorEmoji.Print("📦 ")
		colorInfo.Printf("尝试压缩: ")
		colorPath.Printf("%s\n", path)
	}

	// 获取原始文件大小
	originalInfo, err := os.Stat(path)
	if err != nil {
		return err
	}
	originalSize := originalInfo.Size()

	// 尝试使用upx压缩
	cmd := exec.Command("upx", "--best", path)
	if err := cmd.Run(); err != nil {
		if verbose >= 2 {
			colorWarning.Printf("⚠️  UPX压缩失败: %v\n", err)
			colorInfo.Printf("💡 提示: 请确保已安装UPX工具\n")
		}
		return err
	}

	// 获取压缩后文件大小
	compressedInfo, err := os.Stat(path)
	if err == nil {
		compressedSize := compressedInfo.Size()
		ratio := float64(compressedSize) / float64(originalSize) * 100

		if verbose >= 1 {
			colorEmoji.Print("✅ ")
			colorSuccess.Printf("压缩完成: ")
			colorPath.Printf("%s\n", path)

			// 格式化文件大小
			var originalSizeStr, compressedSizeStr string
			if originalSize < 1024 {
				originalSizeStr = fmt.Sprintf("%d B", originalSize)
			} else if originalSize < 1024*1024 {
				originalSizeStr = fmt.Sprintf("%.1f KB", float64(originalSize)/1024)
			} else {
				originalSizeStr = fmt.Sprintf("%.1f MB", float64(originalSize)/(1024*1024))
			}

			if compressedSize < 1024 {
				compressedSizeStr = fmt.Sprintf("%d B", compressedSize)
			} else if compressedSize < 1024*1024 {
				compressedSizeStr = fmt.Sprintf("%.1f KB", float64(compressedSize)/1024)
			} else {
				compressedSizeStr = fmt.Sprintf("%.1f MB", float64(compressedSize)/(1024*1024))
			}

			colorInfo.Printf("   原始大小: ")
			colorSize.Printf("%s", originalSizeStr)
			colorInfo.Printf(" → 压缩后: ")
			colorSize.Printf("%s", compressedSizeStr)
			colorInfo.Printf(" (压缩率: ")
			colorSize.Printf("%.1f%%", ratio)
			colorInfo.Printf(")\n")
		}
	}

	return nil
}

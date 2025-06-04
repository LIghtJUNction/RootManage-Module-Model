package gobuild

import (
	"errors"
	"fmt"
	"log/slog"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
	"time"

	"github.com/schollz/progressbar/v3"

	"github.com/lightjunction/rootmanager-module-model/gogogo/config"
)

// BuildWithProgress 带进度条的编译
func BuildWithProgress(targets []config.BuildTarget, sourceFile, outputDir, binaryName string, buildConfig config.BuildConfig, progressConfig config.ProgressConfig, logger *slog.Logger) error {
	// 获取进度条颜色
	colorInfoProgress, colorSuccessProgress, colorErrorProgress, colorWarningProgress := config.GetProgressColors() // 获取增强颜色
	colorEmoji, _, colorPath, _, colorPlatform, colorProgress, colorSubtle, colorHighlight := config.GetEnhancedColors()

	if progressConfig.Verbose >= 1 {
		// 美化开始信息
		fmt.Print("\n")
		colorProgress.Printf(strings.Repeat("═", 60) + "\n")
		colorEmoji.Print("🚀 ")
		colorHighlight.Printf("开始批量编译任务\n")
		colorProgress.Printf(strings.Repeat("═", 60) + "\n")
		colorInfoProgress.Printf("📊 目标平台数量: ")
		colorPlatform.Printf("%d\n", len(targets))
		colorInfoProgress.Printf("📝 源文件: ")
		colorPath.Printf("%s\n", sourceFile)
		colorInfoProgress.Printf("📁 输出目录: ")
		colorPath.Printf("%s\n", outputDir)
		colorInfoProgress.Printf("🎯 二进制名称: ")
		colorHighlight.Printf("%s\n", binaryName)
		colorProgress.Printf(strings.Repeat("═", 60) + "\n")
	}

	var bar *progressbar.ProgressBar
	if progressConfig.Progress && progressConfig.Verbose >= 1 {
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
	if !progressConfig.Parallel {
		maxWorkers = 1
	}
	semaphore := make(chan struct{}, maxWorkers)
	for _, target := range targets {
		wg.Add(1)
		go func(t config.BuildTarget) {
			defer wg.Done()

			semaphore <- struct{}{}
			defer func() { <-semaphore }()

			// 重试逻辑
			var err error
			for attempt := 0; attempt <= progressConfig.MaxRetries; attempt++ {
				err = BuildSingle(t, sourceFile, outputDir, binaryName, buildConfig, logger)
				if err == nil {
					break
				}
				if attempt < progressConfig.MaxRetries && progressConfig.Retry {
					if progressConfig.Verbose >= 2 {
						logger.Warn("编译失败，正在重试", "target", t.Name, "attempt", attempt+1, "error", err)
					}
					time.Sleep(time.Second * time.Duration(attempt+1))
				}
			}

			mu.Lock()
			if err != nil {
				if errors.Is(err, config.ErrSkipped) {
					// 跳过的平台不计入错误
					skipped = append(skipped, t.Name)
					if progressConfig.Verbose >= 1 {
						colorEmoji.Print("⏭️ ")
						colorPlatform.Printf("%s ", t.Name)
						colorSubtle.Printf("(跳过)\n")
					}
				} else {
					errs = append(errs, fmt.Errorf("[%s] %v", t.Name, err))
				}
			} else {
				successful = append(successful, t.Name)
				if progressConfig.Verbose >= 1 {
					colorEmoji.Print("✓ ")
					colorPlatform.Printf("%s ", t.Name)
					colorSuccessProgress.Printf("(成功)\n")
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
		colorErrorProgress.Println("\n❌ 编译过程中出现错误:")
		for _, err := range errs {
			colorErrorProgress.Printf("  • %v\n", err)
		}
		return fmt.Errorf("编译失败: %d个目标出现错误", len(errs))
	}

	if progressConfig.Verbose >= 1 {
		if len(successful) > 0 {
			colorSuccessProgress.Printf("\n🎉 编译完成! 共编译 %d 个目标平台\n", len(successful))

			// 显示输出目录信息
			colorInfoProgress.Printf("\n📁 构建输出目录:\n")
			colorInfoProgress.Printf("  绝对路径: %s\n", outputDir)

			// 显示相对路径（如果比绝对路径短）
			if relPath, err := filepath.Rel(".", outputDir); err == nil && len(relPath) < len(outputDir) {
				colorInfoProgress.Printf("  相对路径: %s\n", relPath)
			}

			// 获取目录大小
			totalSize := int64(0)
			fileCount := 0
			err := filepath.Walk(outputDir, func(path string, info os.FileInfo, err error) error {
				if err != nil {
					return nil // 忽略错误，继续遍历
				}
				if !info.IsDir() {
					totalSize += info.Size()
					fileCount++
				}
				return nil
			})

			if err == nil {
				// 格式化总大小
				var totalSizeStr string
				if totalSize < 1024 {
					totalSizeStr = fmt.Sprintf("%d B", totalSize)
				} else if totalSize < 1024*1024 {
					totalSizeStr = fmt.Sprintf("%.1f KB", float64(totalSize)/1024)
				} else {
					totalSizeStr = fmt.Sprintf("%.1f MB", float64(totalSize)/(1024*1024))
				}
				colorInfoProgress.Printf("  总大小: %s (%d 个文件)\n", totalSizeStr, fileCount)
			}

			colorInfoProgress.Printf("\n📋 成功构建的目标:\n")
			// 显示详细的文件信息
			for _, targetName := range successful {
				// 构建文件路径
				parts := strings.Split(targetName, "/")
				if len(parts) == 2 {
					goos, goarch := parts[0], parts[1]

					// 确定文件扩展名
					ext := ""
					if goos == "windows" {
						ext = ".exe"
					}

					// 构建完整的文件路径
					filePath := filepath.Join(outputDir, goos, goarch, binaryName+ext)

					// 检查文件是否存在并获取文件信息
					if fileInfo, err := os.Stat(filePath); err == nil {
						// 格式化文件大小
						size := fileInfo.Size()
						var sizeStr string
						if size < 1024 {
							sizeStr = fmt.Sprintf("%d B", size)
						} else if size < 1024*1024 {
							sizeStr = fmt.Sprintf("%.1f KB", float64(size)/1024)
						} else {
							sizeStr = fmt.Sprintf("%.1f MB", float64(size)/(1024*1024))
						}

						// 显示相对路径
						if relFilePath, err := filepath.Rel(".", filePath); err == nil && len(relFilePath) < len(filePath) {
							colorInfoProgress.Printf("  ✓ %s → %s (%s)\n", targetName, relFilePath, sizeStr)
						} else {
							colorInfoProgress.Printf("  ✓ %s → %s (%s)\n", targetName, filePath, sizeStr)
						}
					} else {
						colorInfoProgress.Printf("  ✓ %s (文件未找到)\n", targetName)
					}
				} else {
					colorInfoProgress.Printf("  ✓ %s\n", targetName)
				}
			}

			// 提示如何查看构建结果
			colorInfoProgress.Printf("\n💡 查看构建结果:\n")
			if runtime.GOOS == "windows" {
				colorInfoProgress.Printf("  • 打开目录: explorer \"%s\"\n", outputDir)
				colorInfoProgress.Printf("  • 命令行查看: dir \"%s\" /s\n", outputDir)
				if relPath, err := filepath.Rel(".", outputDir); err == nil {
					colorInfoProgress.Printf("  • 快速查看: dir \"%s\" /s\n", relPath)
				}
			} else {
				colorInfoProgress.Printf("  • 查看文件: ls -la \"%s\"\n", outputDir)
				colorInfoProgress.Printf("  • 递归查看: find \"%s\" -type f -exec ls -lh {} \\;\n", outputDir)
				if relPath, err := filepath.Rel(".", outputDir); err == nil {
					colorInfoProgress.Printf("  • 快速查看: ls -la \"%s\"\n", relPath)
				}
			}
		}
		if len(skipped) > 0 {
			colorWarningProgress.Printf("⏭️ 跳过 %d 个目标平台: %s\n", len(skipped), strings.Join(skipped, ", "))
		}
		if len(successful) == 0 && len(skipped) > 0 {
			colorInfoProgress.Printf("💡 所有平台都被跳过，没有实际编译任何目标\n")
		}
	}

	return nil
}

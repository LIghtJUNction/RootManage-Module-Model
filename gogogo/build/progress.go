package build

import (
	"errors"
	"fmt"
	"log/slog"
	"runtime"
	"strings"
	"sync"
	"time"

	"github.com/fatih/color"
	"github.com/schollz/progressbar/v3"

	"github.com/lightjunction/rootmanager-module-model/gogogo/utils"
)

var (
	colorInfoProgress    = color.New(color.FgBlue)
	colorSuccessProgress = color.New(color.FgGreen, color.Bold)
	colorErrorProgress   = color.New(color.FgRed, color.Bold)
	colorWarningProgress = color.New(color.FgYellow, color.Bold)
)

// ProgressConfig represents the configuration for progress tracking
type ProgressConfig struct {
	Verbose    int
	Progress   bool
	Parallel   bool
	Retry      bool
	MaxRetries int
}

// BuildWithProgress 带进度条的编译
func BuildWithProgress(targets []utils.BuildTarget, sourceFile, outputDir, binaryName string, buildConfig BuildConfig, progressConfig ProgressConfig, logger *slog.Logger) error {
	if progressConfig.Verbose >= 1 {
		colorInfoProgress.Printf("🚀 开始编译 %d 个目标平台\n", len(targets))
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
		go func(t utils.BuildTarget) {
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
				if errors.Is(err, ErrSkipped) {
					// 跳过的平台不计入错误
					skipped = append(skipped, t.Name)
					if progressConfig.Verbose >= 1 {
						colorWarningProgress.Printf("⏭️ %s (跳过)\n", t.Name)
					}
				} else {
					errs = append(errs, fmt.Errorf("[%s] %v", t.Name, err))
				}
			} else {
				successful = append(successful, t.Name)
				if progressConfig.Verbose >= 1 {
					colorSuccessProgress.Printf("✓ %s\n", t.Name)
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

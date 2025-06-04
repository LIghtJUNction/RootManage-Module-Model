package commands

import (
	"fmt"
	"runtime"

	"github.com/fatih/color"
)

// ShowVersion 显示版本信息
func ShowVersion() {
	colorTitle := color.New(color.FgHiCyan, color.Bold)
	colorBold := color.New(color.Bold)

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

// ShowExamples 显示使用示例
func ShowExamples() {
	colorTitle := color.New(color.FgHiCyan, color.Bold)
	colorBold := color.New(color.Bold)
	colorInfo := color.New(color.FgHiBlue)

	colorTitle.Println("📚 使用示例:")
	examples := []struct {
		desc string
		cmd  string
	}{
		{"交互式模式", "gogogo -i"},
		{"编译桌面平台", "gogogo -s main.go"},
		{"编译指定平台", "gogogo -s main.go -p windows/amd64,linux/amd64"},
		{"详细输出并压缩", "gogogo -s main.go -v 2 -c"},
		{"编译所有平台，清理输出目录", "gogogo -s main.go -p all --clean"},
		{"编译单个OS的本机架构", "gogogo -s main.go -p illumos"},
		{"编译单个OS的所有架构", "gogogo -s main.go -p illumos --all"},
		{"在Android设备上编译", "gogogo -s main.go -p android/arm64,android/arm"},
		{"强制编译iOS（在Windows上）", "gogogo -s main.go -p ios/arm64 --force"},
		{"跳过所有确认提示", "gogogo -s main.go -p mobile --no-prompt"},
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

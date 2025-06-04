package config

import (
	"errors"
	"log/slog"

	"github.com/fatih/color"
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

// ProgressConfig represents the configuration for progress tracking
type ProgressConfig struct {
	Verbose    int
	Progress   bool
	Parallel   bool
	Retry      bool
	MaxRetries int
}

// BuildTarget 构建目标
type BuildTarget struct {
	GOOS   string
	GOARCH string
	Name   string
}

// ClangInstallation 表示一个clang安装
type ClangInstallation struct {
	Path     string
	Version  string
	Type     string
	Priority int
}

// 全局变量
var (
	config Config
	logger *slog.Logger
	// 颜色定义 - 基础颜色
	colorTitle   = color.New(color.FgHiCyan, color.Bold)
	colorSuccess = color.New(color.FgHiGreen)
	colorError   = color.New(color.FgHiRed)
	colorWarning = color.New(color.FgHiYellow)
	colorInfo    = color.New(color.FgHiBlue)
	colorBold    = color.New(color.Bold)

	// 增强颜色 - 特殊用途
	colorEmoji     = color.New(color.FgHiYellow)
	colorCommand   = color.New(color.FgHiBlue, color.Bold)
	colorPath      = color.New(color.FgHiCyan)
	colorSize      = color.New(color.FgHiGreen)
	colorPlatform  = color.New(color.FgHiMagenta, color.Bold)
	colorProgress  = color.New(color.FgHiCyan, color.Bold)
	colorSubtle    = color.New(color.FgHiBlack)
	colorHighlight = color.New(color.FgHiWhite, color.Bold)
)

// PlatformGroups 预设平台组合 - 基于 Go 1.23+ 官方支持的平台
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
		"linux/amd64", "linux/arm64", "linux/ppc64", "linux/ppc64le", "linux/s390x",
		"freebsd/amd64", "freebsd/386", "freebsd/arm", "freebsd/arm64", "freebsd/riscv64",
		"netbsd/amd64", "netbsd/386", "netbsd/arm", "netbsd/arm64",
		"openbsd/amd64", "openbsd/386", "openbsd/arm", "openbsd/arm64", "openbsd/ppc64", "openbsd/riscv64",
		"dragonfly/amd64",
		"solaris/amd64",
		"illumos/amd64",
		"aix/ppc64",
	},
	"mobile": {
		"android/arm64", "android/arm", "android/386", "android/amd64",
		"ios/amd64", "ios/arm64",
	},
	"web": {
		"js/wasm",
		"wasip1/wasm",
	},
	"embedded": {
		"linux/arm", "linux/arm64", "linux/riscv64",
		"linux/mips", "linux/mips64", "linux/mips64le", "linux/mipsle",
		"linux/loong64",
		"plan9/386", "plan9/amd64", "plan9/arm",
	},
	// 新增的专用组合
	"unix": {
		"linux/amd64", "linux/arm64",
		"freebsd/amd64", "freebsd/arm64",
		"netbsd/amd64", "openbsd/amd64",
		"darwin/amd64", "darwin/arm64",
		"solaris/amd64", "illumos/amd64",
		"dragonfly/amd64", "aix/ppc64",
	},
	"bsd": {
		"freebsd/amd64", "freebsd/386", "freebsd/arm", "freebsd/arm64", "freebsd/riscv64",
		"netbsd/amd64", "netbsd/386", "netbsd/arm", "netbsd/arm64",
		"openbsd/amd64", "openbsd/386", "openbsd/arm", "openbsd/arm64", "openbsd/ppc64", "openbsd/riscv64",
		"dragonfly/amd64",
	},
	"linux": {
		"linux/amd64", "linux/386", "linux/arm", "linux/arm64",
		"linux/mips", "linux/mips64", "linux/mips64le", "linux/mipsle",
		"linux/ppc64", "linux/ppc64le", "linux/riscv64", "linux/s390x", "linux/loong64",
	},
	"windows": {
		"windows/amd64", "windows/386", "windows/arm64",
	},
	"darwin": {
		"darwin/amd64", "darwin/arm64",
	},
	"android": {
		"android/arm64", "android/arm", "android/386", "android/amd64",
	},
	"ios": {
		"ios/amd64", "ios/arm64",
	},
	// "all" 组合将通过 getAllSupportedPlatforms() 动态获取
}

// 特殊错误类型
var ErrSkipped = errors.New("跳过编译")

// 全局变量访问器函数
func GetConfig() *Config {
	return &config
}

func GetLogger() *slog.Logger {
	return logger
}

func SetLogger(l *slog.Logger) {
	logger = l
}

// 颜色访问器函数
func GetColors() (title, success, error, warning, info, bold *color.Color) {
	return colorTitle, colorSuccess, colorError, colorWarning, colorInfo, colorBold
}

// GetEnhancedColors 返回增强的颜色变量
func GetEnhancedColors() (emoji, command, path, size, platform, progress, subtle, highlight *color.Color) {
	return colorEmoji, colorCommand, colorPath, colorSize, colorPlatform, colorProgress, colorSubtle, colorHighlight
}

// GetProgressColors 返回进度条相关的颜色变量
func GetProgressColors() (info, success, error, warning *color.Color) {
	var (
		colorInfoProgress    = color.New(color.FgBlue)
		colorSuccessProgress = color.New(color.FgGreen, color.Bold)
		colorErrorProgress   = color.New(color.FgRed, color.Bold)
		colorWarningProgress = color.New(color.FgYellow, color.Bold)
	)
	return colorInfoProgress, colorSuccessProgress, colorErrorProgress, colorWarningProgress
}

// GetAllColors 返回所有颜色变量的便捷访问
func GetAllColors() map[string]*color.Color {
	return map[string]*color.Color{
		"title":     colorTitle,
		"success":   colorSuccess,
		"error":     colorError,
		"warning":   colorWarning,
		"info":      colorInfo,
		"bold":      colorBold,
		"emoji":     colorEmoji,
		"command":   colorCommand,
		"path":      colorPath,
		"size":      colorSize,
		"platform":  colorPlatform,
		"progress":  colorProgress,
		"subtle":    colorSubtle,
		"highlight": colorHighlight,
	}
}

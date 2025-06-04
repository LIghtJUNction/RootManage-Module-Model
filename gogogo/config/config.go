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

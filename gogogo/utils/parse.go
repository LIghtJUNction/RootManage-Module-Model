package utils

import (
	"log/slog"
	"strings"
)

// Config represents the configuration needed for parsing platforms
type Config struct {
	All      bool
	Verbose  int
	NoPrompt bool
}

// PlatformGroups contains predefined platform combinations
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
}

// ParsePlatforms 解析平台字符串
func ParsePlatforms(platformStr string, config Config, logger *slog.Logger) []BuildTarget {
	var targets []BuildTarget
	platforms := strings.Split(platformStr, ",")

	for _, platform := range platforms {
		platform = strings.TrimSpace(platform)

		// 特殊处理 "all" 平台组合
		if platform == "all" {
			allPlatforms, err := GetAllSupportedPlatforms()
			if err != nil {
				if config.Verbose >= 1 {
					ColorError.Printf("⚠️  获取所有平台失败，使用静态列表: %v\n", err)
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

			if config.All { // 获取该OS支持的所有架构
				archs, err = GetArchsForOS(platform)
				if err != nil {
					if config.Verbose >= 1 {
						ColorError.Printf("⚠️  获取 %s 支持的架构失败: %v\n", platform, err)
					}
					continue
				}
				if len(archs) == 0 {
					if config.Verbose >= 1 {
						ColorWarning.Printf("⚠️  操作系统 %s 不支持或未找到\n", platform)
					}
					continue
				}
			} else {
				// 仅使用本机架构
				nativeArch := GetNativeArch()
				// 验证该OS是否支持本机架构
				supportedArchs, err := GetArchsForOS(platform)
				if err != nil {
					if config.Verbose >= 1 {
						ColorError.Printf("⚠️  获取 %s 支持的架构失败: %v\n", platform, err)
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
						ColorWarning.Printf("⚠️  操作系统 %s 不支持本机架构 %s，支持的架构: %s\n",
							platform, nativeArch, strings.Join(supportedArchs, ", "))
						ColorInfo.Printf("💡 可以使用 --all 标志编译该OS的所有架构\n")
					}
					continue
				}
			} // 添加目标平台
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

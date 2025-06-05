package utils

import (
	"log/slog"
	"strings"

	"github.com/lightjunction/rootmanager-module-model/gogogo/config"
)

// ParsePlatforms 解析平台字符串
func ParsePlatforms(platformStr string, cfg *config.Config, logger *slog.Logger) []config.BuildTarget {
	var targets []config.BuildTarget
	platforms := strings.Split(platformStr, ",")

	for _, platform := range platforms {
		platform = strings.TrimSpace(platform)

		// 特殊处理 "all" 平台组合
		if platform == "all" {
			allPlatforms, err := GetAllSupportedPlatforms()
			if err != nil {
				if cfg.Verbose >= 1 {
					ColorError.Printf("⚠️  获取所有平台失败，使用静态列表: %v\n", err)
				}
				// 使用静态的默认平台列表
				for _, p := range config.PlatformGroups["default"] {
					parts := strings.Split(p, "/")
					if len(parts) == 2 {
						targets = append(targets, config.BuildTarget{
							GOOS:   parts[0],
							GOARCH: parts[1],
							Name:   p,
						})
					}
				}
			} else {
				for _, p := range allPlatforms {
					parts := strings.Split(p, "/")
					if len(parts) == 2 {
						targets = append(targets, config.BuildTarget{
							GOOS:   parts[0],
							GOARCH: parts[1],
							Name:   p,
						})
					}
				}
			}
		} else if group, exists := config.PlatformGroups[platform]; exists {
			// 检查是否是其他预设组合
			for _, p := range group {
				parts := strings.Split(p, "/")
				if len(parts) == 2 {
					targets = append(targets, config.BuildTarget{
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
				targets = append(targets, config.BuildTarget{
					GOOS:   parts[0],
					GOARCH: parts[1],
					Name:   platform,
				})
			}
		} else {
			// 单个操作系统名称，需要根据 -all 标志决定架构
			var archs []string
			var err error

			if cfg.All { // 获取该OS支持的所有架构
				archs, err = GetArchsForOS(platform)
				if err != nil {
					if cfg.Verbose >= 1 {
						ColorError.Printf("⚠️  获取 %s 支持的架构失败: %v\n", platform, err)
					}
					continue
				}
				if len(archs) == 0 {
					if cfg.Verbose >= 1 {
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
					if cfg.Verbose >= 1 {
						ColorError.Printf("⚠️  获取 %s 支持的架构失败: %v\n", platform, err)
					}
					continue
				}

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
					if cfg.Verbose >= 1 {
						ColorWarning.Printf("⚠️  %s 不支持本机架构 %s，将跳过\n", platform, nativeArch)
					}
					continue
				}
			}

			// 为该OS的每个架构创建目标
			for _, arch := range archs {
				targets = append(targets, config.BuildTarget{
					GOOS:   platform,
					GOARCH: arch,
					Name:   platform + "/" + arch,
				})
			}
		}
	}

	return targets
}

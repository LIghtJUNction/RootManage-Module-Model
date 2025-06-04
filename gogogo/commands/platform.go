package commands

import (
	"fmt"
	"os/exec"
	"strings"

	"github.com/fatih/color"
)

// getAllSupportedPlatforms 获取Go支持的所有平台
func GetAllSupportedPlatforms() ([]string, error) {
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

// ListPlatforms 列出所有支持的平台
func ListPlatforms() {
	colorTitle := color.New(color.FgHiCyan, color.Bold)
	colorError := color.New(color.FgHiRed)
	colorBold := color.New(color.Bold)

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

// ListGroups 列出平台组合
func ListGroups(platformGroups map[string][]string) {
	colorTitle := color.New(color.FgHiCyan, color.Bold)
	colorError := color.New(color.FgHiRed)
	colorBold := color.New(color.Bold)
	colorInfo := color.New(color.FgHiBlue)

	colorTitle.Println("📦 平台组合:")

	// 显示静态预设组合
	for group, platforms := range platformGroups {
		colorBold.Printf("  %s:\n", group)
		for _, platform := range platforms {
			fmt.Printf("    • %s\n", platform)
		}
		fmt.Println()
	}

	// 动态显示 "all" 组合
	colorBold.Printf("  all (动态获取):\n")
	allPlatforms, err := GetAllSupportedPlatforms()
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

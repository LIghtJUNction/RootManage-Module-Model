package utils

import (
	"fmt"
	"os/exec"
	"runtime"
	"strings"
)

// GetAllSupportedPlatforms 获取所有支持的平台
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

// GetArchsForOS 获取指定操作系统支持的架构列表
func GetArchsForOS(targetOS string) ([]string, error) {
	allPlatforms, err := GetAllSupportedPlatforms()
	if err != nil {
		return nil, err
	}

	var archs []string
	for _, platform := range allPlatforms {
		parts := strings.Split(platform, "/")
		if len(parts) == 2 && parts[0] == targetOS {
			archs = append(archs, parts[1])
		}
	}

	return archs, nil
}

// GetNativeArch 获取本机架构
func GetNativeArch() string {
	return runtime.GOARCH
}

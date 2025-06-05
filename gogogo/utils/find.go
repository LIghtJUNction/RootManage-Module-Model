package utils

import (
	"fmt"
	"log/slog"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"sort"
	"strconv"
	"strings"

	"github.com/lightjunction/rootmanager-module-model/gogogo/config"
)

// getUserHomeDir 获取用户主目录
func getUserHomeDir() (string, error) {
	homeDir, err := os.UserHomeDir()
	if err != nil {
		return "", fmt.Errorf("无法获取用户主目录: %v", err)
	}
	return homeDir, nil
}

// FindSystemNDK 查找系统中已安装的NDK
func FindSystemNDK(logger *slog.Logger) []string {
	var ndkPaths []string

	homeDir, err := getUserHomeDir()
	if err != nil {
		logger.Error("获取用户主目录失败", "error", err)
		return ndkPaths
	}

	// 常见的NDK安装路径
	possiblePaths := []string{}

	switch runtime.GOOS {
	case "windows":
		possiblePaths = []string{
			filepath.Join(homeDir, "AppData", "Local", "Android", "Sdk", "ndk"),
			"C:\\Android\\Sdk\\ndk",
			"C:\\Users\\%USERNAME%\\AppData\\Local\\Android\\Sdk\\ndk",
			filepath.Join(os.Getenv("ANDROID_HOME"), "ndk"),
			filepath.Join(os.Getenv("ANDROID_SDK_ROOT"), "ndk"),
		}
	case "darwin":
		possiblePaths = []string{
			filepath.Join(homeDir, "Library", "Android", "sdk", "ndk"),
			filepath.Join(homeDir, "Android", "Sdk", "ndk"),
			"/opt/android-sdk/ndk",
			filepath.Join(os.Getenv("ANDROID_HOME"), "ndk"),
			filepath.Join(os.Getenv("ANDROID_SDK_ROOT"), "ndk"),
		}
	case "linux":
		possiblePaths = []string{
			filepath.Join(homeDir, "Android", "Sdk", "ndk"),
			"/opt/android-sdk/ndk",
			"/usr/local/android-sdk/ndk",
			filepath.Join(os.Getenv("ANDROID_HOME"), "ndk"),
			filepath.Join(os.Getenv("ANDROID_SDK_ROOT"), "ndk"),
		}
	}

	// 查找NDK版本目录
	for _, basePath := range possiblePaths {
		if basePath == "" {
			continue
		}

		// 展开环境变量
		basePath = os.ExpandEnv(basePath)

		if _, err := os.Stat(basePath); os.IsNotExist(err) {
			continue
		}

		// 查找版本目录
		versions, err := os.ReadDir(basePath)
		if err != nil {
			continue
		}

		for _, version := range versions {
			if version.IsDir() {
				ndkVersionPath := filepath.Join(basePath, version.Name())
				if IsValidNDKDir(ndkVersionPath) {
					ndkPaths = append(ndkPaths, ndkVersionPath)
					logger.Info("找到NDK", "path", ndkVersionPath)
				}
			}
		}
	}

	// 去重并排序
	ndkPaths = removeDuplicateStrings(ndkPaths)
	sort.Strings(ndkPaths)

	return ndkPaths
}

// FindSystemClang 查找系统中已安装的Clang
func FindSystemClang(logger *slog.Logger) []config.ClangInstallation {
	var installations []config.ClangInstallation

	switch runtime.GOOS {
	case "windows":
		installations = append(installations, findWindowsClang(logger)...)
	case "darwin":
		installations = append(installations, findDarwinClang(logger)...)
	case "linux":
		installations = append(installations, findLinuxClang(logger)...)
	}

	// 过滤无效的安装
	var validInstallations []config.ClangInstallation
	for _, installation := range installations {
		if IsValidClangInstallation(installation) {
			validInstallations = append(validInstallations, installation)
			logger.Info("找到Clang", "path", installation.Path, "type", installation.Type, "version", installation.Version)
		}
	}

	return validInstallations
}

// findWindowsClang 查找Windows系统中的Clang
func findWindowsClang(logger *slog.Logger) []config.ClangInstallation {
	var installations []config.ClangInstallation

	// Visual Studio Clang
	vsInstallations := findVisualStudioClang(logger)
	installations = append(installations, vsInstallations...)

	// LLVM独立安装
	llvmInstallations := findLLVMClang(logger)
	installations = append(installations, llvmInstallations...)

	// MinGW/MSYS2
	mingwInstallations := findMinGWClang(logger)
	installations = append(installations, mingwInstallations...)

	// Git for Windows
	gitInstallations := findGitClang(logger)
	installations = append(installations, gitInstallations...)

	return installations
}

// findDarwinClang 查找macOS系统中的Clang
func findDarwinClang(logger *slog.Logger) []config.ClangInstallation {
	var installations []config.ClangInstallation
	// Xcode命令行工具
	if clangPath, err := exec.LookPath("clang"); err == nil {
		version := getClangVersion(clangPath)
		installations = append(installations, config.ClangInstallation{
			Path:     clangPath,
			Type:     "Xcode",
			Version:  version,
			Priority: 3,
		})
	}

	// Homebrew LLVM
	homebrewPaths := []string{
		"/opt/homebrew/bin/clang", // Apple Silicon
		"/usr/local/bin/clang",    // Intel
	}
	for _, path := range homebrewPaths {
		if _, err := os.Stat(path); err == nil {
			version := getClangVersion(path)
			installations = append(installations, config.ClangInstallation{
				Path:    path,
				Type:    "LLVM",
				Version: version,
			})
		}
	}

	return installations
}

// findLinuxClang 查找Linux系统中的Clang
func findLinuxClang(logger *slog.Logger) []config.ClangInstallation {
	var installations []config.ClangInstallation

	// 系统包管理器安装的clang
	if clangPath, err := exec.LookPath("clang"); err == nil {
		version := getClangVersion(clangPath)
		installations = append(installations, config.ClangInstallation{
			Path:    clangPath,
			Type:    "System",
			Version: version,
		})
	}

	// 常见的clang版本路径
	commonPaths := []string{
		"/usr/bin/clang",
		"/usr/local/bin/clang",
	}

	// 查找带版本号的clang
	for i := 10; i <= 18; i++ {
		versionedPaths := []string{
			fmt.Sprintf("/usr/bin/clang-%d", i),
			fmt.Sprintf("/usr/local/bin/clang-%d", i),
		}
		commonPaths = append(commonPaths, versionedPaths...)
	}

	for _, path := range commonPaths {
		if _, err := os.Stat(path); err == nil {
			version := getClangVersion(path)
			installations = append(installations, config.ClangInstallation{
				Path:    path,
				Type:    "System",
				Version: version,
			})
		}
	}

	return installations
}

// findVisualStudioClang 查找Visual Studio中的Clang
func findVisualStudioClang(logger *slog.Logger) []config.ClangInstallation {
	var installations []config.ClangInstallation

	// Visual Studio 2019/2022的常见路径
	vsPaths := []string{
		"C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Tools\\Llvm\\bin\\clang.exe",
		"C:\\Program Files\\Microsoft Visual Studio\\2022\\Professional\\VC\\Tools\\Llvm\\bin\\clang.exe",
		"C:\\Program Files\\Microsoft Visual Studio\\2022\\Enterprise\\VC\\Tools\\Llvm\\bin\\clang.exe",
		"C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Community\\VC\\Tools\\Llvm\\bin\\clang.exe",
		"C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Professional\\VC\\Tools\\Llvm\\bin\\clang.exe",
		"C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Enterprise\\VC\\Tools\\Llvm\\bin\\clang.exe",
	}

	for _, path := range vsPaths {
		if _, err := os.Stat(path); err == nil {
			version := getClangVersion(path)
			installations = append(installations, config.ClangInstallation{
				Path:    path,
				Type:    "Visual Studio",
				Version: version,
			})
		}
	}

	return installations
}

// findLLVMClang 查找独立LLVM安装的Clang
func findLLVMClang(logger *slog.Logger) []config.ClangInstallation {
	var installations []config.ClangInstallation

	// 常见的LLVM安装路径
	llvmPaths := []string{
		"C:\\Program Files\\LLVM\\bin\\clang.exe",
		"C:\\Program Files (x86)\\LLVM\\bin\\clang.exe",
		"C:\\LLVM\\bin\\clang.exe",
	}

	for _, path := range llvmPaths {
		if _, err := os.Stat(path); err == nil {
			version := getClangVersion(path)
			installations = append(installations, config.ClangInstallation{
				Path:    path,
				Type:    "LLVM",
				Version: version,
			})
		}
	}

	return installations
}

// findMinGWClang 查找MinGW/MSYS2中的Clang
func findMinGWClang(logger *slog.Logger) []config.ClangInstallation {
	var installations []config.ClangInstallation

	// MinGW/MSYS2路径
	mingwPaths := []string{
		"C:\\msys64\\mingw64\\bin\\clang.exe",
		"C:\\msys64\\mingw32\\bin\\clang.exe",
		"C:\\msys64\\clang64\\bin\\clang.exe",
		"C:\\mingw64\\bin\\clang.exe",
		"C:\\mingw32\\bin\\clang.exe",
	}

	for _, path := range mingwPaths {
		if _, err := os.Stat(path); err == nil {
			version := getClangVersion(path)
			installations = append(installations, config.ClangInstallation{
				Path:    path,
				Type:    "MinGW",
				Version: version,
			})
		}
	}

	return installations
}

// findGitClang 查找Git for Windows中的Clang
func findGitClang(logger *slog.Logger) []config.ClangInstallation {
	var installations []config.ClangInstallation

	// Git for Windows路径
	gitPaths := []string{
		"C:\\Program Files\\Git\\usr\\bin\\clang.exe",
		"C:\\Program Files (x86)\\Git\\usr\\bin\\clang.exe",
	}

	for _, path := range gitPaths {
		if _, err := os.Stat(path); err == nil {
			version := getClangVersion(path)
			installations = append(installations, config.ClangInstallation{
				Path:    path,
				Type:    "Git for Windows",
				Version: version,
			})
		}
	}

	return installations
}

// getClangVersion 获取Clang版本
func getClangVersion(clangPath string) string {
	cmd := exec.Command(clangPath, "--version")
	output, err := cmd.Output()
	if err != nil {
		return "Unknown"
	}

	lines := strings.Split(string(output), "\n")
	if len(lines) > 0 {
		// 解析版本号
		firstLine := lines[0]
		if strings.Contains(firstLine, "clang version") {
			parts := strings.Fields(firstLine)
			for i, part := range parts {
				if part == "version" && i+1 < len(parts) {
					return parts[i+1]
				}
			}
		}
	}

	return "Unknown"
}

// GetBestClangForTarget 为指定目标获取最佳的Clang
func GetBestClangForTarget(target string, installations []config.ClangInstallation, logger *slog.Logger) config.ClangInstallation {
	if len(installations) == 0 {
		return config.ClangInstallation{}
	}

	// 根据目标平台选择最佳的clang
	var bestInstallation config.ClangInstallation
	bestScore := -1

	for _, installation := range installations {
		score := scoreClangForTarget(target, installation)
		if score > bestScore {
			bestScore = score
			bestInstallation = installation
		}
	}

	logger.Info("为目标选择Clang", "target", target, "clang", bestInstallation.Path, "type", bestInstallation.Type)
	return bestInstallation
}

// scoreClangForTarget 为特定目标评分Clang安装
func scoreClangForTarget(target string, installation config.ClangInstallation) int {
	score := 0

	// 基础分数
	score += 10

	// 根据类型评分
	switch installation.Type {
	case "Xcode":
		score += 50 // macOS上优先使用Xcode
	case "LLVM":
		score += 40 // 独立LLVM安装通常比较新
	case "Visual Studio":
		score += 30 // Windows上Visual Studio集成度好
	case "System":
		score += 20 // 系统安装的稳定性好
	case "MinGW":
		score += 10 // MinGW兼容性一般
	case "Git for Windows":
		score += 5 // Git自带的clang功能有限
	}

	// 根据版本评分 (简单的版本号解析)
	if installation.Version != "Unknown" {
		if versionFloat := parseVersionToFloat(installation.Version); versionFloat > 0 {
			score += int(versionFloat * 2) // 版本越高分数越高
		}
	}

	// 根据目标平台特殊评分
	if strings.Contains(target, "darwin") || strings.Contains(target, "ios") {
		if installation.Type == "Xcode" {
			score += 100 // iOS构建必须使用Xcode
		}
	}

	return score
}

// parseVersionToFloat 将版本字符串转换为浮点数用于比较
func parseVersionToFloat(version string) float64 {
	// 提取主版本号和次版本号
	parts := strings.Split(version, ".")
	if len(parts) >= 2 {
		major, err1 := strconv.Atoi(parts[0])
		minor, err2 := strconv.Atoi(parts[1])
		if err1 == nil && err2 == nil {
			return float64(major) + float64(minor)/10.0
		}
	} else if len(parts) == 1 {
		major, err := strconv.Atoi(parts[0])
		if err == nil {
			return float64(major)
		}
	}
	return 0
}

// SetupClangEnvironment 为指定的clang设置环境变量
func SetupClangEnvironment(installation config.ClangInstallation, logger *slog.Logger) error {
	if installation.Path == "" {
		return fmt.Errorf("无效的clang安装")
	}

	// 设置CC和CXX环境变量
	clangDir := filepath.Dir(installation.Path)
	clangxxPath := filepath.Join(clangDir, "clang++")

	// 在Windows上添加.exe扩展名
	if runtime.GOOS == "windows" {
		if !strings.HasSuffix(clangxxPath, ".exe") {
			clangxxPath += ".exe"
		}
	}

	os.Setenv("CC", installation.Path)
	if _, err := os.Stat(clangxxPath); err == nil {
		os.Setenv("CXX", clangxxPath)
	}

	// 根据安装类型设置额外的环境变量
	switch installation.Type {
	case "Visual Studio":
		// Visual Studio需要设置更多环境变量
		setupVisualStudioEnvironment(installation.Path, logger)
	case "MinGW":
		// MinGW需要设置PATH
		setupMinGWEnvironment(installation.Path, logger)
	}

	logger.Info("已设置Clang环境",
		"CC", installation.Path,
		"CXX", clangxxPath,
		"type", installation.Type)

	return nil
}

// setupVisualStudioEnvironment 设置Visual Studio环境
func setupVisualStudioEnvironment(clangPath string, logger *slog.Logger) {
	// Visual Studio的clang需要设置VCINSTALLDIR等环境变量
	vcDir := filepath.Dir(filepath.Dir(filepath.Dir(clangPath))) // 从bin目录向上3级
	os.Setenv("VCINSTALLDIR", vcDir)

	logger.Info("已设置Visual Studio环境", "VCINSTALLDIR", vcDir)
}

// setupMinGWEnvironment 设置MinGW环境
func setupMinGWEnvironment(clangPath string, logger *slog.Logger) {
	// 将MinGW的bin目录添加到PATH前面
	binDir := filepath.Dir(clangPath)
	currentPath := os.Getenv("PATH")
	newPath := binDir + string(os.PathListSeparator) + currentPath
	os.Setenv("PATH", newPath)

	logger.Info("已设置MinGW环境", "binDir", binDir)
}

// removeDuplicateStrings 去除字符串切片中的重复项
func removeDuplicateStrings(strs []string) []string {
	seen := make(map[string]bool)
	var result []string

	for _, str := range strs {
		if !seen[str] {
			seen[str] = true
			result = append(result, str)
		}
	}

	return result
}

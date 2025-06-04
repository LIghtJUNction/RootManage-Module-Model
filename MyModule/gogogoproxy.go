package main

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"github.com/fatih/color"
)

// ProxyNode 代表一个代理节点
type ProxyNode struct {
	URL   string  `json:"url"`
	Speed float64 `json:"speed"`
}

// APIResponse 代表API响应结构
type APIResponse struct {
	Data []ProxyNode `json:"data"`
}

// 颜色配置
var (
	colorInfo    = color.New(color.FgHiBlue)
	colorSuccess = color.New(color.FgHiGreen)
	colorError   = color.New(color.FgHiRed)
	colorWarning = color.New(color.FgHiYellow)
	colorEmoji   = color.New(color.FgHiYellow)
)

const (
	apiURL     = "https://api.akams.cn/github"
	envFile    = "gogogo.env"
	timeoutSec = 30
)

func main() {
	colorEmoji.Print("🚀 ")
	colorInfo.Println("开始获取GitHub代理节点...")

	// 获取当前程序目录
	modDir, err := getModuleDir()
	if err != nil {
		colorError.Printf("❌ 错误: 获取程序目录失败: %v\n", err)
		os.Exit(1)
	}

	// 获取并排序GitHub代理节点
	proxies, err := fetchAndSortProxies()
	if err != nil {
		colorError.Printf("❌ 错误: %v\n", err)
		os.Exit(1)
	}

	colorEmoji.Print("📊 ")
	colorInfo.Printf("获取到 %d 个GitHub代理节点\n", len(proxies))

	colorEmoji.Print("⚡ ")
	colorSuccess.Printf("最快的GitHub代理: %s (%.2f)\n", proxies[0].URL, proxies[0].Speed)

	// 写入Go环境变量文件（包含固定的Go模块代理配置）
	envPath := filepath.Join(modDir, envFile)
	if err := writeEnvFile(envPath, proxies); err != nil {
		colorError.Printf("❌ 错误: 写入环境变量文件失败: %v\n", err)
		os.Exit(1)
	}

	colorEmoji.Print("✅ ")
	colorSuccess.Printf("Go环境配置已写入: %s\n", envPath)

	// 写入GITHUB.PROXYES文件（GitHub代理列表）
	proxiesPath := filepath.Join(modDir, "GITHUB.PROXYES")
	if err := writeProxiesFile(proxiesPath, proxies); err != nil {
		colorError.Printf("❌ 错误: 写入代理列表文件失败: %v\n", err)
		os.Exit(1)
	}

	colorEmoji.Print("✅ ")
	colorSuccess.Printf("GitHub代理列表已写入: %s\n", proxiesPath)

	// 显示使用说明
	colorEmoji.Print("📋 ")
	colorInfo.Println("使用说明:")
	colorWarning.Println("  1. Go模块代理已设置为国内高速节点")
	colorWarning.Printf("  2. GitHub代理推荐使用: %s\n", proxies[0].URL)
	colorWarning.Println("  3. 配置Git使用GitHub代理:")
	fmt.Printf("     git config --global url.\"%s/https://github.com/\".insteadOf \"https://github.com/\"\n", proxies[0].URL)
}

// getModuleDir 获取程序所在目录
func getModuleDir() (string, error) {
	execPath, err := os.Executable()
	if err != nil {
		return "", fmt.Errorf("获取可执行文件路径失败: %v", err)
	}
	return filepath.Dir(execPath), nil
}

// fetchAndSortProxies 从API获取代理节点并按速度排序
func fetchAndSortProxies() ([]ProxyNode, error) {
	colorEmoji.Print("🌐 ")
	colorInfo.Printf("正在请求API: %s\n", apiURL)

	// 创建HTTP客户端，设置超时
	client := &http.Client{
		Timeout: time.Second * timeoutSec,
	}

	// 发送GET请求
	resp, err := client.Get(apiURL)
	if err != nil {
		return nil, fmt.Errorf("无法访问API: %v", err)
	}
	defer resp.Body.Close()

	// 检查HTTP状态码
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("API请求失败，状态码: %d", resp.StatusCode)
	}

	// 读取响应体
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("读取响应失败: %v", err)
	}

	// 解析JSON
	var apiResp APIResponse
	if err := json.Unmarshal(body, &apiResp); err != nil {
		return nil, fmt.Errorf("解析JSON失败: %v", err)
	}

	// 按速度降序排序
	sort.Slice(apiResp.Data, func(i, j int) bool {
		return apiResp.Data[i].Speed > apiResp.Data[j].Speed
	})

	if len(apiResp.Data) == 0 {
		return nil, fmt.Errorf("未获取到有效的代理节点")
	}

	colorEmoji.Print("📈 ")
	colorSuccess.Println("代理节点已按速度排序")

	return apiResp.Data, nil
}

// generateProxyString 生成代理字符串
func generateProxyString(proxies []ProxyNode) string {
	var urls []string
	for _, proxy := range proxies {
		urls = append(urls, proxy.URL)
	}

	// 添加direct作为最后一个元素
	urls = append(urls, "direct")

	return strings.Join(urls, ",")
}

// writeEnvFile 写入环境变量文件
func writeEnvFile(path string, proxies []ProxyNode) error {
	// 获取推荐的GitHub代理（最快的那个）
	bestGitHubProxy := proxies[0].URL
	content := fmt.Sprintf(`# Go模块代理配置 - 使用国内高速代理服务器
# 注意：这里是Go模块下载代理，不是GitHub访问代理
GOPROXY=https://goproxy.cn,https://goproxy.io,https://proxy.golang.org,direct
GOSUMDB=sum.golang.google.cn
GOTOOLCHAIN=auto
GO111MODULE=on
GOTELEMETRY=off

# ========================================
# GitHub访问加速配置说明
# ========================================
# 以下是GitHub访问代理，用于加速GitHub仓库访问
# 推荐最快代理: %s

# Git依赖说明：
# - 当GOPROXY可用时，Go不需要Git即可下载大部分模块
# - 当GOPROXY失败时，Go会fallback到直接从Git仓库下载
# - 在Android等特殊环境中，建议：
#   1. 确保GOPROXY始终可用（避免fallback到Git）
#   2. 如需Git，使用静态编译版本或Termux
#   3. 设置GOPRIVATE跳过私有仓库的代理

# 配置Git使用GitHub代理的方法：
# git config --global url."%s/https://github.com/".insteadOf "https://github.com/"

# 临时使用代理克隆仓库：
# git clone %s/https://github.com/用户名/仓库名.git

# 注意：GitHub代理和Go模块代理是不同的概念！
# - GOPROXY: 用于下载Go模块包（go get命令）
# - GitHub代理: 用于访问GitHub仓库（git clone命令）
# - 建议优先依赖GOPROXY，减少对Git的依赖
`, bestGitHubProxy, bestGitHubProxy, bestGitHubProxy)

	return os.WriteFile(path, []byte(content), 0644)
}

// writeProxiesFile 写入代理列表文件 (URL SPEED格式)
func writeProxiesFile(path string, proxies []ProxyNode) error {
	var content strings.Builder

	for _, proxy := range proxies {
		content.WriteString(fmt.Sprintf("%s %.2f\n", proxy.URL, proxy.Speed))
	}

	return os.WriteFile(path, []byte(content.String()), 0644)
}

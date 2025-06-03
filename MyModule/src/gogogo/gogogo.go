package main

import (
	"bufio"
	"compress/gzip"
	"encoding/json"
	"flag"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"
	"sync"
	"time"
)

// Platform 定义目标平台和架构
type Platform struct {
	OS   string
	Arch string
}

// BuildTarget 定义构建目标
type BuildTarget struct {
	Platform   Platform
	OutputName string
	OutputPath string
}

// BuildResult 构建结果
type BuildResult struct {
	Target   BuildTarget
	Success  bool
	Error    error
	Duration time.Duration
	Attempt  int
}

// VerbosityLevel 详细程度级别
type VerbosityLevel int

const (
	VerboseQuiet    VerbosityLevel = iota // 0: 安静模式
	VerboseNormal                         // 1: 正常模式
	VerboseDetailed                       // 2: 详细模式
	VerboseDebug                          // 3: 调试模式
)

// PlatformGroup 平台组定义
type PlatformGroup struct {
	Name        string     `json:"name"`
	Description string     `json:"description"`
	Platforms   []Platform `json:"platforms"`
	Category    string     `json:"category"`
	Tags        []string   `json:"tags"`
}

// GroupOperation 组操作类型
type GroupOperation struct {
	Type      string   // "include", "exclude", "only", "except"
	Groups    []string // 组名列表
	Platforms []string // 平台列表
}

// PlatformSet 平台集合，用于去重和集合操作
type PlatformSet map[string]Platform

// Builder 跨平台编译器
type Builder struct {
	SourceFile     string
	OutputDir      string
	Platforms      []Platform
	BinaryName     string
	Verbose        bool
	Parallel       bool
	Compress       bool
	BuildFlags     []string
	LdFlags        string
	Tags           string
	SkipTests      bool
	CleanOutput    bool
	ShowProgress   bool
	VerbosityLevel VerbosityLevel
	RetryFailures  bool
	MaxRetries     int
	Interactive    bool   // 交互式模式
	OutputFormat   string // 输出文件命名格式
	MaxJobs        int    // 最大并行构建进程数
	SkipCGO        bool   // 跳过需要CGO的平台

	// 新增组合功能字段
	IncludeGroups   []string         // 包含的组
	ExcludeGroups   []string         // 排除的组
	CustomGroups    []PlatformGroup  // 自定义组
	GroupOperations []GroupOperation // 组操作序列
	SaveGroupConfig string           // 保存组配置到文件
	LoadGroupConfig string           // 从文件加载组配置
}

// 预定义的常用平台
var commonPlatforms = []Platform{
	// Windows
	{"windows", "amd64"},
	{"windows", "386"},
	{"windows", "arm64"},
	{"windows", "arm"},

	// Linux
	{"linux", "amd64"},
	{"linux", "386"},
	{"linux", "arm64"},
	{"linux", "arm"},
	{"linux", "ppc64"},
	{"linux", "ppc64le"},
	{"linux", "mips"},
	{"linux", "mipsle"},
	{"linux", "mips64"},
	{"linux", "mips64le"},
	{"linux", "s390x"},
	{"linux", "riscv64"},

	// macOS (Darwin)
	{"darwin", "amd64"},
	{"darwin", "arm64"},

	// FreeBSD
	{"freebsd", "amd64"},
	{"freebsd", "386"},
	{"freebsd", "arm64"},
	{"freebsd", "arm"},

	// OpenBSD
	{"openbsd", "amd64"},
	{"openbsd", "386"},
	{"openbsd", "arm64"},
	{"openbsd", "arm"},

	// NetBSD
	{"netbsd", "amd64"},
	{"netbsd", "386"},
	{"netbsd", "arm64"},
	{"netbsd", "arm"},

	// DragonFly BSD
	{"dragonfly", "amd64"},

	// Solaris
	{"solaris", "amd64"},

	// AIX
	{"aix", "ppc64"},

	// Plan 9
	{"plan9", "amd64"},
	{"plan9", "386"},
	{"plan9", "arm"},

	// Android
	{"android", "amd64"},
	{"android", "386"},
	{"android", "arm64"},
	{"android", "arm"},

	// iOS
	{"ios", "amd64"},
	{"ios", "arm64"},

	// JavaScript/WebAssembly
	{"js", "wasm"},
	// WebAssembly System Interface
	{"wasip1", "wasm"},
}

// ANSI颜色代码
const (
	Reset  = "\033[0m"
	Red    = "\033[31m"
	Green  = "\033[32m"
	Yellow = "\033[33m"
	Blue   = "\033[34m"
	Purple = "\033[35m"
	Cyan   = "\033[36m"
	White  = "\033[37m"
	Bold   = "\033[1m"
)

// 预定义的平台组合
var platformGroups = map[string][]Platform{
	// ========== 核心平台组合 ==========
	"minimal":   {{"linux", "amd64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                                                                // 最小支持集
	"desktop":   {{"windows", "amd64"}, {"windows", "arm64"}, {"linux", "amd64"}, {"linux", "arm64"}, {"darwin", "amd64"}, {"darwin", "arm64"}}, // 桌面平台
	"web":       {{"js", "wasm"}, {"wasip1", "wasm"}},                                                                                           // Web平台
	"mobile":    {{"android", "arm64"}, {"android", "arm"}, {"ios", "arm64"}},                                                                   // 移动平台
	"server":    {{"linux", "amd64"}, {"linux", "arm64"}, {"freebsd", "amd64"}, {"freebsd", "arm64"}},                                           // 服务器平台
	"embedded":  {{"linux", "arm"}, {"linux", "arm64"}, {"linux", "mips"}, {"linux", "mips64"}, {"linux", "riscv64"}},                           // 嵌入式平台
	"cloud":     {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}},                                                                 // 云平台
	"container": {{"linux", "amd64"}, {"linux", "arm64"}},                                                                                       // 容器平台

	// ========== 架构分组 ==========
	"amd64-only": {{"windows", "amd64"}, {"linux", "amd64"}, {"darwin", "amd64"}, {"freebsd", "amd64"}},
	"arm64-only": {{"windows", "arm64"}, {"linux", "arm64"}, {"darwin", "arm64"}, {"android", "arm64"}},
	"arm-only":   {{"linux", "arm"}, {"freebsd", "arm"}, {"android", "arm"}},
	"x86-legacy": {{"windows", "386"}, {"linux", "386"}, {"freebsd", "386"}},

	// ========== 操作系统分组 ==========
	"windows-all": {{"windows", "amd64"}, {"windows", "386"}, {"windows", "arm64"}, {"windows", "arm"}},
	"linux-all":   {{"linux", "amd64"}, {"linux", "386"}, {"linux", "arm64"}, {"linux", "arm"}, {"linux", "ppc64"}, {"linux", "ppc64le"}, {"linux", "mips"}, {"linux", "mipsle"}, {"linux", "mips64"}, {"linux", "mips64le"}, {"linux", "s390x"}, {"linux", "riscv64"}},
	"darwin-all":  {{"darwin", "amd64"}, {"darwin", "arm64"}},
	"bsd-all":     {{"freebsd", "amd64"}, {"freebsd", "arm64"}, {"openbsd", "amd64"}, {"netbsd", "amd64"}, {"dragonfly", "amd64"}},

	// ========== 特殊用途分组 ==========
	"cgo-required": {{"android", "arm64"}, {"android", "arm"}, {"ios", "arm64"}},                                        // 需要CGO的平台
	"no-cgo":       {{"windows", "amd64"}, {"linux", "amd64"}, {"darwin", "amd64"}, {"js", "wasm"}, {"wasip1", "wasm"}}, // 纯Go平台
	"cross-safe":   {{"linux", "amd64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                                     // 跨平台安全编译
	"testing":      {{"linux", "amd64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                                     // 测试平台

	// ========== 性能分组 ==========
	"high-perf":    {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}}, // 高性能平台
	"low-resource": {{"linux", "arm"}, {"linux", "mips"}, {"linux", "386"}},                             // 低资源平台

	// ========== 云原生和容器 ==========
	"docker":        {{"linux", "amd64"}, {"linux", "arm64"}},                                            // Docker 镜像
	"k8s":           {{"linux", "amd64"}, {"linux", "arm64"}},                                            // Kubernetes
	"microservices": {{"linux", "amd64"}, {"linux", "arm64"}},                                            // 微服务
	"serverless":    {{"linux", "amd64"}, {"linux", "arm64"}},                                            // 无服务器
	"edge":          {{"linux", "amd64"}, {"linux", "arm64"}, {"linux", "arm"}},                          // 边缘计算
	"mesh":          {{"linux", "amd64"}, {"linux", "arm64"}},                                            // 服务网格
	"istio":         {{"linux", "amd64"}, {"linux", "arm64"}},                                            // Istio 服务网格
	"envoy":         {{"linux", "amd64"}, {"linux", "arm64"}},                                            // Envoy 代理
	"consul":        {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}}, // Consul 服务发现
	"etcd":          {{"linux", "amd64"}, {"linux", "arm64"}},                                            // etcd 键值存储
	"vault":         {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}}, // Vault 密钥管理

	// ========== 数据平台 ==========
	"database":      {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}}, // 数据库
	"nosql":         {{"linux", "amd64"}, {"linux", "arm64"}},                       // NoSQL 数据库
	"timeseries":    {{"linux", "amd64"}, {"linux", "arm64"}},                       // 时间序列数据库
	"analytics":     {{"linux", "amd64"}, {"linux", "arm64"}, {"linux", "ppc64le"}}, // 分析平台	"bigdata":      {{"linux", "amd64"}, {"linux", "arm64"}, {"linux", "ppc64le"}, {"linux", "s390x"}},                    // 大数据
	"stream":        {{"linux", "amd64"}, {"linux", "arm64"}},                       // 流处理
	"batch":         {{"linux", "amd64"}, {"linux", "arm64"}, {"linux", "ppc64le"}}, // 批处理
	"etl":           {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}}, // ETL 数据处理
	"datawarehouse": {{"linux", "amd64"}, {"linux", "arm64"}, {"linux", "ppc64le"}}, // 数据仓库
	"datalake":      {{"linux", "amd64"}, {"linux", "arm64"}},                       // 数据湖
	"search":        {{"linux", "amd64"}, {"linux", "arm64"}},                       // 搜索引擎
	"queue":         {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}}, // 消息队列
	"cache":         {{"linux", "amd64"}, {"linux", "arm64"}},                       // 缓存

	// ========== 监控和可观测性 ==========
	"monitoring":    {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}}, // 监控
	"observability": {{"linux", "amd64"}, {"linux", "arm64"}},                                            // 可观测性
	"metrics":       {{"linux", "amd64"}, {"linux", "arm64"}},                                            // 指标收集
	"logging":       {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}},                      // 日志
	"tracing":       {{"linux", "amd64"}, {"linux", "arm64"}},                                            // 链路追踪
	"alerting":      {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}}, // 告警
	"dashboard":     {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}}, // 仪表板
	"apm":           {{"linux", "amd64"}, {"linux", "arm64"}},                                            // 应用性能监控
	"siem":          {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}},                      // 安全信息事件管理

	// ========== DevOps 和 CI/CD ==========
	"devops":         {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                       // DevOps 工具
	"ci-cd":          {{"linux", "amd64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                                           // CI/CD 流水线
	"gitops":         {{"linux", "amd64"}, {"linux", "arm64"}},                                                                  // GitOps
	"automation":     {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                       // 自动化
	"iac":            {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                       // 基础设施即代码
	"terraform":      {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}, {"freebsd", "amd64"}}, // Terraform
	"ansible":        {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                       // Ansible
	"jenkins":        {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}},                                            // Jenkins
	"github-actions": {{"linux", "amd64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                                           // GitHub Actions
	"gitlab-ci":      {{"linux", "amd64"}, {"linux", "arm64"}},                                                                  // GitLab CI

	// ========== 网络和安全 ==========
	"networking":   {{"linux", "amd64"}, {"linux", "arm64"}, {"freebsd", "amd64"}},                       // 网络工具
	"security":     {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"freebsd", "amd64"}}, // 安全工具
	"firewall":     {{"linux", "amd64"}, {"linux", "arm64"}, {"freebsd", "amd64"}, {"openbsd", "amd64"}}, // 防火墙
	"vpn":          {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"freebsd", "amd64"}}, // VPN
	"proxy":        {{"linux", "amd64"}, {"linux", "arm64"}, {"freebsd", "amd64"}},                       // 代理服务器
	"loadbalancer": {{"linux", "amd64"}, {"linux", "arm64"}, {"freebsd", "amd64"}},                       // 负载均衡
	"apigateway":   {{"linux", "amd64"}, {"linux", "arm64"}},                                             // API网关
	"cdn":          {{"linux", "amd64"}, {"linux", "arm64"}},                                             // CDN
	"dns":          {{"linux", "amd64"}, {"linux", "arm64"}, {"freebsd", "amd64"}, {"openbsd", "amd64"}}, // DNS服务器
	"dhcp":         {{"linux", "amd64"}, {"linux", "arm64"}, {"freebsd", "amd64"}},                       // DHCP服务器

	// ========== 企业和特殊用途 ==========
	"enterprise": {{"linux", "amd64"}, {"linux", "ppc64le"}, {"linux", "s390x"}, {"aix", "ppc64"}, {"solaris", "amd64"}}, // 企业级系统
	"mainframe":  {{"linux", "s390x"}, {"aix", "ppc64"}},                                                                 // 大型机
	"hpc":        {{"linux", "amd64"}, {"linux", "arm64"}, {"linux", "ppc64le"}},                                         // 高性能计算
	"ml":         {{"linux", "amd64"}, {"linux", "arm64"}, {"darwin", "amd64"}, {"darwin", "arm64"}},                     // 机器学习
	"ai":         {{"linux", "amd64"}, {"linux", "arm64"}},                                                               // 人工智能
	"blockchain": {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                    // 区块链
	"fintech":    {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}},                                         // 金融科技
	"healthcare": {{"linux", "amd64"}, {"windows", "amd64"}},                                                             // 医疗健康
	"education":  {{"linux", "amd64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                                        // 教育
	"research":   {{"plan9", "amd64"}, {"solaris", "amd64"}, {"dragonfly", "amd64"}, {"linux", "riscv64"}},               // 研究和特殊用途
	"academic":   {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                    // 学术研究

	// ========== 物联网和嵌入式 ==========
	"iot":        {{"linux", "arm"}, {"linux", "arm64"}, {"linux", "mips"}, {"linux", "riscv64"}}, // 物联网设备
	"raspberry":  {{"linux", "arm"}, {"linux", "arm64"}},                                          // 树莓派系列
	"arduino":    {{"linux", "arm"}},                                                              // Arduino 兼容
	"industrial": {{"linux", "arm"}, {"linux", "arm64"}, {"linux", "amd64"}},                      // 工业控制
	"automotive": {{"linux", "arm64"}, {"linux", "amd64"}},                                        // 汽车电子
	"robotics":   {{"linux", "arm64"}, {"linux", "arm"}, {"linux", "amd64"}},                      // 机器人
	"sensors":    {{"linux", "arm"}, {"linux", "mips"}},                                           // 传感器网络
	"gateway":    {{"linux", "arm64"}, {"linux", "arm"}, {"linux", "amd64"}},                      // 物联网网关

	// ========== 游戏和娱乐 ==========
	"gaming":     {{"windows", "amd64"}, {"linux", "amd64"}, {"darwin", "amd64"}, {"darwin", "arm64"}}, // 游戏平台
	"gameserver": {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}},                       // 游戏服务器
	"streaming":  {{"linux", "amd64"}, {"linux", "arm64"}},                                             // 流媒体
	"media":      {{"linux", "amd64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                      // 媒体处理
	"broadcast":  {{"linux", "amd64"}, {"windows", "amd64"}},                                           // 广播
	"content":    {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}},                       // 内容管理
	// ========== 开发和测试 ==========
	"development": {{"windows", "amd64"}, {"darwin", "amd64"}, {"linux", "amd64"}, {"darwin", "arm64"}}, // 开发环境
	"test_env":    {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}},  // 测试环境
	"staging":     {{"linux", "amd64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                      // 预发布环境
	"production":  {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}},                       // 生产环境
	"prototype":   {{"linux", "amd64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                      // 原型开发
	"poc":         {{"linux", "amd64"}, {"darwin", "amd64"}},                                            // 概念验证
	"mvp":         {{"linux", "amd64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                      // 最小可行产品
	"beta":        {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}},  // Beta测试
	"canary":      {{"linux", "amd64"}, {"linux", "arm64"}},                                             // 金丝雀部署
	"ab-testing":  {{"linux", "amd64"}, {"linux", "arm64"}},                                             // A/B测试

	// ========== 性能和优化 ==========
	"performance": {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                       // 性能优化
	"benchmark":   {{"linux", "amd64"}, {"darwin", "amd64"}, {"windows", "amd64"}},                                           // 性能基准测试
	"stress":      {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}, {"freebsd", "amd64"}}, // 压力测试
	"load":        {{"linux", "amd64"}, {"linux", "arm64"}},                                                                  // 负载测试
	"scalability": {{"linux", "amd64"}, {"linux", "arm64"}},                                                                  // 可扩展性测试
	"reliability": {{"linux", "amd64"}, {"linux", "arm64"}, {"freebsd", "amd64"}},                                            // 可靠性测试

	// ========== 兼容性和跨平台 ==========
	"compat":     {{"windows", "amd64"}, {"windows", "386"}, {"linux", "amd64"}, {"linux", "386"}, {"darwin", "amd64"}, {"freebsd", "amd64"}},    // 兼容性测试
	"multi-arch": {{"linux", "amd64"}, {"linux", "arm64"}, {"linux", "arm"}, {"linux", "ppc64le"}, {"linux", "s390x"}},                           // 多架构支持
	"cross_plat": {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                                            // 跨平台安全编译
	"pure_go":    {{"windows", "amd64"}, {"linux", "amd64"}, {"darwin", "amd64"}, {"freebsd", "amd64"}, {"js", "wasm"}, {"wasip1", "wasm"}},      // 无CGO纯Go
	"legacy":     {{"windows", "386"}, {"linux", "386"}, {"freebsd", "386"}},                                                                     // 32位遗留系统
	"basic_set":  {{"linux", "amd64"}, {"windows", "amd64"}, {"darwin", "amd64"}},                                                                // 最小支持集
	"modern":     {{"windows", "amd64"}, {"windows", "arm64"}, {"linux", "amd64"}, {"linux", "arm64"}, {"darwin", "amd64"}, {"darwin", "arm64"}}, // 现代主流平台

	// ========== 发布和分发 ==========
	"release":        {{"windows", "amd64"}, {"windows", "arm64"}, {"linux", "amd64"}, {"linux", "arm64"}, {"darwin", "amd64"}, {"darwin", "arm64"}}, // 完整发布
	"github-release": {{"windows", "amd64"}, {"linux", "amd64"}, {"darwin", "amd64"}},                                                                // GitHub Release
	"homebrew":       {{"darwin", "amd64"}, {"darwin", "arm64"}, {"linux", "amd64"}},                                                                 // Homebrew
	"chocolatey":     {{"windows", "amd64"}, {"windows", "arm64"}},                                                                                   // Chocolatey
	"snap":           {{"linux", "amd64"}, {"linux", "arm64"}, {"linux", "arm"}},                                                                     // Snap包
	"flatpak":        {{"linux", "amd64"}, {"linux", "arm64"}},                                                                                       // Flatpak
	"appimage":       {{"linux", "amd64"}, {"linux", "arm64"}},                                                                                       // AppImage
	"rpm":            {{"linux", "amd64"}, {"linux", "arm64"}, {"linux", "ppc64le"}, {"linux", "s390x"}},                                             // RPM包
	"deb":            {{"linux", "amd64"}, {"linux", "arm64"}, {"linux", "arm"}},                                                                     // DEB包
	"docker-hub":     {{"linux", "amd64"}, {"linux", "arm64"}, {"linux", "arm"}},                                                                     // Docker Hub
	"npm":            {{"linux", "amd64"}, {"windows", "amd64"}, {"darwin", "amd64"}, {"darwin", "arm64"}},                                           // NPM包
	"pypi":           {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}, {"darwin", "amd64"}, {"darwin", "arm64"}},                       // PyPI包

	// ========== 云服务商 ==========
	"aws":          {{"linux", "amd64"}, {"linux", "arm64"}},                       // Amazon Web Services
	"azure":        {{"linux", "amd64"}, {"linux", "arm64"}, {"windows", "amd64"}}, // Microsoft Azure
	"gcp":          {{"linux", "amd64"}, {"linux", "arm64"}},                       // Google Cloud Platform
	"alibaba":      {{"linux", "amd64"}, {"linux", "arm64"}},                       // 阿里云
	"tencent":      {{"linux", "amd64"}, {"linux", "arm64"}},                       // 腾讯云
	"huawei":       {{"linux", "amd64"}, {"linux", "arm64"}},                       // 华为云
	"digitalocean": {{"linux", "amd64"}, {"linux", "arm64"}},                       // DigitalOcean
	"linode":       {{"linux", "amd64"}, {"linux", "arm64"}},                       // Linode
	"vultr":        {{"linux", "amd64"}, {"linux", "arm64"}},                       // Vultr
	"oracle":       {{"linux", "amd64"}, {"linux", "arm64"}},                       // Oracle Cloud
}

// 平台组元数据
var platformGroupMetadata = map[string]PlatformGroup{
	"minimal": {
		Name:        "minimal",
		Description: "最小支持集：主流桌面平台",
		Category:    "core",
		Tags:        []string{"essential", "desktop", "mainstream"},
	},
	"desktop": {
		Name:        "desktop",
		Description: "桌面平台：Windows、Linux、macOS 的主流架构",
		Category:    "core",
		Tags:        []string{"desktop", "gui", "mainstream"},
	},
	"server": {
		Name:        "server",
		Description: "服务器平台：适用于后端服务部署",
		Category:    "deployment",
		Tags:        []string{"server", "backend", "production"},
	},
	"mobile": {
		Name:        "mobile",
		Description: "移动平台：Android 和 iOS",
		Category:    "mobile",
		Tags:        []string{"mobile", "cgo-required", "cross-compile"},
	},
	"web": {
		Name:        "web",
		Description: "Web平台：WebAssembly 目标",
		Category:    "web",
		Tags:        []string{"web", "wasm", "browser"},
	},
	"embedded": {
		Name:        "embedded",
		Description: "嵌入式平台：ARM、MIPS、RISC-V 等",
		Category:    "embedded",
		Tags:        []string{"embedded", "iot", "arm", "mips", "riscv"},
	},
	"cgo-required": {
		Name:        "cgo-required",
		Description: "需要CGO支持的平台",
		Category:    "special",
		Tags:        []string{"cgo", "cross-compile", "toolchain"},
	},
	"no-cgo": {
		Name:        "no-cgo",
		Description: "纯Go平台，无需CGO支持",
		Category:    "special",
		Tags:        []string{"pure-go", "simple", "fast"},
	},
}

func main() {
	// 检查Go环境并为Android设备设置必要环境变量
	if !checkGoEnvironment() {
		os.Exit(1) // 如果Go命令不可用，则退出程序
	}

	var (
		sourceFile    = flag.String("s", "", "源Go文件路径 (必需)")
		outputDir     = flag.String("o", "./build", "输出目录")
		platforms     = flag.String("p", "desktop", "目标平台")
		binaryName    = flag.String("n", "", "二进制文件名")
		verboseLevel  = flag.Int("v", 1, "详细程度 (0-3)")
		parallel      = flag.Bool("parallel", true, "并行编译")
		compress      = flag.Bool("c", false, "压缩二进制文件")
		ldflags       = flag.String("ldflags", "", "链接器标志")
		tags          = flag.String("tags", "", "构建标签")
		skipTests     = flag.Bool("skip-tests", false, "跳过测试")
		cleanOutput   = flag.Bool("clean", false, "编译前清理输出目录")
		retryFailures = flag.Bool("retry", true, "失败时重试")
		maxRetries    = flag.Int("max-retries", 2, "最大重试次数")
		maxJobs       = flag.Int("j", 0, "最大并行构建进程数 (0=CPU核心数)")
		skipCGO       = flag.Bool("skip-cgo", false, "自动跳过需要CGO但编译器不可用的平台")
		interactive   = flag.Bool("i", false, "交互式确认构建")
		outputFormat  = flag.String("format", "flat", "输出格式 (folder/flat)")
		listPlatforms = flag.Bool("list", false, "列出所有支持的平台")
		listGroups    = flag.Bool("groups", false, "列出所有平台组合")
		version       = flag.Bool("version", false, "显示版本信息")
		help          = flag.Bool("h", false, "显示帮助信息")
		progress      = flag.Bool("progress", true, "显示进度条")
		quickStart    = flag.Bool("quick", false, "快速开始向导")

		// 新增的组操作参数
		includeGroups = flag.String("include", "", "包含的组 (用逗号分隔)")
		excludeGroups = flag.String("exclude", "", "排除的组 (用逗号分隔)")
		onlyGroups    = flag.String("only", "", "仅包含指定组 (用逗号分隔)")
		exceptGroups  = flag.String("except", "", "排除指定组 (用逗号分隔)")
		customGroup   = flag.String("custom", "", "自定义平台组 (格式: name:os1/arch1,os2/arch2)")
		saveConfig    = flag.String("save-config", "", "保存当前组配置到文件")
		loadConfig    = flag.String("load-config", "", "从文件加载组配置")
		_             = flag.Bool("categories", false, "列出平台组分类")
		_             = flag.String("search", "", "搜索包含指定标签的组")
		_             = flag.Bool("validate", false, "验证组配置的有效性")
		_             = flag.String("group-info", "", "显示指定组的详细信息")
	)
	// 简化的别名
	flag.StringVar(sourceFile, "source", "", "源Go文件路径")
	flag.StringVar(outputDir, "output", "./build", "输出目录")
	flag.StringVar(platforms, "platforms", "desktop", "目标平台")
	flag.StringVar(binaryName, "name", "", "二进制文件名")
	flag.IntVar(verboseLevel, "verbose", 1, "详细程度")
	flag.BoolVar(compress, "compress", false, "压缩二进制文件")
	flag.BoolVar(help, "help", false, "显示帮助信息")

	// 自定义用法信息
	flag.Usage = printCustomUsage

	flag.Parse()

	if *help {
		printHelp()
		return
	}

	if *version {
		printVersion()
		return
	}

	if *listPlatforms {
		printPlatforms()
		return
	}

	if *listGroups {
		printPlatformGroups()
		return
	}

	if *quickStart {
		runQuickStart()
		return
	}

	if *sourceFile == "" {
		printError("必须指定源文件")
		printCustomUsage()
		os.Exit(1)
	}

	// 检查源文件是否存在
	if _, err := os.Stat(*sourceFile); os.IsNotExist(err) {
		printError(fmt.Sprintf("源文件 '%s' 不存在", *sourceFile))
		os.Exit(1)
	}

	// 解析平台列表（支持新的组操作）
	targetPlatforms, err := parsePlatformsAdvanced(*platforms, *includeGroups, *excludeGroups, *onlyGroups, *exceptGroups, *customGroup)
	if err != nil {
		printError(err.Error())
		os.Exit(1)
	}

	// 确定二进制文件名
	if *binaryName == "" {
		base := filepath.Base(*sourceFile)
		*binaryName = strings.TrimSuffix(base, filepath.Ext(base))
	}

	// 清理输出目录
	if *cleanOutput {
		if err := os.RemoveAll(*outputDir); err != nil {
			printWarning(fmt.Sprintf("清理输出目录失败: %v", err))
		} else {
			printSuccess("输出目录已清理")
		}
	}

	// 创建构建器
	builder := &Builder{
		SourceFile:     *sourceFile,
		OutputDir:      *outputDir,
		Platforms:      targetPlatforms,
		BinaryName:     *binaryName,
		Verbose:        *verboseLevel > 0,
		Parallel:       *parallel,
		Compress:       *compress,
		LdFlags:        *ldflags,
		Tags:           *tags,
		SkipTests:      *skipTests,
		CleanOutput:    *cleanOutput,
		ShowProgress:   *progress,
		VerbosityLevel: VerbosityLevel(*verboseLevel),
		RetryFailures:  *retryFailures,
		MaxRetries:     *maxRetries,
		Interactive:    *interactive,
		OutputFormat:   *outputFormat,
		MaxJobs:        *maxJobs,
		SkipCGO:        *skipCGO,

		// 设置组相关配置
		IncludeGroups:   parseGroupList(*includeGroups),
		ExcludeGroups:   parseGroupList(*excludeGroups),
		SaveGroupConfig: *saveConfig,
		LoadGroupConfig: *loadConfig,
	}
	// 设置MaxJobs默认值
	if builder.MaxJobs <= 0 {
		builder.MaxJobs = runtime.NumCPU()
	}

	// CGO编译器检测和自动安装
	if builder.SkipCGO {
		// 检查是否有需要CGO的平台
		hasCGOPlatforms := false
		for _, platform := range builder.Platforms {
			if builder.needsCGO(platform) {
				hasCGOPlatforms = true
				break
			}
		}

		if hasCGOPlatforms {
			// 检查编译器可用性
			compilers := builder.checkCompilerAvailability()
			if !compilers["clang"].Available {
				printWarning("检测到需要 CGO 支持的平台，但 clang 编译器不可用")

				// 提示用户安装编译器
				if builder.promptInstallCompilers() {
					if builder.autoInstallCompilers() {
						printSuccess("编译器安装成功，继续构建...")
					} else {
						printWarning("编译器安装失败，将跳过需要 CGO 的平台")
					}
				} else {
					printInfo("将跳过需要 CGO 的平台")
				}
			}
		}
	}

	// 交互式确认
	if *interactive {
		if !confirmBuild(builder) {
			printInfo("构建已取消")
			return
		}
	}

	// 执行构建
	start := time.Now()
	results, err := builder.BuildWithRetry()
	duration := time.Since(start)

	// 统计结果
	successCount := 0
	failedResults := []BuildResult{}
	for _, result := range results {
		if result.Success {
			successCount++
		} else {
			failedResults = append(failedResults, result)
		}
	}

	if len(failedResults) > 0 {
		printWarning(fmt.Sprintf("部分构建失败: %d/%d 成功", successCount, len(results)))
		for _, result := range failedResults {
			printError(fmt.Sprintf("%s/%s: %v", result.Target.Platform.OS, result.Target.Platform.Arch, result.Error))
		}
	}

	if successCount > 0 {
		printSuccess(fmt.Sprintf("成功编译 %d/%d 个平台的二进制文件到 '%s' (耗时: %v)",
			successCount, len(builder.Platforms), builder.OutputDir, duration))
	}

	if err != nil {
		printError(fmt.Sprintf("构建过程出现错误: %v", err))
		os.Exit(1)
	}

	if len(failedResults) > 0 {
		os.Exit(1)
	}
}

// 检查Go环境并为Android设备设置必要的环境变量
func checkGoEnvironment() bool {
	// 检查是否为Android设备
	isAndroid := false
	if _, err := os.Stat("/data/adb"); err == nil {
		isAndroid = true
	}

	// 首先尝试从各种位置加载环境变量
	// 1. 从环境变量配置文件加载（如果存在）
	envFiles := []string{
		"/data/adb/modules/gogogo/gogogo.env",    // Magisk模块环境配置
		"/data/adb/modules/gogogo/go.env",        // 全局环境
		"/data/local/go.env",                     // 本地环境
		"/sdcard/go.env",                         // 用户环境
		"/data/adb/modules/gogogo/gogogo_env.sh", // 模块脚本
	}

	// 检查环境变量文件，尝试自动加载
	for _, envFile := range envFiles {
		if _, err := os.Stat(envFile); err == nil {
			if strings.HasSuffix(envFile, ".sh") {
				// 这里无法直接在Go中source shell脚本，但可以提示用户
				if os.Getenv("GOGOGO_VERBOSE") != "" {
					fmt.Printf("找到环境变量脚本: %s (无法自动加载)\n", envFile)
				}
			} else if strings.HasSuffix(envFile, ".env") {
				if os.Getenv("GOGOGO_VERBOSE") != "" {
					fmt.Printf("找到环境变量文件: %s (尝试解析)\n", envFile)
				}
				// 尝试解析.env文件
				if data, err := os.ReadFile(envFile); err == nil {
					lines := strings.Split(string(data), "\n")
					for _, line := range lines {
						line = strings.TrimSpace(line)
						// 跳过注释和空行
						if line == "" || strings.HasPrefix(line, "#") {
							continue
						}
						// 处理格式为KEY=VALUE的行
						if parts := strings.SplitN(line, "=", 2); len(parts) == 2 {
							key := strings.TrimSpace(parts[0])
							value := strings.TrimSpace(parts[1])
							// 只设置未定义的环境变量
							if os.Getenv(key) == "" {
								os.Setenv(key, value)
								if os.Getenv("GOGOGO_VERBOSE") != "" {
									fmt.Printf("已从文件设置环境变量: %s=%s\n", key, value)
								}
							}
						}
					}
				}
			}
		}
	}

	// 2. 如果在Android设备上且找到了GoGogo模块，设置所有必要环境变量
	if isAndroid {
		moduleDir := "/data/adb/modules/gogogo"
		if _, err := os.Stat(moduleDir); err == nil {
			fmt.Printf("%s检测到Android环境，自动配置Go环境变量...%s\n", Cyan+Bold, Reset)

			// 设置关键环境变量
			envVars := map[string]string{
				"GOENV":          moduleDir + "/gogogo.env",
				"GOROOT":         moduleDir + "/GOROOT",
				"GOPATH":         moduleDir + "/go",
				"GOCACHE":        moduleDir + "/GOCACHE",
				"GOTELEMETRYDIR": moduleDir + "/GOTELEMETRYDIR",
				"GO111MODULE":    "on",
				"GOMODCACHE":     moduleDir + "/go/pkg/mod",
			}

			// 设置所有环境变量
			for key, value := range envVars {
				if os.Getenv(key) == "" {
					os.Setenv(key, value)
					if os.Getenv("GOGOGO_VERBOSE") != "" {
						fmt.Printf("已设置 %s=%s\n", key, value)
					}
				}
			}

			// 添加Go bin目录到PATH
			currentPath := os.Getenv("PATH")
			goBinPath := moduleDir + "/GOROOT/bin"
			goUserBinPath := moduleDir + "/go/bin"
			systemBinPath := moduleDir + "/system/bin"

			// 检查PATH中是否已包含Go路径
			if !strings.Contains(currentPath, goBinPath) {
				newPath := currentPath + ":" + goBinPath
				os.Setenv("PATH", newPath)
				if os.Getenv("GOGOGO_VERBOSE") != "" {
					fmt.Printf("已将 %s 添加至PATH\n", goBinPath)
				}
			}

			if !strings.Contains(currentPath, goUserBinPath) {
				newPath := os.Getenv("PATH") + ":" + goUserBinPath
				os.Setenv("PATH", newPath)
				if os.Getenv("GOGOGO_VERBOSE") != "" {
					fmt.Printf("已将 %s 添加至PATH\n", goUserBinPath)
				}
			}

			if !strings.Contains(currentPath, systemBinPath) {
				newPath := os.Getenv("PATH") + ":" + systemBinPath
				os.Setenv("PATH", newPath)
				if os.Getenv("GOGOGO_VERBOSE") != "" {
					fmt.Printf("已将 %s 添加至PATH\n", systemBinPath)
				}
			}

			// 验证环境变量设置
			if os.Getenv("GOGOGO_VERBOSE") != "" {
				fmt.Printf("Go环境变量已设置:\n")
				fmt.Printf("  GOROOT=%s\n", os.Getenv("GOROOT"))
				fmt.Printf("  GOPATH=%s\n", os.Getenv("GOPATH"))
				fmt.Printf("  PATH=%s\n", os.Getenv("PATH"))
			}
		}
	}

	// 检查go命令是否可用
	_, err := exec.LookPath("go")
	if err != nil {
		fmt.Printf("%s❌ 错误：未找到 Go 编译器!%s\n\n", Red+Bold, Reset)

		if isAndroid {
			fmt.Printf("%s提示：您似乎在Android设备上运行，请安装 GoGogo Magisk模块%s\n", Yellow+Bold, Reset)
			fmt.Printf("      可以在Magisk模块仓库或GitHub搜索 'GoGogo Module'\n")
			fmt.Printf("      安装后重启设备，或手动加载环境变量:\n")
			fmt.Printf("      $ source /data/adb/modules/gogogo/gogogo_env.sh\n\n")

			// 检查常见问题
			checkAndroidCommonIssues()
		} else {
			fmt.Printf("%s提示：请安装Go编译器并确保已添加到PATH环境变量中%s\n", Yellow+Bold, Reset)
			fmt.Printf("      下载地址: https://golang.org/dl/\n")
			fmt.Printf("      安装指南: https://golang.org/doc/install\n\n")
		}

		return false
	}

	return true
}

// 检查Android设备上的常见问题
func checkAndroidCommonIssues() {
	moduleDir := "/data/adb/modules/gogogo"

	// 检查模块是否已安装
	if _, err := os.Stat(moduleDir); err != nil {
		fmt.Printf("%s警告：未找到GoGogo模块目录%s\n", Yellow, Reset)
		return
	}

	// 检查Go可执行文件是否存在
	goBin := moduleDir + "/GOROOT/bin/go"
	if _, err := os.Stat(goBin); err != nil {
		systemGoBin := "/system/bin/go"
		if _, err := os.Stat(systemGoBin); err != nil {
			fmt.Printf("%s警告：未找到Go可执行文件%s\n", Yellow, Reset)
			fmt.Printf("      请确保模块正确安装并重启设备\n")
		}
	}

	// 检查环境变量脚本是否存在
	envScript := moduleDir + "/gogogo_env.sh"
	if _, err := os.Stat(envScript); err != nil {
		fmt.Printf("%s警告：未找到环境变量脚本%s\n", Yellow, Reset)
	} else {
		fmt.Printf("%s提示：找到环境变量脚本，可以尝试手动加载:%s\n", Green+Bold, Reset)
		fmt.Printf("      source %s\n", envScript)
	}
}

// parsePlatforms 解析平台字符串，支持平台组合
func parsePlatforms(platformsStr string) ([]Platform, error) {
	if platformsStr == "all" {
		return commonPlatforms, nil
	}

	// 检查是否是预定义的平台组合
	if group, exists := platformGroups[platformsStr]; exists {
		return group, nil
	}

	var platforms []Platform
	parts := strings.Split(platformsStr, ",")

	for _, part := range parts {
		part = strings.TrimSpace(part)
		if part == "" {
			continue
		}

		// 检查是否是平台组合
		if group, exists := platformGroups[part]; exists {
			platforms = append(platforms, group...)
			continue
		}

		osArch := strings.Split(part, "/")
		if len(osArch) != 2 {
			return nil, fmt.Errorf("无效的平台格式: %s (应该是 OS/ARCH 格式)", part)
		}

		platforms = append(platforms, Platform{
			OS:   strings.TrimSpace(osArch[0]),
			Arch: strings.TrimSpace(osArch[1]),
		})
	}

	if len(platforms) == 0 {
		return nil, fmt.Errorf("没有指定有效的平台")
	}

	// 去重
	uniquePlatforms := make([]Platform, 0, len(platforms))
	seen := make(map[string]bool)
	for _, p := range platforms {
		key := p.OS + "/" + p.Arch
		if !seen[key] {
			seen[key] = true
			uniquePlatforms = append(uniquePlatforms, p)
		}
	}

	return uniquePlatforms, nil
}

// BuildWithRetry 执行跨平台编译，支持重试机制
func (b *Builder) BuildWithRetry() ([]BuildResult, error) {
	// 创建输出目录
	if err := os.MkdirAll(b.OutputDir, 0755); err != nil {
		return nil, fmt.Errorf("创建输出目录失败: %v", err)
	}

	// 准备构建目标
	targets := b.prepareBuildTargets()

	if b.VerbosityLevel >= VerboseNormal {
		printInfo(fmt.Sprintf("开始编译 %d 个平台...", len(targets)))
	}

	// 第一次构建尝试
	results := b.buildTargets(targets, 1)

	// 检查失败的目标并重试
	if b.RetryFailures && b.MaxRetries > 1 {
		failedTargets := []BuildTarget{}
		for _, result := range results {
			if !result.Success {
				failedTargets = append(failedTargets, result.Target)
			}
		}

		if len(failedTargets) > 0 {
			if b.VerbosityLevel >= VerboseNormal {
				printWarning(fmt.Sprintf("发现 %d 个失败的构建目标，开始重试...", len(failedTargets)))
			}

			// 使用更慢但更稳定的参数重试
			retryResults := b.buildTargetsWithSlowParams(failedTargets, 2)

			// 更新结果
			retryMap := make(map[string]BuildResult)
			for _, result := range retryResults {
				key := result.Target.Platform.OS + "/" + result.Target.Platform.Arch
				retryMap[key] = result
			}

			for i, result := range results {
				if !result.Success {
					key := result.Target.Platform.OS + "/" + result.Target.Platform.Arch
					if retryResult, exists := retryMap[key]; exists {
						results[i] = retryResult
					}
				}
			}
		}
	}

	return results, nil
}

// buildTargets 构建目标列表
func (b *Builder) buildTargets(targets []BuildTarget, attempt int) []BuildResult {
	if b.Parallel {
		return b.buildParallelWithResults(targets, attempt)
	} else {
		return b.buildSequentialWithResults(targets, attempt)
	}
}

// buildTargetsWithSlowParams 使用慢速参数构建目标列表
func (b *Builder) buildTargetsWithSlowParams(targets []BuildTarget, attempt int) []BuildResult {
	// 强制顺序编译并增加详细输出
	originalParallel := b.Parallel
	originalVerbosity := b.VerbosityLevel

	b.Parallel = false              // 禁用并行编译
	b.VerbosityLevel = VerboseDebug // 设置为调试级别

	if b.VerbosityLevel >= VerboseNormal {
		printInfo("重试时使用调试模式，禁用并行编译以获得更稳定的构建")
	}

	results := b.buildSequentialWithResults(targets, attempt)

	// 恢复原始设置
	b.Parallel = originalParallel
	b.VerbosityLevel = originalVerbosity

	return results
}

// prepareBuildTargets 准备构建目标
func (b *Builder) prepareBuildTargets() []BuildTarget {
	var targets []BuildTarget

	// 如果启用了SkipCGO，先进行智能CGO检测和过滤
	platforms := b.Platforms
	if b.SkipCGO {
		validPlatforms, skippedPlatforms := b.filterPlatformsByCGO(b.Platforms)
		platforms = validPlatforms

		// 显示CGO状态信息
		if b.VerbosityLevel >= VerboseNormal && len(skippedPlatforms) > 0 {
			b.printCGOStatus()
			printWarning(fmt.Sprintf("跳过了 %d 个需要CGO但编译器不可用的平台:", len(skippedPlatforms)))
			for _, platform := range skippedPlatforms {
				fmt.Printf("  - %s/%s\n", platform.OS, platform.Arch)
			}
			fmt.Println()
		}
	}

	for _, platform := range platforms {
		outputName := b.BinaryName
		if platform.OS == "windows" {
			outputName += ".exe"
		}

		var outputPath string
		if b.OutputFormat == "flat" {
			// 平铺格式：所有文件放在同一个目录，文件名包含平台信息
			fileName := fmt.Sprintf("%s_%s_%s", b.BinaryName, platform.OS, platform.Arch)
			if platform.OS == "windows" {
				fileName += ".exe"
			}
			outputPath = filepath.Join(b.OutputDir, fileName)
		} else {
			// 文件夹格式：每个平台一个子目录
			outputPath = filepath.Join(b.OutputDir, fmt.Sprintf("%s_%s_%s", b.BinaryName, platform.OS, platform.Arch), outputName)
		}

		targets = append(targets, BuildTarget{
			Platform:   platform,
			OutputName: outputName,
			OutputPath: outputPath,
		})
	}

	return targets
}

// buildParallelWithResults 并行编译并返回结果
func (b *Builder) buildParallelWithResults(targets []BuildTarget, attempt int) []BuildResult {
	var wg sync.WaitGroup
	results := make([]BuildResult, len(targets))

	// 使用信号量控制并发数量
	maxJobs := b.MaxJobs
	if maxJobs <= 0 {
		maxJobs = runtime.NumCPU()
	}

	// 创建带缓冲的channel作为信号量
	semaphore := make(chan struct{}, maxJobs)

	if b.VerbosityLevel >= VerboseDetailed {
		printInfo(fmt.Sprintf("使用 %d 个并行构建进程", maxJobs))
	}

	for i, target := range targets {
		wg.Add(1)
		go func(idx int, t BuildTarget) {
			defer wg.Done()

			// 获取信号量
			semaphore <- struct{}{}
			defer func() { <-semaphore }() // 释放信号量

			start := time.Now()
			err := b.buildTarget(t, attempt)
			duration := time.Since(start)

			results[idx] = BuildResult{
				Target:   t,
				Success:  err == nil,
				Error:    err,
				Duration: duration,
				Attempt:  attempt}

			if b.ShowProgress {
				printProgress(idx+1, len(targets), t, err == nil)
			}
		}(i, target)
	}

	wg.Wait()
	return results
}

// buildSequentialWithResults 顺序编译并返回结果
func (b *Builder) buildSequentialWithResults(targets []BuildTarget, attempt int) []BuildResult {
	results := make([]BuildResult, len(targets))

	for i, target := range targets {
		start := time.Now()
		err := b.buildTarget(target, attempt)
		duration := time.Since(start)

		results[i] = BuildResult{
			Target:   target,
			Success:  err == nil,
			Error:    err,
			Duration: duration,
			Attempt:  attempt,
		}

		if b.ShowProgress {
			printProgress(i+1, len(targets), target, err == nil)
		}
	}

	return results
}

// buildTarget 编译单个目标
func (b *Builder) buildTarget(target BuildTarget, attempt int) error {
	// 创建输出目录
	outputDir := filepath.Dir(target.OutputPath)
	if err := os.MkdirAll(outputDir, 0755); err != nil {
		return fmt.Errorf("创建目录 %s 失败: %v", outputDir, err)
	}

	if b.VerbosityLevel >= VerboseDetailed {
		printInfo(fmt.Sprintf("编译 %s/%s (尝试 %d)...", target.Platform.OS, target.Platform.Arch, attempt))
	}

	// 准备编译命令
	args := []string{"build", "-o", target.OutputPath}

	// 添加额外的构建标志
	if b.LdFlags != "" {
		args = append(args, "-ldflags", b.LdFlags)
	}

	if b.Tags != "" {
		args = append(args, "-tags", b.Tags)
	}

	// 如果是重试，添加更慢但更稳定的参数
	if attempt > 1 {
		args = append(args, "-a") // 强制重新构建所有包
		if b.VerbosityLevel >= VerboseDebug {
			args = append(args, "-x") // 显示执行的命令
		}
	}
	args = append(args, b.SourceFile)

	cmd := exec.Command("go", args...)

	// 确定是否需要启用CGO
	cgoEnabled := b.needsCGO(target.Platform)

	cmd.Env = append(os.Environ(),
		fmt.Sprintf("GOOS=%s", target.Platform.OS),
		fmt.Sprintf("GOARCH=%s", target.Platform.Arch),
		fmt.Sprintf("CGO_ENABLED=%s", map[bool]string{true: "1", false: "0"}[cgoEnabled]),
	) // 执行编译
	output, err := cmd.CombinedOutput()
	if err != nil {
		// 检查是否是CGO相关的错误，提供更友好的错误信息
		if b.isCGOError(output, target.Platform) {
			cgoError := fmt.Sprintf("CGO编译失败: %s/%s 平台需要 C 编译器支持", target.Platform.OS, target.Platform.Arch)
			if target.Platform.OS == "ios" {
				cgoError += "\n建议: iOS 编译需要安装 Xcode 命令行工具: xcode-select --install"
			}
			if b.VerbosityLevel >= VerboseDetailed {
				printError(cgoError)
				if b.VerbosityLevel >= VerboseDebug {
					fmt.Printf("详细错误输出:\n%s\n", string(output))
				}
			}
			return fmt.Errorf("CGO编译失败: 缺少 C 编译器")
		}

		if b.VerbosityLevel >= VerboseDetailed {
			printError(fmt.Sprintf("编译 %s/%s 失败 (尝试 %d): %v", target.Platform.OS, target.Platform.Arch, attempt, err))
			if b.VerbosityLevel >= VerboseDebug {
				fmt.Printf("命令输出:\n%s\n", string(output))
			}
		}
		return fmt.Errorf("编译失败: %v", err)
	}

	// 检查是否是 CGO 相关的错误
	if b.isCGOError(output, target.Platform) {
		return fmt.Errorf("CGO 编译失败: 请确保安装了相应的 C 编译器（如 clang 或 gcc）")
	}

	// 压缩二进制文件（如果启用）
	if b.Compress {
		if err := b.compressBinary(target.OutputPath); err != nil {
			printWarning(fmt.Sprintf("压缩 %s/%s 失败: %v", target.Platform.OS, target.Platform.Arch, err))
		}
	}

	if b.VerbosityLevel >= VerboseDetailed {
		printSuccess(fmt.Sprintf("✅ %s/%s 编译完成: %s", target.Platform.OS, target.Platform.Arch, target.OutputPath))
	}

	return nil
}

// 检查是否是 CGO 相关的错误
func (b *Builder) isCGOError(output []byte, platform Platform) bool {
	outputStr := string(output)
	cgoErrorPatterns := []string{
		"C compiler \"clang\" not found",
		"C compiler \"gcc\" not found",
		"C compiler \"cc\" not found",
		"cgo: C compiler",
		"exec: \"clang\": executable file not found",
		"exec: \"gcc\": executable file not found",
	}

	for _, pattern := range cgoErrorPatterns {
		if strings.Contains(outputStr, pattern) {
			return true
		}
	}
	return false
}

// compressBinary 压缩二进制文件
func (b *Builder) compressBinary(binaryPath string) error {
	// 读取原始文件
	inputFile, err := os.Open(binaryPath)
	if err != nil {
		return err
	}
	defer inputFile.Close()

	// 创建压缩文件
	outputPath := binaryPath + ".gz"
	outputFile, err := os.Create(outputPath)
	if err != nil {
		return err
	}
	defer outputFile.Close()

	// 创建gzip writer
	gzipWriter := gzip.NewWriter(outputFile)
	defer gzipWriter.Close()
	// 复制数据
	_, err = io.Copy(gzipWriter, inputFile)
	return err
}

// 颜色输出函数
func printError(msg string) {
	fmt.Printf("%s❌ 错误: %s%s\n", Red, msg, Reset)
}

func printWarning(msg string) {
	fmt.Printf("%s⚠️  警告: %s%s\n", Yellow, msg, Reset)
}

func printSuccess(msg string) {
	fmt.Printf("%s✅ %s%s\n", Green, msg, Reset)
}

func printInfo(msg string) {
	fmt.Printf("%s🔹 %s%s\n", Blue, msg, Reset)
}

func printDebug(msg string) {
	fmt.Printf("%s🔧 调试: %s%s\n", Purple, msg, Reset)
}

// 进度条显示
func printProgress(current, total int, target BuildTarget, success bool) {
	status := "🔄"
	color := Yellow
	statusText := "编译中"

	if success {
		status = "✅"
		color = Green
		statusText = "成功"
	} else {
		status = "❌"
		color = Red
		statusText = "失败"
	}

	// 每个平台显示一行，便于查看编译速度
	fmt.Printf("%s%s (%d/%d) %s/%s - %s%s\n",
		color, status, current, total,
		target.Platform.OS, target.Platform.Arch,
		statusText, Reset)
}

// 自定义用法信息
func printCustomUsage() {
	fmt.Printf(`%s🚀 gogogo v2.0.0 - Go跨平台编译工具-Android-专版%s

%s💡 快速开始:%s
  gogogo -s main.go                    # 编译桌面平台
  gogogo -s main.go -i                 # 交互式编译
  gogogo -quick                        # 快速开始向导

%s📋 常用命令:%s
  -s <文件>     源文件 (必需)
  -o <目录>     输出目录 (默认: ./build)
  -p <平台>     目标平台 (默认: all)
  -i            交互式确认构建
  -v <级别>     详细程度 (0-3)
  -c            压缩输出
  -clean        清理输出目录
  -format <格式> 输出格式 (folder/flat，默认: flat)

%s🎯 平台选项:%s
  desktop       桌面平台 (Windows, Linux, macOS)
  server        服务器平台
  mobile        移动平台
  web           WebAssembly
  all           所有平台
  -list         查看所有支持的平台
  -groups       查看平台组合

%s🔧 高级选项:%s
  -parallel     并行编译 (默认开启)
  -retry        自动重试 (默认开启)
  -ldflags      链接器标志
  -tags         构建标签

%s❓ 帮助:%s
  -h, -help     显示详细帮助
  -version      显示版本信息
  -quick        快速开始向导

`, Bold+Green, Reset,
		Bold+Yellow, Reset,
		Bold+Blue, Reset,
		Bold+Purple, Reset,
		Bold+Cyan, Reset,
		Bold+Red, Reset)
}

// 快速开始向导
func runQuickStart() {
	reader := bufio.NewReader(os.Stdin)

	fmt.Printf("%s🚀 gogogo 快速开始向导%s\n\n", Bold+Green, Reset)

	// 1. 选择源文件
	fmt.Printf("%s1. 请输入Go源文件路径:%s ", Bold+Yellow, Reset)
	sourceFile, _ := reader.ReadString('\n')
	sourceFile = strings.TrimSpace(sourceFile)

	if sourceFile == "" {
		printError("必须指定源文件")
		return
	}

	// 2. 选择平台
	fmt.Printf("\n%s2. 选择目标平台:%s\n", Bold+Yellow, Reset)
	fmt.Println("  1) desktop  - 桌面平台 (推荐)")
	fmt.Println("  2) server   - 服务器平台")
	fmt.Println("  3) mobile   - 移动平台")
	fmt.Println("  4) web      - WebAssembly")
	fmt.Println("  5) all      - 所有平台")
	fmt.Println("  6) 自定义   - 手动指定")

	fmt.Printf("请选择 (1-6, 默认1): ")
	choice, _ := reader.ReadString('\n')
	choice = strings.TrimSpace(choice)

	platformMap := map[string]string{
		"1": "desktop",
		"2": "server",
		"3": "mobile",
		"4": "web",
		"5": "all",
		"":  "desktop",
	}

	platform := platformMap[choice]
	if choice == "6" {
		fmt.Printf("请输入平台 (如: windows/amd64,linux/amd64): ")
		platform, _ = reader.ReadString('\n')
		platform = strings.TrimSpace(platform)
	}

	// 3. 其他选项
	fmt.Printf("\n%s3. 其他选项:%s\n", Bold+Yellow, Reset)
	fmt.Printf("压缩输出? (y/N): ")
	compressChoice, _ := reader.ReadString('\n')
	compress := strings.ToLower(strings.TrimSpace(compressChoice)) == "y"

	fmt.Printf("详细输出? (y/N): ")
	verboseChoice, _ := reader.ReadString('\n')
	verbose := strings.ToLower(strings.TrimSpace(verboseChoice)) == "y"

	// 4. 构建命令
	fmt.Printf("\n%s🔨 生成的命令:%s\n", Bold+Green, Reset)
	cmd := fmt.Sprintf("gogogo -s %s -p %s", sourceFile, platform)
	if compress {
		cmd += " -c"
	}
	if verbose {
		cmd += " -v 2"
	}

	fmt.Printf("%s%s%s\n", Bold+Cyan, cmd, Reset)

	fmt.Printf("\n现在执行构建? (Y/n): ")
	executeChoice, _ := reader.ReadString('\n')
	if strings.ToLower(strings.TrimSpace(executeChoice)) != "n" {
		// 执行构建逻辑
		printInfo("开始构建...")
		// 这里可以调用实际的构建逻辑
	}
}

// 交互式确认构建
func confirmBuild(builder *Builder) bool {
	reader := bufio.NewReader(os.Stdin)

	for {
		fmt.Printf("\n%s📋 构建配置确认%s\n", Bold+Cyan, Reset)
		fmt.Printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n")
		fmt.Printf("%s源文件:%s     %s\n", Bold+Yellow, Reset, builder.SourceFile)
		fmt.Printf("%s输出目录:%s   %s\n", Bold+Yellow, Reset, builder.OutputDir)
		fmt.Printf("%s二进制名:%s   %s\n", Bold+Yellow, Reset, builder.BinaryName)
		fmt.Printf("%s目标平台:%s   %d 个平台\n", Bold+Yellow, Reset, len(builder.Platforms))

		for i, platform := range builder.Platforms {
			if i < 5 {
				fmt.Printf("           %s/%s\n", platform.OS, platform.Arch)
			} else if i == 5 {
				fmt.Printf("           ... 还有 %d 个平台\n", len(builder.Platforms)-5)
				break
			}
		}
		fmt.Printf("%s输出格式:%s   %s\n", Bold+Yellow, Reset, builder.OutputFormat)
		fmt.Printf("%s并行编译:%s   %v (最大 %d 进程)\n", Bold+Yellow, Reset, builder.Parallel, builder.MaxJobs)
		fmt.Printf("%s压缩输出:%s   %v\n", Bold+Yellow, Reset, builder.Compress)
		fmt.Printf("%s自动重试:%s   %v (最多 %d 次)\n", Bold+Yellow, Reset, builder.RetryFailures, builder.MaxRetries)
		fmt.Printf("%s跳过CGO:%s    %v\n", Bold+Yellow, Reset, builder.SkipCGO)
		fmt.Printf("%s详细程度:%s   级别 %d\n", Bold+Yellow, Reset, int(builder.VerbosityLevel))

		if builder.LdFlags != "" {
			fmt.Printf("%s链接标志:%s   %s\n", Bold+Yellow, Reset, builder.LdFlags)
		}
		if builder.Tags != "" {
			fmt.Printf("%s构建标签:%s   %s\n", Bold+Yellow, Reset, builder.Tags)
		}
		fmt.Printf("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n")
		fmt.Printf("\n%s选择操作:%s\n", Bold+Green, Reset)
		fmt.Printf("  1) 开始构建\n")
		fmt.Printf("  2) 修改输出目录\n")
		fmt.Printf("  3) 修改目标平台\n")
		fmt.Printf("  4) 修改输出格式 (folder/flat)\n")
		fmt.Printf("  5) 切换并行编译\n")
		fmt.Printf("  6) 修改并行进程数\n")
		fmt.Printf("  7) 切换压缩输出\n")
		fmt.Printf("  8) 修改详细程度 (0-3)\n")
		fmt.Printf("  9) 切换CGO跳过模式\n")
		fmt.Printf("  a) 修改链接器标志\n")
		fmt.Printf("  0) 取消构建\n")
		fmt.Printf("\n请选择 (0-9,a, 默认1): ")
		choice, _ := reader.ReadString('\n')
		choice = strings.TrimSpace(choice)
		// 转换成小写以支持大小写无关的输入
		choice = strings.ToLower(choice)
		switch choice {
		case "", "1":
			return true
		case "2":
			fmt.Printf("输入新的输出目录 (当前: %s): ", builder.OutputDir)
			newOutputDir, _ := reader.ReadString('\n')
			newOutputDir = strings.TrimSpace(newOutputDir)
			if newOutputDir != "" {
				builder.OutputDir = newOutputDir
				printSuccess("输出目录已更新")
			}
		case "3":
			fmt.Printf("\n%s🎯 选择目标平台:%s\n", Bold+Yellow, Reset)
			fmt.Println("  0) all      - 所有平台")
			fmt.Println("  1) desktop  - 桌面平台 (Windows, Linux, macOS)")
			fmt.Println("  2) server   - 服务器平台 (Linux, FreeBSD)")
			fmt.Println("  3) mobile   - 移动平台 (Android, iOS)")
			fmt.Println("  4) web      - Web平台 (WebAssembly)")
			fmt.Println("  5) embedded - 嵌入式平台 (ARM, MIPS, RISC-V)")
			fmt.Println("  6) windows  - Windows平台")
			fmt.Println("  7) linux    - Linux平台")
			fmt.Println("  8) darwin   - macOS平台")
			fmt.Println("  9) bsd      - BSD平台")
			fmt.Println("  a) android  - Android平台")
			fmt.Println("  b) ios      - iOS平台")
			fmt.Println("  c) 自定义   - 手动输入")

			fmt.Printf("请选择 (0-9, c): ")
			platformChoice, _ := reader.ReadString('\n')
			platformChoice = strings.TrimSpace(platformChoice)

			var platformStr string // 转换成小写以支持大小写无关的输入
			platformChoice = strings.ToLower(platformChoice)
			switch platformChoice {
			case "0":
				platformStr = "all"
			case "1", "":
				platformStr = "desktop"
			case "2":
				platformStr = "server"
			case "3":
				platformStr = "mobile"
			case "4":
				platformStr = "web"
			case "5":
				platformStr = "embedded"
			case "6":
				platformStr = "windows"
			case "7":
				platformStr = "linux"
			case "8":
				platformStr = "darwin"
			case "9":
				platformStr = "bsd"
			case "a":
				platformStr = "android"
			case "b":
				platformStr = "ios"
			case "c":
				fmt.Printf("输入自定义平台 (如: windows/amd64,linux/amd64): ")
				platformStr, _ = reader.ReadString('\n')
				platformStr = strings.TrimSpace(platformStr)
			default:
				printError("无效选择")
				continue
			}
			if platformStr != "" {
				platforms, err := parsePlatforms(platformStr)
				if err != nil {
					printError(fmt.Sprintf("平台解析失败: %v", err))
				} else {
					builder.Platforms = platforms
					printSuccess(fmt.Sprintf("目标平台已更新为: %s (%d个平台)", platformStr, len(platforms)))
				}
			}
		case "4":
			if builder.OutputFormat == "folder" {
				builder.OutputFormat = "flat"
			} else {
				builder.OutputFormat = "folder"
			}
			printSuccess(fmt.Sprintf("输出格式已更改为: %s", builder.OutputFormat))
		case "5":
			builder.Parallel = !builder.Parallel
			printSuccess(fmt.Sprintf("并行编译已%s", map[bool]string{true: "启用", false: "禁用"}[builder.Parallel]))
		case "6":
			fmt.Printf("输入并行进程数 (0=CPU核心数, 当前: %d): ", builder.MaxJobs)
			jobsStr, _ := reader.ReadString('\n')
			jobsStr = strings.TrimSpace(jobsStr)
			if jobs, err := strconv.Atoi(jobsStr); err == nil && jobs >= 0 {
				builder.MaxJobs = jobs
				if builder.MaxJobs == 0 {
					builder.MaxJobs = runtime.NumCPU()
				}
				printSuccess(fmt.Sprintf("并行进程数已设置为: %d", builder.MaxJobs))
			} else {
				printError("无效的进程数，请输入 >= 0 的整数")
			}
		case "7":
			builder.Compress = !builder.Compress
			printSuccess(fmt.Sprintf("压缩输出已%s", map[bool]string{true: "启用", false: "禁用"}[builder.Compress]))
		case "8":
			fmt.Printf("输入详细程度 (0=安静, 1=正常, 2=详细, 3=调试): ")
			verboseStr, _ := reader.ReadString('\n')
			verboseStr = strings.TrimSpace(verboseStr)
			if verboseLevel, err := strconv.Atoi(verboseStr); err == nil && verboseLevel >= 0 && verboseLevel <= 3 {
				builder.VerbosityLevel = VerbosityLevel(verboseLevel)
				builder.Verbose = verboseLevel > 0
				printSuccess(fmt.Sprintf("详细程度已设置为: %d", verboseLevel))
			} else {
				printError("无效的详细程度，请输入 0-3")
			}
		case "9":
			builder.SkipCGO = !builder.SkipCGO
			printSuccess(fmt.Sprintf("CGO跳过模式已%s", map[bool]string{true: "启用", false: "禁用"}[builder.SkipCGO]))
		case "a":
			fmt.Printf("输入链接器标志 (当前: %s): ", builder.LdFlags)
			newLdFlags, _ := reader.ReadString('\n')
			newLdFlags = strings.TrimSpace(newLdFlags)
			builder.LdFlags = newLdFlags
			printSuccess("链接器标志已更新")
		case "0":
			return false
		default:
			printWarning("无效选择，请重新选择")
		}
	}
}

// 帮助信息
func printHelp() {
	fmt.Printf(`%s%sgogogo v2.0.0 - Go跨平台编译工具%s

%s用法:%s
  gogogo -s <源文件> [选项]

%s基础选项:%s
  -s, -source <文件>     源Go文件路径 (必需)
  -o, -output <目录>     输出目录 (默认: ./build)
  -n, -name <名称>       二进制文件名 (默认: 源文件名)
  -p, -platforms <平台>  目标平台 (默认: desktop)

%s平台选项:%s
  -list                  列出所有支持的平台
  -groups                列出所有平台组合
  
  预设平台组合:
    desktop    桌面平台 (Windows, Linux, macOS)
    server     服务器平台 (Linux, FreeBSD)
    mobile     移动平台 (Android, iOS)
    web        Web平台 (WebAssembly)
    embedded   嵌入式平台 (ARM, MIPS, RISC-V)
    all        所有支持的平台

%s构建选项:%s
  -v, -verbose <级别>    详细程度 (0=安静, 1=正常, 2=详细, 3=调试)
  -parallel              并行编译 (默认: true)
  -c, -compress          压缩二进制文件
  -clean                 编译前清理输出目录
  -retry                 失败时重试 (默认: true)
  -max-retries <次数>    最大重试次数 (默认: 2)
  -progress              显示进度条 (默认: true)

%s高级选项:%s
  -ldflags <标志>        链接器标志
  -tags <标签>           构建标签
  -skip-tests            跳过测试
  -skip-cgo              跳过需要CGO支持的平台（如Android和iOS）

%s环境设置:%s
  程序启动时会自动检查Go环境配置：
    - 检测Go命令是否可用
    - Android设备上自动设置GOENV环境变量
    - Magisk模块路径: /data/adb/modules/gogogo

%s其他选项:%s
  -h, -help              显示帮助信息
  -version               显示版本信息

%s示例:%s
  # 编译桌面平台
  gogogo -s main.go

  # 编译指定平台
  gogogo -s main.go -p windows/amd64,linux/amd64

  # 详细输出并压缩
  gogogo -s main.go -v 2 -c

  # 编译所有平台，清理输出目录
  gogogo -s main.go -p all -clean

  # 在Android设备上编译
  gogogo -s main.go -p android/arm64,android/arm

  # 安静模式编译
  gogogo -s main.go -v 0
`, Bold+Cyan, Bold, Reset,
		Bold+Yellow, Reset,
		Bold+Green, Reset,
		Bold+Blue, Reset,
		Bold+Purple, Reset, runtime.Version(),
		Bold+Cyan, Reset, runtime.GOOS, runtime.GOARCH)
}

// 打印平台列表
func printPlatforms() {
	fmt.Printf("%s%s支持的平台 (%d个):%s\n\n", Bold+Cyan, Bold, len(commonPlatforms), Reset)

	// 按操作系统分组
	osGroups := make(map[string][]Platform)
	for _, p := range commonPlatforms {
		osGroups[p.OS] = append(osGroups[p.OS], p)
	}

	osOrder := []string{"windows", "linux", "darwin", "freebsd", "openbsd", "netbsd", "dragonfly", "solaris", "aix", "plan9", "android", "ios", "js", "wasip1"}

	for _, os := range osOrder {
		if platforms, exists := osGroups[os]; exists {
			fmt.Printf("%s%s%s:%s\n", Bold+Green, strings.ToUpper(os), Reset, Reset)
			for _, p := range platforms {
				fmt.Printf("  %s/%s\n", p.OS, p.Arch)
			}
			fmt.Println()
		}
	}
}

// 打印平台组合
func printPlatformGroups() {
	fmt.Printf("%s%s平台组合:%s\n\n", Bold+Cyan, Bold, Reset)

	groupOrder := []string{"desktop", "server", "mobile", "web", "embedded", "windows", "linux", "darwin", "bsd"}

	for _, group := range groupOrder {
		if platforms, exists := platformGroups[group]; exists {
			fmt.Printf("%s%s%s:%s (%d个平台)\n", Bold+Yellow, group, Reset, Reset, len(platforms))
			for _, p := range platforms {
				fmt.Printf("  %s/%s\n", p.OS, p.Arch)
			}
			fmt.Println()
		}
	}
}

// CompilerInfo 编译器信息
type CompilerInfo struct {
	Available bool
	Path      string
	Version   string
	Type      string // "gcc", "clang", "cl"
}

// CGORequirement CGO要求信息
type CGORequirement struct {
	Platform      Platform
	RequiredTools []string
	Available     bool
	Reason        string
}

// CompilerInstaller 编译器安装器
type CompilerInstaller struct {
	OS             string
	Architecture   string
	PackageManager string
}

// InstallResult 安装结果
type InstallResult struct {
	Success   bool
	Message   string
	Installed []string
	Failed    []string
}

// checkCompilerAvailability 检测编译器可用性
func (b *Builder) checkCompilerAvailability() map[string]CompilerInfo {
	compilers := make(map[string]CompilerInfo)

	// 检测常见的C编译器
	candidateCompilers := []struct {
		name string
		cmd  string
		typ  string
	}{
		{"gcc", "gcc", "gcc"},
		{"clang", "clang", "clang"},
		{"cl", "cl", "cl"}, // Microsoft Visual C++
	}

	for _, compiler := range candidateCompilers {
		info := CompilerInfo{Type: compiler.typ}

		// 检查编译器是否可用
		if path, err := exec.LookPath(compiler.cmd); err == nil {
			info.Available = true
			info.Path = path

			// 尝试获取版本信息
			if cmd := exec.Command(compiler.cmd, "--version"); cmd != nil {
				if output, err := cmd.Output(); err == nil {
					lines := strings.Split(string(output), "\n")
					if len(lines) > 0 {
						info.Version = strings.TrimSpace(lines[0])
					}
				}
			}
		}

		compilers[compiler.name] = info
	}

	return compilers
}

// checkCGORequirements 检查CGO要求
func (b *Builder) checkCGORequirements(platforms []Platform) []CGORequirement {
	requirements := make([]CGORequirement, 0)
	compilers := b.checkCompilerAvailability()

	for _, platform := range platforms {
		if !b.needsCGO(platform) {
			continue
		}

		req := CGORequirement{
			Platform: platform,
		}

		// 根据平台确定需要的编译器
		switch platform.OS {
		case "android":
			req.RequiredTools = []string{"clang"}
			if compiler, ok := compilers["clang"]; ok && compiler.Available {
				req.Available = true
			} else {
				req.Available = false
				req.Reason = "需要clang编译器用于Android交叉编译"
			}
		case "ios":
			req.RequiredTools = []string{"clang"}
			if compiler, ok := compilers["clang"]; ok && compiler.Available {
				req.Available = true
			} else {
				req.Available = false
				req.Reason = "需要clang编译器用于iOS交叉编译"
			}
		case "windows":
			// Windows可以使用多种编译器
			req.RequiredTools = []string{"gcc", "clang", "cl"}
			if compilers["gcc"].Available || compilers["clang"].Available || compilers["cl"].Available {
				req.Available = true
			} else {
				req.Available = false
				req.Reason = "需要GCC、Clang或MSVC编译器"
			}
		default:
			// 其他平台通常使用gcc或clang
			req.RequiredTools = []string{"gcc", "clang"}
			if compilers["gcc"].Available || compilers["clang"].Available {
				req.Available = true
			} else {
				req.Available = false
				req.Reason = "需要GCC或Clang编译器"
			}
		}

		requirements = append(requirements, req)
	}

	return requirements
}

// detectPackageManager 检测系统包管理器
func (ci *CompilerInstaller) detectPackageManager() string {
	if ci.PackageManager != "" {
		return ci.PackageManager
	}

	// Windows 包管理器检测
	if runtime.GOOS == "windows" {
		// 检测 Scoop
		if _, err := exec.LookPath("scoop"); err == nil {
			return "scoop"
		}
		// 检测 Chocolatey
		if _, err := exec.LookPath("choco"); err == nil {
			return "chocolatey"
		}
		// 检测 winget
		if _, err := exec.LookPath("winget"); err == nil {
			return "winget"
		}
		return "manual"
	}

	// macOS 包管理器检测
	if runtime.GOOS == "darwin" {
		// 检测 Homebrew
		if _, err := exec.LookPath("brew"); err == nil {
			return "homebrew"
		}
		return "xcode"
	}

	// Linux 包管理器检测
	if runtime.GOOS == "linux" {
		// 检测各种Linux包管理器
		if _, err := exec.LookPath("apt"); err == nil {
			return "apt"
		}
		if _, err := exec.LookPath("yum"); err == nil {
			return "yum"
		}
		if _, err := exec.LookPath("dnf"); err == nil {
			return "dnf"
		}
		if _, err := exec.LookPath("pacman"); err == nil {
			return "pacman"
		}
		if _, err := exec.LookPath("zypper"); err == nil {
			return "zypper"
		}
	}

	return "unknown"
}

// installClang 安装 clang 编译器
func (ci *CompilerInstaller) installClang() InstallResult {
	result := InstallResult{}
	packageManager := ci.detectPackageManager()

	fmt.Printf("%s正在尝试安装 clang 编译器...%s\n", Yellow, Reset)
	fmt.Printf("检测到包管理器: %s\n", packageManager)

	var commands [][]string
	var description string

	switch packageManager {
	case "scoop":
		commands = [][]string{
			{"scoop", "bucket", "add", "main"},
			{"scoop", "install", "llvm"},
		}
		description = "使用 Scoop 安装 LLVM (包含 clang)"

	case "chocolatey":
		commands = [][]string{
			{"choco", "install", "llvm", "-y"},
		}
		description = "使用 Chocolatey 安装 LLVM (包含 clang)"

	case "winget":
		commands = [][]string{
			{"winget", "install", "LLVM.LLVM"},
		}
		description = "使用 winget 安装 LLVM (包含 clang)"

	case "homebrew":
		commands = [][]string{
			{"brew", "install", "llvm"},
		}
		description = "使用 Homebrew 安装 LLVM (包含 clang)"

	case "xcode":
		commands = [][]string{
			{"xcode-select", "--install"},
		}
		description = "安装 Xcode 命令行工具 (包含 clang)"

	case "apt":
		commands = [][]string{
			{"sudo", "apt", "update"},
			{"sudo", "apt", "install", "-y", "clang"},
		}
		description = "使用 apt 安装 clang"

	case "yum":
		commands = [][]string{
			{"sudo", "yum", "install", "-y", "clang"},
		}
		description = "使用 yum 安装 clang"

	case "dnf":
		commands = [][]string{
			{"sudo", "dnf", "install", "-y", "clang"},
		}
		description = "使用 dnf 安装 clang"

	case "pacman":
		commands = [][]string{
			{"sudo", "pacman", "-S", "--noconfirm", "clang"},
		}
		description = "使用 pacman 安装 clang"

	case "zypper":
		commands = [][]string{
			{"sudo", "zypper", "install", "-y", "clang"},
		}
		description = "使用 zypper 安装 clang"

	default:
		result.Success = false
		result.Message = fmt.Sprintf("不支持的包管理器: %s。请手动安装 clang 编译器。", packageManager)
		return result
	}

	fmt.Printf("%s%s%s\n", Cyan, description, Reset)

	// 执行安装命令
	for i, cmd := range commands {
		fmt.Printf("执行命令 %d/%d: %s\n", i+1, len(commands), strings.Join(cmd, " "))

		execCmd := exec.Command(cmd[0], cmd[1:]...)
		output, err := execCmd.CombinedOutput()

		if err != nil {
			result.Success = false
			result.Message = fmt.Sprintf("安装失败: %v\n输出: %s", err, string(output))
			result.Failed = append(result.Failed, strings.Join(cmd, " "))
			return result
		}

		if len(output) > 0 {
			fmt.Printf("输出: %s\n", string(output))
		}
	}

	// 验证安装
	if path, err := exec.LookPath("clang"); err == nil {
		result.Success = true
		result.Message = fmt.Sprintf("✅ clang 安装成功: %s", path)
		result.Installed = append(result.Installed, "clang")

		// 获取版本信息
		if cmd := exec.Command("clang", "--version"); cmd != nil {
			if output, err := cmd.Output(); err == nil {
				lines := strings.Split(string(output), "\n")
				if len(lines) > 0 {
					fmt.Printf("版本信息: %s\n", strings.TrimSpace(lines[0]))
				}
			}
		}
	} else {
		result.Success = false
		result.Message = "安装命令执行成功，但仍无法找到 clang。可能需要重启终端或更新 PATH 环境变量。"
	}

	return result
}

// promptInstallCompilers 提示用户安装编译器
func (b *Builder) promptInstallCompilers() bool {
	compilers := b.checkCompilerAvailability()

	// 检查哪些编译器缺失
	var missingCompilers []string
	if !compilers["clang"].Available {
		missingCompilers = append(missingCompilers, "clang")
	}

	if len(missingCompilers) == 0 {
		return false
	}

	fmt.Printf("\n%s🔧 检测到缺失的编译器: %s%s\n", Yellow, strings.Join(missingCompilers, ", "), Reset)
	fmt.Printf("这些编译器是 Android 和 iOS 平台编译所必需的。\n\n")

	if !b.Interactive {
		// 非交互模式下询问用户
		fmt.Printf("是否要自动安装缺失的编译器？ (y/N): ")
		reader := bufio.NewReader(os.Stdin)
		response, _ := reader.ReadString('\n')
		response = strings.TrimSpace(strings.ToLower(response))
		return response == "y" || response == "yes"
	}

	return true // 交互模式下默认允许安装
}

// autoInstallCompilers 自动安装编译器
func (b *Builder) autoInstallCompilers() bool {
	installer := &CompilerInstaller{
		OS:           runtime.GOOS,
		Architecture: runtime.GOARCH,
	}

	result := installer.installClang()

	if result.Success {
		fmt.Printf("\n%s%s%s\n", Green, result.Message, Reset)
		return true
	} else {
		fmt.Printf("\n%s❌ %s%s\n", Red, result.Message, Reset)

		// 提供手动安装指导
		b.printManualInstallInstructions()
		return false
	}
}

// printManualInstallInstructions 打印手动安装指导
func (b *Builder) printManualInstallInstructions() {
	fmt.Printf("\n%s📖 手动安装指导:%s\n", Cyan, Reset)

	switch runtime.GOOS {
	case "windows":
		fmt.Printf("Windows 系统:\n")
		fmt.Printf("1. 安装 Scoop: Set-ExecutionPolicy RemoteSigned -scope CurrentUser; iwr -useb get.scoop.sh | iex\n")
		fmt.Printf("2. 安装 LLVM: scoop install llvm\n")
		fmt.Printf("或者:\n")
		fmt.Printf("1. 安装 Chocolatey: 访问 https://chocolatey.org/install\n")
		fmt.Printf("2. 安装 LLVM: choco install llvm\n")
		fmt.Printf("或者:\n")
		fmt.Printf("1. 直接下载 LLVM: https://releases.llvm.org/download.html\n")

	case "darwin":
		fmt.Printf("macOS 系统:\n")
		fmt.Printf("1. 安装 Xcode 命令行工具: xcode-select --install\n")
		fmt.Printf("或者:\n")
		fmt.Printf("1. 安装 Homebrew: /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"\n")
		fmt.Printf("2. 安装 LLVM: brew install llvm\n")

	case "linux":
		fmt.Printf("Linux 系统:\n")
		fmt.Printf("Ubuntu/Debian: sudo apt update && sudo apt install clang\n")
		fmt.Printf("CentOS/RHEL: sudo yum install clang 或 sudo dnf install clang\n")
		fmt.Printf("Arch Linux: sudo pacman -S clang\n")
		fmt.Printf("openSUSE: sudo zypper install clang\n")
	}

	fmt.Printf("\n安装完成后，请重启终端或重新加载环境变量。\n")
}

// needsCGO 判断指定平台是否需要启用CGO
func (b *Builder) needsCGO(platform Platform) bool {
	// 移动平台(Android和iOS)总是需要CGO支持
	// 它们需要与底层系统进行交互，需要外部链接器
	switch platform.OS {
	case "android", "ios":
		return true
	default:
		return false
	}
}

// filterPlatformsByCGO 过滤需要CGO但编译器不可用的平台
func (b *Builder) filterPlatformsByCGO(platforms []Platform) (valid []Platform, skipped []Platform) {
	compilers := b.checkCompilerAvailability()

	for _, platform := range platforms {
		if b.needsCGO(platform) {
			// 需要CGO的平台，检查clang是否可用
			if !compilers["clang"].Available {
				skipped = append(skipped, platform)
				continue
			}
		}
		valid = append(valid, platform)
	}

	return valid, skipped
}

// printCGOStatus 打印CGO编译器状态信息
func (b *Builder) printCGOStatus() {
	compilers := b.checkCompilerAvailability()

	fmt.Printf("%s🔧 CGO 编译器状态:%s\n", Cyan, Reset)

	// 显示各编译器状态
	for name, info := range compilers {
		status := "❌ 不可用"
		if info.Available {
			status = fmt.Sprintf("✅ 可用 (%s)", info.Path)
		}
		fmt.Printf("  %s: %s\n", name, status)
	}

	// 检查需要CGO的平台
	cgoRequiredPlatforms := []Platform{}
	for _, platform := range b.Platforms {
		if b.needsCGO(platform) {
			cgoRequiredPlatforms = append(cgoRequiredPlatforms, platform)
		}
	}

	if len(cgoRequiredPlatforms) > 0 {
		fmt.Printf("\n需要 CGO 支持的平台:\n")
		for _, platform := range cgoRequiredPlatforms {
			status := "❌"
			if compilers["clang"].Available {
				status = "✅"
			}
			fmt.Printf("  %s %s/%s\n", status, platform.OS, platform.Arch)
		}
	}

	fmt.Println()
}

// printVersion 显示版本信息
func printVersion() {
	fmt.Printf("gogogo v2.0.0\n")
	fmt.Printf("跨平台 Go 编译工具\n")
	fmt.Printf("支持 %d 个平台架构的自动化编译\n", len(commonPlatforms))
	fmt.Printf("构建时间: 2025-06-02\n")
	fmt.Printf("Go 版本: %s\n", runtime.Version())
	fmt.Printf("操作系统: %s/%s\n", runtime.GOOS, runtime.GOARCH)

	// 检测Android环境
	if _, err := os.Stat("/data/adb/modules/gogogo"); err == nil {
		fmt.Printf("%s运行于Android环境 (Magisk模块)%s\n", Green+Bold, Reset)
		fmt.Printf("自动配置: GOENV=/data/adb/modules/gogogo/gogogo.env\n")
	}
}

// parsePlatformsAdvanced 高级平台解析，支持组操作
func parsePlatformsAdvanced(platformsStr, includeStr, excludeStr, onlyStr, exceptStr, customStr string) ([]Platform, error) {
	if onlyStr != "" && exceptStr != "" {
		return nil, fmt.Errorf("不能同时使用 -only 和 -except 参数")
	}

	var finalPlatforms PlatformSet = make(PlatformSet)

	// 1. 处理基础平台列表
	if platformsStr != "" {
		basePlatforms, err := parsePlatforms(platformsStr)
		if err != nil {
			return nil, err
		}
		finalPlatforms.AddPlatforms(basePlatforms)
	}

	// 2. 处理 only 操作（仅包含指定组）
	if onlyStr != "" {
		finalPlatforms = make(PlatformSet) // 清空基础列表
		onlyGroups := parseGroupList(onlyStr)
		for _, groupName := range onlyGroups {
			if platforms, exists := platformGroups[groupName]; exists {
				finalPlatforms.AddPlatforms(platforms)
			} else {
				return nil, fmt.Errorf("未知的平台组: %s", groupName)
			}
		}
	}

	// 3. 处理 include 操作（添加组）
	if includeStr != "" {
		includeGroups := parseGroupList(includeStr)
		for _, groupName := range includeGroups {
			if platforms, exists := platformGroups[groupName]; exists {
				finalPlatforms.AddPlatforms(platforms)
			} else {
				return nil, fmt.Errorf("未知的平台组: %s", groupName)
			}
		}
	}

	// 4. 处理 exclude/except 操作（移除组）
	excludeList := append(parseGroupList(excludeStr), parseGroupList(exceptStr)...)
	for _, groupName := range excludeList {
		if platforms, exists := platformGroups[groupName]; exists {
			finalPlatforms.RemovePlatforms(platforms)
		} else {
			return nil, fmt.Errorf("未知的平台组: %s", groupName)
		}
	}

	// 5. 处理自定义组
	if customStr != "" {
		customPlatforms, err := parseCustomGroup(customStr)
		if err != nil {
			return nil, err
		}
		finalPlatforms.AddPlatforms(customPlatforms)
	}

	// 6. 转换为切片
	result := finalPlatforms.ToSlice()
	if len(result) == 0 {
		return nil, fmt.Errorf("解析后没有有效的平台")
	}

	return result, nil
}

// parseGroupList 解析组列表
func parseGroupList(groupsStr string) []string {
	if groupsStr == "" {
		return nil
	}

	var groups []string
	for _, group := range strings.Split(groupsStr, ",") {
		group = strings.TrimSpace(group)
		if group != "" {
			groups = append(groups, group)
		}
	}
	return groups
}

// parseCustomGroup 解析自定义组
func parseCustomGroup(customStr string) ([]Platform, error) {
	// 格式: name:os1/arch1,os2/arch2 或直接 os1/arch1,os2/arch2
	var platformStr string

	if strings.Contains(customStr, ":") {
		parts := strings.SplitN(customStr, ":", 2)
		if len(parts) != 2 {
			return nil, fmt.Errorf("自定义组格式错误，应为 name:platforms 或直接 platforms")
		}
		platformStr = parts[1]
	} else {
		platformStr = customStr
	}

	var platforms []Platform
	for _, part := range strings.Split(platformStr, ",") {
		part = strings.TrimSpace(part)
		if part == "" {
			continue
		}

		osArch := strings.Split(part, "/")
		if len(osArch) != 2 {
			return nil, fmt.Errorf("自定义平台格式错误: %s (应该是 OS/ARCH)", part)
		}

		platforms = append(platforms, Platform{
			OS:   strings.TrimSpace(osArch[0]),
			Arch: strings.TrimSpace(osArch[1]),
		})
	}

	return platforms, nil
}

// PlatformSet 方法
func (ps PlatformSet) AddPlatforms(platforms []Platform) {
	for _, p := range platforms {
		key := p.OS + "/" + p.Arch
		ps[key] = p
	}
}

func (ps PlatformSet) RemovePlatforms(platforms []Platform) {
	for _, p := range platforms {
		key := p.OS + "/" + p.Arch
		delete(ps, key)
	}
}

func (ps PlatformSet) ToSlice() []Platform {
	var result []Platform
	for _, p := range ps {
		result = append(result, p)
	}
	return result
}

func (ps PlatformSet) Contains(platform Platform) bool {
	key := platform.OS + "/" + platform.Arch
	_, exists := ps[key]
	return exists
}

// printGroupCategories 打印平台组分类
func printGroupCategories() {
	fmt.Printf("%s%s平台组分类:%s\n\n", Bold+Cyan, Bold, Reset)

	categories := make(map[string][]string)
	for groupName, metadata := range platformGroupMetadata {
		category := metadata.Category
		if category == "" {
			category = "未分类"
		}
		categories[category] = append(categories[category], groupName)
	}

	categoryOrder := []string{"core", "deployment", "mobile", "web", "embedded", "special", "未分类"}

	for _, category := range categoryOrder {
		if groups, exists := categories[category]; exists {
			fmt.Printf("%s%s%s:%s\n", Bold+Yellow, strings.ToUpper(category), Reset, Reset)
			for _, groupName := range groups {
				if metadata, exists := platformGroupMetadata[groupName]; exists {
					platformCount := len(platformGroups[groupName])
					fmt.Printf("  %-12s %s (%d个平台)\n", groupName, metadata.Description, platformCount)
				}
			}
			fmt.Println()
		}
	}
}

// searchPlatformGroups 搜索包含指定标签的组
func searchPlatformGroups(searchTerm string) {
	fmt.Printf("%s🔍 搜索结果: \"%s\"%s\n\n", Bold+Cyan, searchTerm, Reset)

	found := false
	searchTerm = strings.ToLower(searchTerm)

	for groupName, metadata := range platformGroupMetadata {
		match := false

		// 搜索组名
		if strings.Contains(strings.ToLower(groupName), searchTerm) {
			match = true
		}

		// 搜索描述
		if strings.Contains(strings.ToLower(metadata.Description), searchTerm) {
			match = true
		}

		// 搜索标签
		for _, tag := range metadata.Tags {
			if strings.Contains(strings.ToLower(tag), searchTerm) {
				match = true
				break
			}
		}

		if match {
			found = true
			platformCount := len(platformGroups[groupName])
			fmt.Printf("%s%s%s: %s (%d个平台)\n", Bold+Green, groupName, Reset, metadata.Description, platformCount)
			fmt.Printf("  分类: %s\n", metadata.Category)
			fmt.Printf("  标签: %s\n", strings.Join(metadata.Tags, ", "))
			fmt.Println()
		}
	}

	if !found {
		fmt.Printf("%s未找到匹配的平台组%s\n", Yellow, Reset)
	}
}

// validatePlatformGroups 验证平台组配置
func validatePlatformGroups() {
	fmt.Printf("%s🔍 验证平台组配置...%s\n\n", Bold+Cyan, Reset)

	totalGroups := len(platformGroups)
	validGroups := 0
	issues := []string{}

	for groupName, platforms := range platformGroups {
		// 检查是否有重复平台
		seen := make(map[string]bool)
		duplicates := []string{}

		for _, platform := range platforms {
			key := platform.OS + "/" + platform.Arch
			if seen[key] {
				duplicates = append(duplicates, key)
			}
			seen[key] = true
		}

		if len(duplicates) > 0 {
			issues = append(issues, fmt.Sprintf("组 '%s' 包含重复平台: %s", groupName, strings.Join(duplicates, ", ")))
		}

		// 检查是否有空组
		if len(platforms) == 0 {
			issues = append(issues, fmt.Sprintf("组 '%s' 为空", groupName))
		} else {
			validGroups++
		}

		// 检查元数据是否存在
		if _, exists := platformGroupMetadata[groupName]; !exists {
			issues = append(issues, fmt.Sprintf("组 '%s' 缺少元数据", groupName))
		}
	}

	// 输出验证结果
	fmt.Printf("%s验证结果:%s\n", Bold+Green, Reset)
	fmt.Printf("  总组数: %d\n", totalGroups)
	fmt.Printf("  有效组: %d\n", validGroups)
	fmt.Printf("  问题数: %d\n", len(issues))

	if len(issues) > 0 {
		fmt.Printf("\n%s发现的问题:%s\n", Bold+Red, Reset)
		for i, issue := range issues {
			fmt.Printf("  %d. %s\n", i+1, issue)
		}
	} else {
		fmt.Printf("\n%s✅ 所有平台组配置有效%s\n", Green, Reset)
	}
}

// printGroupInfo 显示指定组的详细信息
func printGroupInfo(groupName string) {
	if platforms, exists := platformGroups[groupName]; !exists {
		printError(fmt.Sprintf("未找到平台组: %s", groupName))
		return
	} else {
		fmt.Printf("%s%s组信息: %s%s\n\n", Bold+Cyan, Bold, groupName, Reset)

		// 显示元数据
		if metadata, exists := platformGroupMetadata[groupName]; exists {
			fmt.Printf("%s描述:%s %s\n", Bold+Yellow, Reset, metadata.Description)
			fmt.Printf("%s分类:%s %s\n", Bold+Yellow, Reset, metadata.Category)
			fmt.Printf("%s标签:%s %s\n", Bold+Yellow, Reset, strings.Join(metadata.Tags, ", "))
		}

		fmt.Printf("%s平台数量:%s %d\n", Bold+Yellow, Reset, len(platforms))
		fmt.Printf("%s包含平台:%s\n", Bold+Yellow, Reset)

		// 按操作系统分组显示
		osGroups := make(map[string][]Platform)
		for _, platform := range platforms {
			osGroups[platform.OS] = append(osGroups[platform.OS], platform)
		}

		for os, osPlatforms := range osGroups {
			var archs []string
			for _, p := range osPlatforms {
				archs = append(archs, p.Arch)
			}
			fmt.Printf("  %s: %s\n", os, strings.Join(archs, ", "))
		}

		fmt.Println()
	}
}

// SaveGroupConfiguration 保存组配置到文件
func (b *Builder) SaveGroupConfiguration(filename string) error {
	config := map[string]interface{}{
		"include_groups": b.IncludeGroups,
		"exclude_groups": b.ExcludeGroups,
		"custom_groups":  b.CustomGroups,
		"platforms":      b.Platforms,
		"timestamp":      time.Now().Format(time.RFC3339),
		"version":        "2.0.0",
	}

	data, err := json.MarshalIndent(config, "", "  ")
	if err != nil {
		return err
	}

	return os.WriteFile(filename, data, 0644)
}

// LoadGroupConfiguration 从文件加载组配置
func (b *Builder) LoadGroupConfiguration(filename string) error {
	data, err := os.ReadFile(filename)
	if err != nil {
		return err
	}

	var config map[string]interface{}
	if err := json.Unmarshal(data, &config); err != nil {
		return err
	}

	// 加载配置
	if includeGroups, ok := config["include_groups"].([]interface{}); ok {
		b.IncludeGroups = make([]string, len(includeGroups))
		for i, group := range includeGroups {
			b.IncludeGroups[i] = group.(string)
		}
	}

	if excludeGroups, ok := config["exclude_groups"].([]interface{}); ok {
		b.ExcludeGroups = make([]string, len(excludeGroups))
		for i, group := range excludeGroups {
			b.ExcludeGroups[i] = group.(string)
		}
	}

	return nil
}

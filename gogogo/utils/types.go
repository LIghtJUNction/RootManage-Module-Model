package utils

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

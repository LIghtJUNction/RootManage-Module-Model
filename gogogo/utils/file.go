package utils

import (
	"log/slog"
	"os"
	"path/filepath"
	"strings"

	"github.com/klauspost/compress/gzip"
	"github.com/lightjunction/rootmanager-module-model/gogogo/config"
)

// CompressFile 压缩文件
func CompressFile(filePath string) error {
	// 读取原文件
	input, err := os.ReadFile(filePath)
	if err != nil {
		return err
	}

	// 创建压缩文件
	compressedPath := filePath + ".gz"
	output, err := os.Create(compressedPath)
	if err != nil {
		return err
	}
	defer output.Close()

	// 使用gzip压缩
	writer := gzip.NewWriter(output)
	defer writer.Close()

	_, err = writer.Write(input)
	if err != nil {
		return err
	}

	// 删除原文件
	os.Remove(filePath)

	return nil
}

// CleanOutputDir 清理输出目录
func CleanOutputDir(outputDir string, verbose int, logger *slog.Logger) error {
	if _, err := os.Stat(outputDir); err == nil {
		if verbose >= 1 {
			// 获取颜色函数
			_, _, _, _, colorInfo, _ := config.GetColors()
			colorInfo.Printf("🧹 清理输出目录: %s\n", outputDir)
		}
		return os.RemoveAll(outputDir)
	}
	return nil
}

// GetBinaryNameFromSource 从源文件路径中提取二进制文件名
func GetBinaryNameFromSource(sourceFile string) string {
	// 获取文件名（不含路径）
	filename := filepath.Base(sourceFile)

	// 移除扩展名
	if ext := filepath.Ext(filename); ext != "" {
		filename = strings.TrimSuffix(filename, ext)
	}

	return filename
}

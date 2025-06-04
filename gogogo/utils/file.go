package utils

import (
	"log/slog"
	"os"

	"github.com/fatih/color"
	"github.com/klauspost/compress/gzip"
)

var (
	colorInfoFile = color.New(color.FgBlue)
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
			colorInfoFile.Printf("🧹 清理输出目录: %s\n", outputDir)
		}
		return os.RemoveAll(outputDir)
	}
	return nil
}

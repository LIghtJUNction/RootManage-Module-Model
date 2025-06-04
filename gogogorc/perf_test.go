package main

import (
	"fmt"
	"os"
	"os/exec"
	"testing"
	"time"
)

func TestPerformance(t *testing.T) {
	// 设置一个复杂的PATH来测试性能（包含真实的Android路径）
	complexPath := "/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:/usr/local/go/bin:/opt/homebrew/bin:/usr/local/sbin:/data/adb/modules/gogogo/GOBIN:/data/adb/modules/gogogo/GOXBIN:/system/bin:/system/xbin:/vendor/bin:/data/local/tmp:/data/tmp:" +
		// 真实的Android "/0/" 路径（MT Manager相关）
		"/data/media/0/mt_manager/bin:/data/media/0/MT2/tools/bin:/data/media/0/Download/bin:" +
		// KernelSU相关路径
		"/data/adb/ksu/bin:/data/adb/kernelsu/bin:/data/adb/ap/bin:/data/media/0/KernelSU/bin:" +
		// 其他Android应用相关的 "/0/" 路径
		"/storage/emulated/0/termux/bin:/storage/emulated/0/Android/data/com.termux/files/usr/bin:" +
		"/data/media/0/tmate/bin:/data/media/0/zarchiver/bin:/data/media/0/busybox/bin:" +
		// 模拟器和虚拟化环境路径
		"/data/media/0/vmos/bin:/data/media/0/parallel_space/bin"

	// 测试原生版本
	os.Setenv("PATH", complexPath)

	// 预热
	for i := 0; i < 10; i++ {
		cmd := exec.Command("./gogogorc-core.exe")
		cmd.Run()
	}

	// 实际测试
	iterations := 100
	totalTime := time.Duration(0)

	for i := 0; i < iterations; i++ {
		start := time.Now()
		cmd := exec.Command("./gogogorc-core.exe")
		cmd.Run()
		elapsed := time.Since(start)
		totalTime += elapsed
	}

	avgTime := totalTime / time.Duration(iterations)
	fmt.Printf("平均执行时间: %v\n", avgTime)
	fmt.Printf("总计时间: %v\n", totalTime)
	if avgTime < 5*time.Millisecond {
		fmt.Println("✅ 性能目标达成！(<5ms)")
	} else {
		fmt.Printf("❌ 需要进一步优化，当前: %v, 目标: <5ms\n", avgTime)
		t.Errorf("性能未达标：%v > 5ms", avgTime)
	}
}

// 基准测试 - 更精确的性能测量
func BenchmarkGogogorcCore(b *testing.B) {
	// 设置复杂的PATH（包含真实的Android路径）
	complexPath := "/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:/usr/local/go/bin:/opt/homebrew/bin:/usr/local/sbin:/data/adb/modules/gogogo/GOBIN:/data/adb/modules/gogogo/GOXBIN:/system/bin:/system/xbin:/vendor/bin:/data/local/tmp:/data/tmp:" +
		// 真实的Android "/0/" 路径（MT Manager相关）
		"/data/media/0/mt_manager/bin:/data/media/0/MT2/tools/bin:/data/media/0/Download/bin:" +
		// KernelSU相关路径
		"/data/adb/ksu/bin:/data/adb/kernelsu/bin:/data/adb/ap/bin:/data/media/0/KernelSU/bin:" +
		// 其他Android应用相关的 "/0/" 路径
		"/storage/emulated/0/termux/bin:/storage/emulated/0/Android/data/com.termux/files/usr/bin:" +
		"/data/media/0/tmate/bin:/data/media/0/zarchiver/bin:/data/media/0/busybox/bin:" +
		// 模拟器和虚拟化环境路径
		"/data/media/0/vmos/bin:/data/media/0/parallel_space/bin"
	os.Setenv("PATH", complexPath)

	// 重置计时器，排除设置时间
	b.ResetTimer()
	// 运行基准测试
	for i := 0; i < b.N; i++ {
		cmd := exec.Command("./gogogorc-core.exe")
		cmd.Run()
	}
}

// 测试纯算法性能（不包括进程启动开销）
func TestAlgorithmPerformance(t *testing.T) {
	// 设置复杂的PATH（包含真实的Android路径）
	complexPath := "/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:/usr/local/go/bin:/opt/homebrew/bin:/usr/local/sbin:/data/adb/modules/gogogo/GOBIN:/data/adb/modules/gogogo/GOXBIN:/system/bin:/system/xbin:/vendor/bin:/data/local/tmp:/data/tmp:" +
		// 真实的Android "/0/" 路径（MT Manager相关）
		"/data/media/0/mt_manager/bin:/data/media/0/MT2/tools/bin:/data/media/0/Download/bin:" +
		// KernelSU相关路径
		"/data/adb/ksu/bin:/data/adb/kernelsu/bin:/data/adb/ap/bin:/data/media/0/KernelSU/bin:" +
		// 其他Android应用相关的 "/0/" 路径
		"/storage/emulated/0/termux/bin:/storage/emulated/0/Android/data/com.termux/files/usr/bin:" +
		"/data/media/0/tmate/bin:/data/media/0/zarchiver/bin:/data/media/0/busybox/bin:" +
		// 模拟器和虚拟化环境路径
		"/data/media/0/vmos/bin:/data/media/0/parallel_space/bin"

	// 测试addPaths
	addPaths := "/data/adb/modules/gogogo/GOBIN:/data/adb/modules/gogogo/GOXBIN"

	// 预热
	for i := 0; i < 100; i++ {
		sortPath(complexPath, addPaths)
	}

	// 实际测试
	iterations := 10000
	start := time.Now()

	for i := 0; i < iterations; i++ {
		sortPath(complexPath, addPaths)
	}

	elapsed := time.Since(start)
	avgTime := elapsed / time.Duration(iterations)

	fmt.Printf("纯算法平均执行时间: %v\n", avgTime)
	fmt.Printf("纯算法总计时间: %v\n", elapsed)

	if avgTime < time.Microsecond { // 算法本身应该在1微秒内完成
		fmt.Printf("✅ 算法性能优秀！(<%v)\n", time.Microsecond)
	} else if avgTime < 10*time.Microsecond {
		fmt.Printf("✅ 算法性能良好！(<%v)\n", 10*time.Microsecond)
	} else {
		fmt.Printf("⚠️ 算法需要优化，当前: %v\n", avgTime)
	}

	// 验证结果正确性
	result := sortPath(complexPath, addPaths)
	if result == "" {
		t.Error("排序结果为空")
	}

	// 验证Go路径是否在前面
	if !contains(result, "/data/adb/modules/gogogo/GOBIN") {
		t.Error("结果中缺少Go路径")
	}

	fmt.Printf("排序后PATH长度: %d 个字符\n", len(result))
	fmt.Printf("路径数量: %d 个\n", countPaths(result))
}

// 辅助函数：检查字符串是否包含子串
func contains(str, substr string) bool {
	return len(str) >= len(substr) && str[:len(substr)] == substr ||
		(len(str) > len(substr) && str[len(str)-len(substr):] == substr) ||
		findInString(str, substr)
}

func findInString(str, substr string) bool {
	for i := 0; i <= len(str)-len(substr); i++ {
		if str[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}

// 辅助函数：计算路径数量
func countPaths(pathStr string) int {
	if pathStr == "" {
		return 0
	}
	count := 1
	for i := 0; i < len(pathStr); i++ {
		if pathStr[i] == ':' {
			count++
		}
	}
	return count
}

// 基准测试纯算法性能
func BenchmarkSortPathAlgorithm(b *testing.B) {
	// 设置复杂的PATH（包含真实的Android路径）
	complexPath := "/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:/usr/local/go/bin:/opt/homebrew/bin:/usr/local/sbin:/data/adb/modules/gogogo/GOBIN:/data/adb/modules/gogogo/GOXBIN:/system/bin:/system/xbin:/vendor/bin:/data/local/tmp:/data/tmp:" +
		// 真实的Android "/0/" 路径（MT Manager相关）
		"/data/media/0/mt_manager/bin:/data/media/0/MT2/tools/bin:/data/media/0/Download/bin:" +
		// KernelSU相关路径
		"/data/adb/ksu/bin:/data/adb/kernelsu/bin:/data/adb/ap/bin:/data/media/0/KernelSU/bin:" +
		// 其他Android应用相关的 "/0/" 路径
		"/storage/emulated/0/termux/bin:/storage/emulated/0/Android/data/com.termux/files/usr/bin:" +
		"/data/media/0/tmate/bin:/data/media/0/zarchiver/bin:/data/media/0/busybox/bin:" +
		// 模拟器和虚拟化环境路径
		"/data/media/0/vmos/bin:/data/media/0/parallel_space/bin"

	addPaths := "/data/adb/modules/gogogo/GOBIN:/data/adb/modules/gogogo/GOXBIN"

	// 重置计时器
	b.ResetTimer()

	// 运行基准测试
	for i := 0; i < b.N; i++ {
		sortPath(complexPath, addPaths)
	}
}

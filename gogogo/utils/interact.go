package utils

import (
	"bufio"
	"os"
	"strings"

	"github.com/lightjunction/rootmanager-module-model/gogogo/config"
)

// AskUserConfirm 询问用户确认
func AskUserConfirm(prompt string, noPrompt bool) bool {
	if noPrompt {
		return true
	}

	// 获取颜色函数
	_, _, _, colorWarning, _, _ := config.GetColors()

	colorWarning.Printf("%s (y/N): ", prompt)
	scanner := bufio.NewScanner(os.Stdin)
	if scanner.Scan() {
		response := strings.ToLower(strings.TrimSpace(scanner.Text()))
		return response == "y" || response == "yes"
	}
	return false
}

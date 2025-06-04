package utils

import (
	"bufio"
	"os"
	"strings"

	"github.com/fatih/color"
)

var (
	colorWarningInteract = color.New(color.FgYellow, color.Bold)
)

// AskUserConfirm 询问用户确认
func AskUserConfirm(prompt string, noPrompt bool) bool {
	if noPrompt {
		return true
	}

	colorWarningInteract.Printf("%s (y/N): ", prompt)
	scanner := bufio.NewScanner(os.Stdin)
	if scanner.Scan() {
		response := strings.ToLower(strings.TrimSpace(scanner.Text()))
		return response == "y" || response == "yes"
	}
	return false
}

package utils

import "github.com/fatih/color"

// Color variables used across the utils package
var (
	ColorError   = color.New(color.FgRed, color.Bold)
	ColorWarning = color.New(color.FgYellow, color.Bold)
	ColorInfo    = color.New(color.FgBlue)
	ColorSuccess = color.New(color.FgGreen, color.Bold)
)

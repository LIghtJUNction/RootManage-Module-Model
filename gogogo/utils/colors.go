package utils

import "github.com/fatih/color"

// Color variables used across the utils package
var (
	ColorError     = color.New(color.FgRed, color.Bold)
	ColorWarning   = color.New(color.FgYellow, color.Bold)
	ColorInfo      = color.New(color.FgBlue)
	ColorSuccess   = color.New(color.FgGreen, color.Bold)
	ColorDebug     = color.New(color.FgMagenta)
	ColorHighlight = color.New(color.FgCyan, color.Bold)
	ColorSubtle    = color.New(color.FgHiBlack)
	ColorBright    = color.New(color.FgHiWhite, color.Bold)
	ColorCommand   = color.New(color.FgHiBlue, color.Bold)
	ColorPath      = color.New(color.FgHiCyan)
	ColorSize      = color.New(color.FgHiGreen)
	ColorPlatform  = color.New(color.FgHiMagenta, color.Bold)
)

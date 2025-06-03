package main

import (
	"fmt"
	"runtime"
)

func main() {
	fmt.Printf("Hello from gogogo!\n")
	fmt.Printf("Running on %s/%s\n", runtime.GOOS, runtime.GOARCH)
	fmt.Printf("Go version: %s\n", runtime.Version())
}

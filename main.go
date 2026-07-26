package main

import (
	"os"

	"docup/src"
)

func main() {
	os.Exit(src.Run(os.Args[1:]))
}

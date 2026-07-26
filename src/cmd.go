package src

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"
)

const (
	colorReset  = "\033[0m"
	colorRed    = "\033[31m"
	colorGreen  = "\033[32m"
	colorYellow = "\033[33m"
	colorBlue   = "\033[34m"
	colorCyan   = "\033[36m"
	colorBold   = "\033[1m"
)

// Run is the entry point forwarded from main.go.
func Run(args []string) int {
	if len(args) < 1 {
		printUsage()
		return 1
	}

	switch args[0] {
	case "build":
		return runBuild(args[1:])
	case "help", "-h", "--help":
		printUsage()
		return 0
	default:
		fmt.Fprintf(os.Stderr, "%s✗%s unknown command %q\n\n", colorRed, colorReset, args[0])
		printUsage()
		return 1
	}
}

func printUsage() {
	fmt.Printf("%s%sDocUP Compiler%s — .du → .html\n\n", colorBold, colorCyan, colorReset)
	fmt.Printf("  %sUsage:%s docup build <input.du> -o <output.html>\n", colorBold, colorReset)
}

func runBuild(args []string) int {
	var inputPath, outputPath string
	for i := 0; i < len(args); i++ {
		switch args[i] {
		case "-o":
			if i+1 >= len(args) || strings.HasPrefix(args[i+1], "-") {
				fail("missing value for -o")
				return 1
			}
			outputPath = args[i+1]
			i++
		default:
			if strings.HasPrefix(args[i], "-") {
				fail(fmt.Sprintf("unknown flag %q", args[i]))
				return 1
			}
			if inputPath != "" {
				fail(fmt.Sprintf("unexpected extra argument %q", args[i]))
				return 1
			}
			inputPath = args[i]
		}
	}

	if inputPath == "" {
		fail("no input .du file specified")
		printUsage()
		return 1
	}
	if ext := filepath.Ext(inputPath); ext != ".du" {
		warn(fmt.Sprintf("input file %q does not have a .du extension", inputPath))
	}
	if outputPath == "" {
		outputPath = swapExt(inputPath, ".html")
	}

	start := time.Now()

	step(1, 5, "Reading", inputPath)
	source, err := os.ReadFile(inputPath)
	if err != nil {
		reportFileError(inputPath, err)
		return 1
	}
	if len(strings.TrimSpace(string(source))) == 0 {
		fail(fmt.Sprintf("%s is empty", inputPath))
		return 1
	}

	step(2, 5, "Lexing & parsing", inputPath)
	parser, err := NewParser(source)
	if err != nil {
		reportCompileError(inputPath, source, err)
		return 1
	}
	doc, err := parser.ParseDocument()
	if err != nil {
		reportCompileError(inputPath, source, err)
		return 1
	}

	step(3, 5, "Analyzing", "semantic checks")
	if err := Analyze(doc); err != nil {
		reportCompileError(inputPath, source, err)
		return 1
	}

	step(4, 5, "Generating", "HTML5 output")
	out := Generate(doc)

	step(5, 5, "Writing", outputPath)
	if err := os.WriteFile(outputPath, []byte(out), 0644); err != nil {
		reportFileError(outputPath, err)
		return 1
	}

	elapsed := time.Since(start)
	fmt.Printf("\n%s%s✓ build succeeded%s in %s%.2fms%s\n", colorBold, colorGreen, colorReset, colorCyan, float64(elapsed.Microseconds())/1000.0, colorReset)
	fmt.Printf("  %s→%s %s%s%s\n", colorGreen, colorReset, colorBold, outputPath, colorReset)
	return 0
}

func step(n, total int, verb, detail string) {
	fmt.Printf("%s[%d/%d]%s %s%s%s %s\n", colorBlue, n, total, colorReset, colorBold, verb, colorReset, detail)
}

func fail(msg string) {
	fmt.Fprintf(os.Stderr, "%s✗ error:%s %s\n", colorRed, colorReset, msg)
}

func warn(msg string) {
	fmt.Fprintf(os.Stderr, "%s⚠ warning:%s %s\n", colorYellow, colorReset, msg)
}

// reportFileError turns an os error (not found, permission denied, is a
// directory, etc.) into a clear, specific message instead of raw Go
// error text.
func reportFileError(path string, err error) {
	switch {
	case errors.Is(err, os.ErrNotExist):
		fail(fmt.Sprintf("file not found: %s", path))
	case errors.Is(err, os.ErrPermission):
		fail(fmt.Sprintf("permission denied: %s", path))
	default:
		fail(fmt.Sprintf("cannot access %s: %v", path, err))
	}
}

// reportCompileError prints a lex/parse/sema error together with a
// caret-annotated snippet of the offending source line whenever the error
// carries a position; otherwise it falls back to the plain message.
func reportCompileError(path string, source []byte, err error) {
	var perr PositionedError
	if !errors.As(err, &perr) {
		fail(err.Error())
		return
	}
	line, col := perr.Position()
	fmt.Fprintf(os.Stderr, "%s✗ error:%s %s\n", colorRed, colorReset, err.Error())
	fmt.Fprintf(os.Stderr, "  %s--> %s:%d:%d%s\n", colorCyan, path, line, col, colorReset)
	if snippet := SourceSnippet(source, line, col); snippet != "" {
		for _, l := range strings.Split(snippet, "\n") {
			fmt.Fprintf(os.Stderr, "  %s%s%s\n", colorYellow, l, colorReset)
		}
	}
}

func swapExt(path, newExt string) string {
	ext := filepath.Ext(path)
	return path[:len(path)-len(ext)] + newExt
}

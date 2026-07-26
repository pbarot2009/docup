package src

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"
)

// version is the DocUP compiler's own version, reported by `docup version`
// and `docup --version`. Bump this when the compiler's behavior changes.
const version = "0.2.0"

var (
	colorReset  = "\033[0m"
	colorRed    = "\033[31m"
	colorGreen  = "\033[32m"
	colorYellow = "\033[33m"
	colorBlue   = "\033[34m"
	colorCyan   = "\033[36m"
	colorBold   = "\033[1m"
	colorDim    = "\033[2m"
)

func init() {
	// Disable ANSI color when explicitly requested (NO_COLOR convention)
	// or when stdout isn't a terminal (e.g. piped into a file or another
	// program), so build logs and redirected output stay clean plain text.
	if _, set := os.LookupEnv("NO_COLOR"); set || !isTerminal(os.Stdout) {
		disableColor()
	}
}

func disableColor() {
	colorReset, colorRed, colorGreen, colorYellow = "", "", "", ""
	colorBlue, colorCyan, colorBold, colorDim = "", "", "", ""
}

func isTerminal(f *os.File) bool {
	info, err := f.Stat()
	if err != nil {
		return false
	}
	return (info.Mode() & os.ModeCharDevice) != 0
}

// options holds parsed global/build flags.
type options struct {
	inputPath  string
	outputPath string
	verbose    bool
	quiet      bool
}

// Run is the entry point forwarded from main.go.
func Run(args []string) int {
	if len(args) < 1 {
		printUsage()
		return 1
	}

	switch args[0] {
	case "build":
		return runBuild(args[1:])
	case "version", "-v", "--version":
		fmt.Printf("docup version %s\n", version)
		return 0
	case "help", "-h", "--help":
		printUsage()
		return 0
	default:
		fail(fmt.Sprintf("unknown command %q", args[0]))
		fmt.Println()
		printUsage()
		return 1
	}
}

func printUsage() {
	fmt.Printf("%s%sDocUP Compiler%s %s— compiles .du documents to standalone HTML5%s\n\n", colorBold, colorCyan, colorReset, colorDim, colorReset)
	fmt.Printf("%sUSAGE%s\n", colorBold, colorReset)
	fmt.Printf("  docup build <input.du> [flags]\n")
	fmt.Printf("  docup version\n")
	fmt.Printf("  docup help\n\n")
	fmt.Printf("%sFLAGS%s\n", colorBold, colorReset)
	fmt.Printf("  -o <file>       output HTML path (default: input with .html extension)\n")
	fmt.Printf("  --verbose, -V   print detailed timing for each build stage\n")
	fmt.Printf("  --quiet, -q     suppress step-by-step progress output\n\n")
	fmt.Printf("%sEXAMPLES%s\n", colorBold, colorReset)
	fmt.Printf("  docup build report.du\n")
	fmt.Printf("  docup build report.du -o dist/report.html\n")
	fmt.Printf("  docup build report.du --quiet\n")
}

func runBuild(args []string) int {
	opts, ok := parseBuildArgs(args)
	if !ok {
		return 1
	}
	if opts.inputPath == "" {
		fail("no input .du file specified")
		fmt.Println()
		printUsage()
		return 1
	}
	if ext := filepath.Ext(opts.inputPath); !strings.EqualFold(ext, ".du") {
		warn(fmt.Sprintf("input file %q does not have a .du extension", opts.inputPath))
	}
	if opts.outputPath == "" {
		opts.outputPath = swapExt(opts.inputPath, ".html")
	}
	return build(opts)
}

// parseBuildArgs parses flags for the "build" subcommand. It returns
// ok=false after already printing an error for any malformed invocation.
func parseBuildArgs(args []string) (options, bool) {
	var opts options
	for i := 0; i < len(args); i++ {
		arg := args[i]
		switch arg {
		case "-o", "--output":
			if i+1 >= len(args) || looksLikeFlag(args[i+1]) {
				fail(fmt.Sprintf("missing value for %s", arg))
				return opts, false
			}
			opts.outputPath = args[i+1]
			i++
		case "--verbose", "-V":
			opts.verbose = true
		case "--quiet", "-q":
			opts.quiet = true
		default:
			if looksLikeFlag(arg) {
				fail(fmt.Sprintf("unknown flag %q", arg))
				return opts, false
			}
			if opts.inputPath != "" {
				fail(fmt.Sprintf("unexpected extra argument %q (input file already set to %q)", arg, opts.inputPath))
				return opts, false
			}
			opts.inputPath = arg
		}
	}
	if opts.verbose && opts.quiet {
		fail("--verbose and --quiet cannot be used together")
		return opts, false
	}
	return opts, true
}

func looksLikeFlag(s string) bool {
	return strings.HasPrefix(s, "-") && s != "-"
}

// build runs the full read -> lex/parse -> analyze -> generate -> write
// pipeline, reporting progress and timing according to opts.
func build(opts options) int {
	overallStart := time.Now()
	stageStart := time.Now()
	total := 5

	logStage := func(n int, verb, detail string) {
		if opts.quiet {
			return
		}
		if opts.verbose && n > 1 {
			fmt.Printf("%s      (%.2fms)%s\n", colorDim, msSince(stageStart), colorReset)
		}
		stageStart = time.Now()
		step(n, total, verb, detail)
	}

	logStage(1, "Reading", opts.inputPath)
	source, err := os.ReadFile(opts.inputPath)
	if err != nil {
		reportFileError(opts.inputPath, err)
		return 1
	}
	if len(strings.TrimSpace(string(source))) == 0 {
		fail(fmt.Sprintf("%s is empty", opts.inputPath))
		return 1
	}

	logStage(2, "Lexing & parsing", opts.inputPath)
	parser, err := NewParser(source)
	if err != nil {
		reportCompileError(opts.inputPath, source, err)
		return 1
	}
	doc, err := parser.ParseDocument()
	if err != nil {
		reportCompileError(opts.inputPath, source, err)
		return 1
	}

	logStage(3, "Analyzing", "semantic checks")
	if err := Analyze(doc); err != nil {
		reportCompileError(opts.inputPath, source, err)
		return 1
	}

	logStage(4, "Generating", "HTML5 output")
	out := Generate(doc)

	logStage(5, "Writing", opts.outputPath)
	if err := os.WriteFile(opts.outputPath, []byte(out), 0644); err != nil {
		reportFileError(opts.outputPath, err)
		return 1
	}
	if opts.verbose && !opts.quiet {
		fmt.Printf("%s      (%.2fms)%s\n", colorDim, msSince(stageStart), colorReset)
	}

	if !opts.quiet {
		elapsed := time.Since(overallStart)
		size := len(out)
		fmt.Printf("\n%s%s✓ build succeeded%s in %s%.2fms%s\n", colorBold, colorGreen, colorReset, colorCyan, float64(elapsed.Microseconds())/1000.0, colorReset)
		fmt.Printf("  %s→%s %s%s%s %s(%s)%s\n", colorGreen, colorReset, colorBold, opts.outputPath, colorReset, colorDim, humanSize(size), colorReset)
	}
	return 0
}

func msSince(t time.Time) float64 {
	return float64(time.Since(t).Microseconds()) / 1000.0
}

func humanSize(bytes int) string {
	const unit = 1024
	if bytes < unit {
		return fmt.Sprintf("%d B", bytes)
	}
	div, exp := int64(unit), 0
	for n := bytes / unit; n >= unit; n /= unit {
		div *= unit
		exp++
	}
	return fmt.Sprintf("%.1f %cB", float64(bytes)/float64(div), "KMGTPE"[exp])
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

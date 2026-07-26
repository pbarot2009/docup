package src

import (
	"fmt"
	"strings"
)

// PositionedError is implemented by any compiler-stage error that can
// point at a specific line/column in the source.
type PositionedError interface {
	error
	Position() (line, col int)
}

func (e *SemaError) Position() (int, int) { return e.Line, e.Col }

// LexError and ParseError give lexing/parsing failures the same
// line/column reporting contract as SemaError, so cmd.go can render a
// single consistent diagnostic for any compiler stage.
type LexError struct {
	Line, Col int
	Message   string
}

func (e *LexError) Error() string {
	return fmt.Sprintf("lex error at %d:%d: %s", e.Line, e.Col, e.Message)
}
func (e *LexError) Position() (int, int) { return e.Line, e.Col }

type ParseError struct {
	Line, Col int
	Message   string
}

func (e *ParseError) Error() string {
	return fmt.Sprintf("parse error at %d:%d: %s", e.Line, e.Col, e.Message)
}
func (e *ParseError) Position() (int, int) { return e.Line, e.Col }

// SourceSnippet renders the offending line of source with a caret ("^")
// pointing at the exact column, for use in diagnostic output.
func SourceSnippet(src []byte, line, col int) string {
	lines := strings.Split(string(src), "\n")
	if line < 1 || line > len(lines) {
		return ""
	}
	text := lines[line-1]
	if col < 1 {
		col = 1
	}
	caretPos := col - 1
	if caretPos > len(text) {
		caretPos = len(text)
	}
	gutter := fmt.Sprintf("%d | ", line)
	pad := strings.Repeat(" ", len(gutter)+caretPos)
	return fmt.Sprintf("%s%s\n%s^", gutter, text, pad)
}

package src

import (
	"bytes"
	"fmt"
)

type TokenType int

const (
	TokEOF TokenType = iota
	TokIdent
	TokString
	TokLParen
	TokRParen
	TokLBrace
	TokRBrace
	TokComma
	TokColon
	TokRawScopeOpen // {!
	TokNumber       // bare integer literal, e.g. heading level
)

type Token struct {
	Type       TokenType
	Value      string
	Line, Col  int
}

type Lexer struct {
	src        []byte
	pos        int
	line, col  int
}

func NewLexer(src []byte) *Lexer {
	return &Lexer{src: src, pos: 0, line: 1, col: 1}
}

func (l *Lexer) errorf(format string, args ...interface{}) error {
	return &LexError{Line: l.line, Col: l.col, Message: fmt.Sprintf(format, args...)}
}

func (l *Lexer) peek() byte {
	if l.pos >= len(l.src) {
		return 0
	}
	return l.src[l.pos]
}

func (l *Lexer) peekAt(offset int) byte {
	if l.pos+offset >= len(l.src) {
		return 0
	}
	return l.src[l.pos+offset]
}

func (l *Lexer) advance() byte {
	c := l.src[l.pos]
	l.pos++
	if c == '\n' {
		l.line++
		l.col = 1
	} else {
		l.col++
	}
	return c
}

func (l *Lexer) skipWhitespaceAndComments() error {
	for l.pos < len(l.src) {
		c := l.peek()
		switch {
		case c == ' ' || c == '\t' || c == '\r' || c == '\n':
			l.advance()
		case c == '/' && l.peekAt(1) == '/':
			for l.pos < len(l.src) && l.peek() != '\n' {
				l.advance()
			}
		case c == '/' && l.peekAt(1) == '*':
			startLine, startCol := l.line, l.col
			l.advance()
			l.advance()
			closed := false
			for l.pos < len(l.src) {
				if l.peek() == '*' && l.peekAt(1) == '/' {
					l.advance()
					l.advance()
					closed = true
					break
				}
				l.advance()
			}
			if !closed {
				return &LexError{Line: startLine, Col: startCol, Message: "unterminated block comment"}
			}
		default:
			return nil
		}
	}
	return nil
}

func isIdentStart(c byte) bool {
	return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}

func isIdentCont(c byte) bool {
	return isIdentStart(c) || (c >= '0' && c <= '9')
}

// NextToken returns the next structural token, skipping whitespace/comments.
// Used for meta blocks, headers, attribute lists, and structural scanning.
func (l *Lexer) NextToken() (Token, error) {
	if err := l.skipWhitespaceAndComments(); err != nil {
		return Token{}, err
	}
	if l.pos >= len(l.src) {
		return Token{Type: TokEOF, Line: l.line, Col: l.col}, nil
	}

	startLine, startCol := l.line, l.col
	c := l.peek()

	switch {
	case c == '(':
		l.advance()
		return Token{Type: TokLParen, Value: "(", Line: startLine, Col: startCol}, nil
	case c == ')':
		l.advance()
		return Token{Type: TokRParen, Value: ")", Line: startLine, Col: startCol}, nil
	case c == '{':
		if l.peekAt(1) == '!' {
			l.advance()
			l.advance()
			return Token{Type: TokRawScopeOpen, Value: "{!", Line: startLine, Col: startCol}, nil
		}
		l.advance()
		return Token{Type: TokLBrace, Value: "{", Line: startLine, Col: startCol}, nil
	case c == '}':
		l.advance()
		return Token{Type: TokRBrace, Value: "}", Line: startLine, Col: startCol}, nil
	case c == ',':
		l.advance()
		return Token{Type: TokComma, Value: ",", Line: startLine, Col: startCol}, nil
	case c == ':':
		l.advance()
		return Token{Type: TokColon, Value: ":", Line: startLine, Col: startCol}, nil
	case c == '"':
		return l.readString()
	case isIdentStart(c):
		return l.readIdent()
	case c >= '0' && c <= '9':
		return l.readNumber()
	default:
		return Token{}, l.errorf("unexpected character %q", c)
	}
}

func (l *Lexer) readNumber() (Token, error) {
	startLine, startCol := l.line, l.col
	start := l.pos
	for l.pos < len(l.src) && l.peek() >= '0' && l.peek() <= '9' {
		l.advance()
	}
	return Token{Type: TokNumber, Value: string(l.src[start:l.pos]), Line: startLine, Col: startCol}, nil
}

func (l *Lexer) readString() (Token, error) {
	startLine, startCol := l.line, l.col
	l.advance() // opening quote
	var buf bytes.Buffer
	for {
		if l.pos >= len(l.src) {
			return Token{}, &LexError{Line: startLine, Col: startCol, Message: "unterminated string literal"}
		}
		c := l.peek()
		if c == '"' {
			l.advance()
			break
		}
		if c == '\\' && l.peekAt(1) == '"' {
			l.advance()
			buf.WriteByte(l.advance())
			continue
		}
		buf.WriteByte(l.advance())
	}
	return Token{Type: TokString, Value: buf.String(), Line: startLine, Col: startCol}, nil
}

func (l *Lexer) readIdent() (Token, error) {
	startLine, startCol := l.line, l.col
	start := l.pos
	for l.pos < len(l.src) && isIdentCont(l.peek()) {
		l.advance()
	}
	return Token{Type: TokIdent, Value: string(l.src[start:l.pos]), Line: startLine, Col: startCol}, nil
}

// ReadRawUntilBangBrace scans raw bytes verbatim starting right after "{!"
// until it finds the closing "!}" delimiter, using bytes.Index for the search.
func (l *Lexer) ReadRawUntilBangBrace() (string, int, int, error) {
	startLine, startCol := l.line, l.col
	closeIdx := bytes.Index(l.src[l.pos:], []byte("!}"))
	if closeIdx == -1 {
		return "", startLine, startCol, &LexError{Line: startLine, Col: startCol, Message: "unterminated raw scope, expected closing !}"}
	}
	raw := string(l.src[l.pos : l.pos+closeIdx])
	for i := 0; i < closeIdx+2; i++ {
		l.advance()
	}
	trimmed := trimRawBlock(raw)
	return trimmed, startLine, startCol, nil
}

// trimRawBlock removes exactly one leading newline (and its preceding
// whitespace) and trailing whitespace-only line, preserving internal
// indentation of the code.
func trimRawBlock(raw string) string {
	s := raw
	if len(s) > 0 && s[0] == '\r' {
		s = s[1:]
	}
	if len(s) > 0 && s[0] == '\n' {
		s = s[1:]
	}
	end := len(s)
	for end > 0 && (s[end-1] == ' ' || s[end-1] == '\t' || s[end-1] == '\r' || s[end-1] == '\n') {
		end--
	}
	return s[:end]
}

// PeekChar returns the raw byte at the current lexer position, skipping
// nothing. Used by prose-mode parsing to decide what construct follows a
// text run without going through the structural tokenizer.
func (l *Lexer) PeekChar() byte {
	return l.peek()
}

// braceInlineKeywords lists inline elements of the form "kw{...}" (as
// opposed to "link(...) {...}", which takes attributes first). Keeping
// this as a single shared list means AtInlineStart and ReadTextRun can
// never drift out of sync on which identifiers start an inline element.
var braceInlineKeywords = map[string]bool{
	"b":      true,
	"i":      true,
	"code":   true,
	"strike": true,
}

// AtInlineStart reports whether the lexer is currently positioned at the
// start of a brace-style inline element ("b{", "i{", "code{", "strike{")
// or "link(...) {", returning the matched keyword if so. The lexer
// position is not moved.
func (l *Lexer) AtInlineStart() (string, bool) {
	if !isIdentStart(l.peek()) {
		return "", false
	}
	ident := l.peekIdent()
	if braceInlineKeywords[ident] && l.charAfterIdent(ident) == '{' {
		return ident, true
	}
	if ident == "link" && l.charAfterIdent(ident) == '(' {
		return ident, true
	}
	return "", false
}

// ConsumeIdentOnly advances past the given identifier text (already
// confirmed present via AtInlineStart), leaving whatever follows (a '{'
// or a '(') at the current position for the caller to handle.
func (l *Lexer) ConsumeIdentOnly(ident string) {
	for i := 0; i < len(ident); i++ {
		l.advance()
	}
}

// ConsumeLBrace advances past a '{' at the current raw position, or
// returns an error if the current byte isn't '{'.
func (l *Lexer) ConsumeLBrace() error {
	if l.pos >= len(l.src) || l.peek() != '{' {
		return l.errorf("expected '{'")
	}
	l.advance()
	return nil
}

// ConsumeRBrace advances past a '}' at the current raw position, or
// returns an error if the current byte isn't '}'.
func (l *Lexer) ConsumeRBrace() error {
	if l.pos >= len(l.src) || l.peek() != '}' {
		return l.errorf("expected '}'")
	}
	l.advance()
	return nil
}

// AtRawIdent reports whether the identifier at the current raw position
// exactly matches want, without consuming anything.
func (l *Lexer) AtRawIdent(want string) bool {
	if !isIdentStart(l.peek()) {
		return false
	}
	return l.peekIdent() == want
}

// AtEOF reports whether the lexer has consumed the entire input.
func (l *Lexer) AtEOF() bool {
	return l.pos >= len(l.src)
}

// ReadRawString reads a "..."-quoted string at the current raw position
// (same escaping rules as the tokenizer's string literal) and returns its
// unescaped contents.
func (l *Lexer) ReadRawString() (string, error) {
	if l.pos >= len(l.src) || l.peek() != '"' {
		return "", l.errorf("expected string literal")
	}
	tok, err := l.readString()
	if err != nil {
		return "", err
	}
	return tok.Value, nil
}

// SkipRawSpaces advances past any spaces/tabs at the current raw position
// (not newlines — used for the tight spacing inside inline attribute lists).
func (l *Lexer) SkipRawSpaces() {
	for l.pos < len(l.src) && (l.peek() == ' ' || l.peek() == '\t') {
		l.advance()
	}
}

// ReadRawIdent reads an identifier at the current raw position.
func (l *Lexer) ReadRawIdent() string {
	return l.peekIdentAdvance()
}

func (l *Lexer) peekIdentAdvance() string {
	start := l.pos
	for l.pos < len(l.src) && isIdentCont(l.peek()) {
		l.advance()
	}
	return string(l.src[start:l.pos])
}

// ConsumeRawByte advances past a single expected byte at the current raw
// position, or returns an error naming what was expected.
func (l *Lexer) ConsumeRawByte(expected byte, what string) error {
	if l.pos >= len(l.src) || l.peek() != expected {
		return l.errorf("expected %s", what)
	}
	l.advance()
	return nil
}

// ReadBalancedBraces reads raw content up to the '}' that balances the
// '{' already consumed by the caller (depth starts at 1), tracking
// nesting depth so inner "{"/"}" pairs — e.g. a struct definition inside
// an inline code{...} span — don't prematurely end the read. The closing
// '}' itself is NOT consumed; the caller does that afterward. Strings are
// not given special treatment here: this is for raw code snippets, where
// brace characters inside string literals are rare and, if present,
// still balance correctly in the overwhelmingly common case of
// brace-structured code (C-like blocks, JSON, etc).
func (l *Lexer) ReadBalancedBraces() string {
	var buf bytes.Buffer
	depth := 1
	for l.pos < len(l.src) {
		c := l.peek()
		if c == '{' {
			depth++
		} else if c == '}' {
			depth--
			if depth == 0 {
				break
			}
		}
		buf.WriteByte(l.advance())
	}
	return buf.String()
}

// ReadTextRun reads plain text content inside a p/h block up to (but not
// including) the next inline-start marker (b{, i{, code{, strike{,
// link()) or a closing }.
func (l *Lexer) ReadTextRun() string {
	return l.readTextRunStoppingAt(nil)
}

// ReadTextRunUntilBlock behaves like ReadTextRun, but additionally stops
// right before a bare occurrence of any keyword in stopWords (e.g. "list"
// or "quote") followed by whitespace-then-'(' or whitespace-then-'{' —
// i.e. the start of a nested block construct, not just the word appearing
// in prose. This lets item/quote bodies detect a nested list{}/quote{}
// without swallowing it as plain text first.
func (l *Lexer) ReadTextRunUntilBlock(stopWords ...string) string {
	return l.readTextRunStoppingAt(stopWords)
}

func (l *Lexer) readTextRunStoppingAt(stopWords []string) string {
	var buf bytes.Buffer
	for l.pos < len(l.src) {
		c := l.peek()
		if c == '}' {
			break
		}
		if isIdentStart(c) {
			ident := l.peekIdent()
			if braceInlineKeywords[ident] && l.charAfterIdent(ident) == '{' {
				break
			}
			if ident == "link" && l.charAfterIdent(ident) == '(' {
				break
			}
			if stopWordMatches(stopWords, ident) && l.startsBlockAfterIdent(ident) {
				break
			}
		}
		buf.WriteByte(l.advance())
	}
	return buf.String()
}

func stopWordMatches(stopWords []string, ident string) bool {
	for _, w := range stopWords {
		if w == ident {
			return true
		}
	}
	return false
}

// startsBlockAfterIdent reports whether ident, at the current lexer
// position, is immediately followed (allowing spaces/tabs) by '(' or '{'
// — the shape of a block construct like "list {" or "list(ordered: true) {"
// — as opposed to the word simply appearing in running prose.
func (l *Lexer) startsBlockAfterIdent(ident string) bool {
	i := l.pos + len(ident)
	for i < len(l.src) && (l.src[i] == ' ' || l.src[i] == '\t') {
		i++
	}
	if i >= len(l.src) {
		return false
	}
	return l.src[i] == '(' || l.src[i] == '{'
}

func (l *Lexer) peekIdent() string {
	i := l.pos
	for i < len(l.src) && isIdentCont(l.src[i]) {
		i++
	}
	return string(l.src[l.pos:i])
}

func (l *Lexer) charAfterIdent(ident string) byte {
	i := l.pos + len(ident)
	if i < len(l.src) {
		return l.src[i]
	}
	return 0
}

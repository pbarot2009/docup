package src

import "fmt"

type Parser struct {
	lex   *Lexer
	cur   Token
	depth int
}

// maxInlineDepth bounds nested inline elements (b{i{b{...}}}) to prevent
// pathological or adversarial input from overflowing the Go call stack.
const maxInlineDepth = 64

func NewParser(src []byte) (*Parser, error) {
	lex := NewLexer(src)
	p := &Parser{lex: lex}
	if err := p.next(); err != nil {
		return nil, err
	}
	return p, nil
}

func (p *Parser) next() error {
	tok, err := p.lex.NextToken()
	if err != nil {
		return err
	}
	p.cur = tok
	return nil
}

func (p *Parser) errorf(format string, args ...interface{}) error {
	return &ParseError{Line: p.cur.Line, Col: p.cur.Col, Message: fmt.Sprintf(format, args...)}
}

func (p *Parser) expect(t TokenType, what string) (Token, error) {
	if p.cur.Type != t {
		return Token{}, p.errorf("expected %s, got %q", what, p.cur.Value)
	}
	tok := p.cur
	if err := p.next(); err != nil {
		return Token{}, err
	}
	return tok, nil
}

// ParseDocument parses the entire token stream into a DocumentNode.
func (p *Parser) ParseDocument() (*DocumentNode, error) {
	doc := &DocumentNode{}
	for p.cur.Type != TokEOF {
		if p.cur.Type != TokIdent {
			return nil, p.errorf("expected a top-level block (meta, h, p, codeblock, hr, list, quote, image, table), got %q", p.cur.Value)
		}
		switch p.cur.Value {
		case "meta":
			if doc.Meta != nil {
				return nil, p.errorf("duplicate meta block")
			}
			m, err := p.parseMeta()
			if err != nil {
				return nil, err
			}
			doc.Meta = m
		case "h":
			h, err := p.parseHeading()
			if err != nil {
				return nil, err
			}
			doc.Blocks = append(doc.Blocks, h)
		case "p":
			para, err := p.parseParagraph()
			if err != nil {
				return nil, err
			}
			doc.Blocks = append(doc.Blocks, para)
		case "codeblock":
			cb, err := p.parseCodeBlock()
			if err != nil {
				return nil, err
			}
			doc.Blocks = append(doc.Blocks, cb)
		case "hr":
			hr, err := p.parseHR()
			if err != nil {
				return nil, err
			}
			doc.Blocks = append(doc.Blocks, hr)
		case "list":
			list, err := p.parseList()
			if err != nil {
				return nil, err
			}
			doc.Blocks = append(doc.Blocks, list)
		case "quote":
			q, err := p.parseQuote()
			if err != nil {
				return nil, err
			}
			doc.Blocks = append(doc.Blocks, q)
		case "image":
			img, err := p.parseImage()
			if err != nil {
				return nil, err
			}
			doc.Blocks = append(doc.Blocks, img)
		case "table":
			tbl, err := p.parseTable()
			if err != nil {
				return nil, err
			}
			doc.Blocks = append(doc.Blocks, tbl)
		default:
			return nil, p.errorf("unknown block type %q", p.cur.Value)
		}
	}
	return doc, nil
}

func (p *Parser) parseMeta() (*MetaNode, error) {
	line, col := p.cur.Line, p.cur.Col
	if err := p.next(); err != nil { // consume 'meta'
		return nil, err
	}
	if _, err := p.expect(TokLBrace, "'{'"); err != nil {
		return nil, err
	}
	fields := make(map[string]string)
	for p.cur.Type != TokRBrace {
		key, err := p.expect(TokIdent, "metadata key")
		if err != nil {
			return nil, err
		}
		if _, err := p.expect(TokColon, "':'"); err != nil {
			return nil, err
		}
		val, err := p.expect(TokString, "string value")
		if err != nil {
			return nil, err
		}
		fields[key.Value] = val.Value
		if p.cur.Type == TokComma {
			if err := p.next(); err != nil {
				return nil, err
			}
		}
	}
	if _, err := p.expect(TokRBrace, "'}'"); err != nil {
		return nil, err
	}
	return &MetaNode{Line: line, Col: col, Fields: fields}, nil
}

// parseAttrs parses an optional "(key: "val", key2: "val2", posArg)" list.
// Positional string/ident args before the first key:value pair are ignored
// by callers that don't need them (kept simple for Phase 1's fixed grammar).
func (p *Parser) parseAttrs() (map[string]string, []string, error) {
	attrs := make(map[string]string)
	var positional []string
	if p.cur.Type != TokLParen {
		return attrs, positional, nil
	}
	if err := p.next(); err != nil { // consume '('
		return nil, nil, err
	}
	for p.cur.Type != TokRParen {
		if p.cur.Type == TokString {
			positional = append(positional, p.cur.Value)
			if err := p.next(); err != nil {
				return nil, nil, err
			}
		} else if p.cur.Type == TokIdent {
			key := p.cur.Value
			if err := p.next(); err != nil {
				return nil, nil, err
			}
			if _, err := p.expect(TokColon, "':'"); err != nil {
				return nil, nil, err
			}
			val, err := p.expectAttrValue()
			if err != nil {
				return nil, nil, err
			}
			attrs[key] = val
		} else {
			return nil, nil, p.errorf("unexpected token %q in attribute list", p.cur.Value)
		}
		if p.cur.Type == TokComma {
			if err := p.next(); err != nil {
				return nil, nil, err
			}
		}
	}
	if _, err := p.expect(TokRParen, "')'"); err != nil {
		return nil, nil, err
	}
	return attrs, positional, nil
}

// expectAttrValue accepts either a quoted string or a bare boolean
// identifier (true/false) as an attribute value, since flags like
// "ordered: true" read more naturally unquoted.
func (p *Parser) expectAttrValue() (string, error) {
	if p.cur.Type == TokString {
		v := p.cur.Value
		if err := p.next(); err != nil {
			return "", err
		}
		return v, nil
	}
	if p.cur.Type == TokIdent && (p.cur.Value == "true" || p.cur.Value == "false") {
		v := p.cur.Value
		if err := p.next(); err != nil {
			return "", err
		}
		return v, nil
	}
	return "", p.errorf("expected string or boolean value, got %q", p.cur.Value)
}

func (p *Parser) parseHeading() (*HeadingNode, error) {
	line, col := p.cur.Line, p.cur.Col
	if err := p.next(); err != nil { // consume 'h'
		return nil, err
	}
	if _, err := p.expect(TokLParen, "'(' after h"); err != nil {
		return nil, err
	}
	levelTok, err := p.expect(TokNumber, "heading level")
	if err != nil {
		return nil, err
	}
	level := 0
	if _, scanErr := fmt.Sscanf(levelTok.Value, "%d", &level); scanErr != nil {
		return nil, p.errorf("invalid heading level %q", levelTok.Value)
	}
	attrs := make(map[string]string)
	for p.cur.Type == TokComma {
		if err := p.next(); err != nil {
			return nil, err
		}
		key, err := p.expect(TokIdent, "attribute name")
		if err != nil {
			return nil, err
		}
		if _, err := p.expect(TokColon, "':'"); err != nil {
			return nil, err
		}
		val, err := p.expect(TokString, "attribute value")
		if err != nil {
			return nil, err
		}
		attrs[key.Value] = val.Value
	}
	if _, err := p.expect(TokRParen, "')'"); err != nil {
		return nil, err
	}
	if p.cur.Type != TokLBrace {
		return nil, p.errorf("expected '{', got %q", p.cur.Value)
	}
	children, err := p.parseProseBlock()
	if err != nil {
		return nil, err
	}
	return &HeadingNode{Line: line, Col: col, Level: level, Attrs: attrs, Children: children}, nil
}

func (p *Parser) parseParagraph() (*ParagraphNode, error) {
	line, col := p.cur.Line, p.cur.Col
	if err := p.next(); err != nil { // consume 'p'
		return nil, err
	}
	if p.cur.Type != TokLBrace {
		return nil, p.errorf("expected '{', got %q", p.cur.Value)
	}
	children, err := p.parseProseBlock()
	if err != nil {
		return nil, err
	}
	return &ParagraphNode{Line: line, Col: col, Children: children}, nil
}

// parseProseBlock expects p.cur to be the opening '{' token of a p/h block.
// From here on, parsing switches entirely to the lexer's raw byte position
// (never touching structural tokens) until the matching '}' is consumed,
// at which point a single call to p.next() resyncs the token stream. This
// avoids any drift between token lookahead and raw scanning, since the two
// never interleave mid-block.
func (p *Parser) parseProseBlock() ([]Node, error) {
	// p.cur is the '{' token: NextToken() already advanced the lexer's raw
	// position past it, so raw prose scanning can start immediately.
	children, err := p.parseProseUntilRBrace()
	if err != nil {
		return nil, err
	}
	if err := p.next(); err != nil { // resync token stream past '}'
		return nil, err
	}
	return children, nil
}

// parseProseUntilRBrace reads text and nested inline elements (b{}, i{},
// code{}) directly from the lexer's raw byte stream until a top-level '}'
// is found, which it consumes before returning.
func (p *Parser) parseProseUntilRBrace() ([]Node, error) {
	var children []Node
	for {
		text := collapseWhitespace(p.lex.ReadTextRun())
		if text != "" {
			children = append(children, &InlineNode{NodeKind: InlineText, Value: text})
		}
		if p.lex.AtEOF() {
			return nil, &ParseError{Line: p.lex.line, Col: p.lex.col, Message: "unterminated block, expected '}'"}
		}
		if ident, ok := p.lex.AtInlineStart(); ok {
			inline, err := p.parseInlineRaw(ident)
			if err != nil {
				return nil, err
			}
			children = append(children, inline)
			continue
		}
		// Not an inline start and ReadTextRun stopped, so this must be '}'.
		if err := p.lex.ConsumeRBrace(); err != nil {
			return nil, err
		}
		trimBoundaryWhitespace(children)
		return children, nil
	}
}

// trimBoundaryWhitespace strips a leading space from the first text child
// and a trailing space from the last text child of a prose block, since
// those reflect the block's own opening/closing whitespace rather than
// meaningful separation between words.
func trimBoundaryWhitespace(children []Node) {
	if len(children) == 0 {
		return
	}
	if first, ok := children[0].(*InlineNode); ok && first.NodeKind == InlineText {
		first.Value = trimLeftSpace(first.Value)
	}
	if last, ok := children[len(children)-1].(*InlineNode); ok && last.NodeKind == InlineText {
		last.Value = trimRightSpace(last.Value)
	}
}

func trimLeftSpace(s string) string {
	if len(s) > 0 && s[0] == ' ' {
		return s[1:]
	}
	return s
}

func trimRightSpace(s string) string {
	if len(s) > 0 && s[len(s)-1] == ' ' {
		return s[:len(s)-1]
	}
	return s
}

// parseInlineRaw parses b{...}, i{...}, code{...}, or link(url){...}
// entirely via raw byte scanning. The lexer must be positioned exactly at
// the start of `ident`.
func (p *Parser) parseInlineRaw(ident string) (*InlineNode, error) {
	line, col := p.lex.line, p.lex.col

	p.depth++
	defer func() { p.depth-- }()
	if p.depth > maxInlineDepth {
		return nil, &ParseError{Line: line, Col: col, Message: fmt.Sprintf("inline elements nested too deeply (limit %d)", maxInlineDepth)}
	}

	if ident == "link" {
		return p.parseLinkRaw(line, col)
	}

	var kind string
	switch ident {
	case "b":
		kind = InlineBold
	case "i":
		kind = InlineItalic
	case "code":
		kind = InlineCode
	case "strike":
		kind = InlineStrike
	}
	p.lex.ConsumeIdentOnly(ident)
	if err := p.lex.ConsumeLBrace(); err != nil {
		return nil, err
	}

	if kind == InlineCode {
		raw := p.lex.ReadBalancedBraces()
		if err := p.lex.ConsumeRBrace(); err != nil {
			return nil, err
		}
		return &InlineNode{Line: line, Col: col, NodeKind: kind, Value: raw}, nil
	}

	children, err := p.parseProseUntilRBrace()
	if err != nil {
		return nil, err
	}
	return &InlineNode{Line: line, Col: col, NodeKind: kind, Children: children}, nil
}

// parseLinkRaw parses link("url") { link text }, entirely via raw byte
// scanning: link( "url" [, title: "..."] ) { ... }. Only the URL is kept
// for Phase 1's inline link node; any extra attributes are read and
// discarded so they don't break parsing.
func (p *Parser) parseLinkRaw(line, col int) (*InlineNode, error) {
	p.lex.ConsumeIdentOnly("link")
	if err := p.lex.ConsumeRawByte('(', "'(' after link"); err != nil {
		return nil, err
	}
	p.lex.SkipRawSpaces()
	url, err := p.lex.ReadRawString()
	if err != nil {
		return nil, err
	}
	p.lex.SkipRawSpaces()
	for p.lex.PeekChar() == ',' {
		if err := p.lex.ConsumeRawByte(',', "','"); err != nil {
			return nil, err
		}
		p.lex.SkipRawSpaces()
		attrName := p.lex.ReadRawIdent()
		if attrName == "" {
			return nil, p.lex.errorf("expected attribute name in link(...)")
		}
		p.lex.SkipRawSpaces()
		if err := p.lex.ConsumeRawByte(':', "':' in link attribute"); err != nil {
			return nil, err
		}
		p.lex.SkipRawSpaces()
		if _, err := p.lex.ReadRawString(); err != nil {
			return nil, err
		}
		p.lex.SkipRawSpaces()
	}
	if err := p.lex.ConsumeRawByte(')', "')' to close link(...)"); err != nil {
		return nil, err
	}
	p.lex.SkipRawSpaces()
	if err := p.lex.ConsumeLBrace(); err != nil {
		return nil, err
	}
	children, err := p.parseProseUntilRBrace()
	if err != nil {
		return nil, err
	}
	return &InlineNode{Line: line, Col: col, NodeKind: InlineLink, URL: url, Children: children}, nil
}

func (p *Parser) parseCodeBlock() (*CodeBlockNode, error) {
	line, col := p.cur.Line, p.cur.Col
	if err := p.next(); err != nil { // consume 'codeblock'
		return nil, err
	}
	attrs, _, err := p.parseAttrs()
	if err != nil {
		return nil, err
	}
	if p.cur.Type != TokRawScopeOpen {
		return nil, p.errorf("expected '{!' to open raw code scope, got %q", p.cur.Value)
	}
	raw, _, _, err := p.lex.ReadRawUntilBangBrace()
	if err != nil {
		return nil, err
	}
	if err := p.next(); err != nil {
		return nil, err
	}
	return &CodeBlockNode{
		Line:     line,
		Col:      col,
		Language: attrs["lang"],
		File:     attrs["file"],
		RawCode:  raw,
	}, nil
}

// parseHR parses a standalone "hr {}" or "hr{}" block.
func (p *Parser) parseHR() (*HRNode, error) {
	line, col := p.cur.Line, p.cur.Col
	if err := p.next(); err != nil { // consume 'hr'
		return nil, err
	}
	if _, err := p.expect(TokLBrace, "'{' after hr"); err != nil {
		return nil, err
	}
	if _, err := p.expect(TokRBrace, "'}' to close hr{}"); err != nil {
		return nil, err
	}
	return &HRNode{Line: line, Col: col}, nil
}

// parseList parses "list { item { ... } item { ... } }" or
// "list(ordered: true) { ... }".
func (p *Parser) parseList() (*ListNode, error) {
	line, col := p.cur.Line, p.cur.Col
	if err := p.next(); err != nil { // consume 'list'
		return nil, err
	}
	attrs, _, err := p.parseAttrs()
	if err != nil {
		return nil, err
	}
	if _, err := p.expect(TokLBrace, "'{' after list"); err != nil {
		return nil, err
	}
	var items []*ItemNode
	for p.cur.Type != TokRBrace {
		if p.cur.Type != TokIdent || (p.cur.Value != "item" && p.cur.Value != "task") {
			return nil, p.errorf("expected 'item' or 'task' inside list, got %q", p.cur.Value)
		}
		var item *ItemNode
		var err error
		if p.cur.Value == "task" {
			item, err = p.parseTask()
		} else {
			item, err = p.parseItem()
		}
		if err != nil {
			return nil, err
		}
		items = append(items, item)
	}
	if _, err := p.expect(TokRBrace, "'}' to close list"); err != nil {
		return nil, err
	}
	return &ListNode{Line: line, Col: col, Ordered: attrs["ordered"] == "true", Items: items}, nil
}

func (p *Parser) parseItem() (*ItemNode, error) {
	line, col := p.cur.Line, p.cur.Col
	if err := p.next(); err != nil { // consume 'item'
		return nil, err
	}
	if p.cur.Type != TokLBrace {
		return nil, p.errorf("expected '{' after item, got %q", p.cur.Value)
	}
	children, err := p.parseItemBody()
	if err != nil {
		return nil, err
	}
	return &ItemNode{Line: line, Col: col, Children: children}, nil
}

// parseTask parses "task(done: true) { ... }" or "task(done: false) { ... }",
// the task-list counterpart of parseItem.
func (p *Parser) parseTask() (*ItemNode, error) {
	line, col := p.cur.Line, p.cur.Col
	if err := p.next(); err != nil { // consume 'task'
		return nil, err
	}
	attrs, _, err := p.parseAttrs()
	if err != nil {
		return nil, err
	}
	done := attrs["done"] == "true"
	if p.cur.Type != TokLBrace {
		return nil, p.errorf("expected '{' after task(...), got %q", p.cur.Value)
	}
	children, err := p.parseItemBody()
	if err != nil {
		return nil, err
	}
	return &ItemNode{Line: line, Col: col, Children: children, Done: &done}, nil
}

// parseItemBody parses an item/task body: p.cur must be the opening '{'.
// It behaves like ordinary prose, except that a "list { ... }" appearing
// where a text run would otherwise be read is parsed as a genuinely
// nested ListNode child instead of literal text — this is how DocUP
// supports nested lists in Phase 1, without adding block-nesting to
// prose everywhere.
func (p *Parser) parseItemBody() ([]Node, error) {
	var children []Node
	for {
		text := collapseWhitespace(p.lex.ReadTextRunUntilBlock("list"))
		if text != "" {
			children = append(children, &InlineNode{NodeKind: InlineText, Value: text})
		}
		if p.lex.AtEOF() {
			return nil, &ParseError{Line: p.lex.line, Col: p.lex.col, Message: "unterminated item, expected '}'"}
		}
		if ident, ok := p.lex.AtInlineStart(); ok {
			inline, err := p.parseInlineRaw(ident)
			if err != nil {
				return nil, err
			}
			children = append(children, inline)
			continue
		}
		if p.lex.AtRawIdent("list") {
			p.depth++
			if p.depth > maxInlineDepth {
				return nil, &ParseError{Line: p.lex.line, Col: p.lex.col, Message: fmt.Sprintf("lists nested too deeply (limit %d)", maxInlineDepth)}
			}
			nested, err := p.parseNestedList()
			p.depth--
			if err != nil {
				return nil, err
			}
			children = append(children, nested)
			// parseNestedList finishes via token-based parsing (its
			// closing '}' is consumed with p.next(), which tokenizes
			// one token past it) — the raw lexer position is now ahead
			// of what plain ReadTextRun() scanning expects. A nested
			// list must therefore be the last thing in an item's body:
			// the only valid continuation here is the item's own
			// closing '}', consumed as a token rather than raw bytes.
			if p.cur.Type != TokRBrace {
				return nil, p.errorf("expected '}' to close item after nested list, got %q", p.cur.Value)
			}
			if err := p.next(); err != nil {
				return nil, err
			}
			trimBoundaryWhitespace(children)
			return children, nil
		}
		if err := p.lex.ConsumeRBrace(); err != nil {
			return nil, err
		}
		trimBoundaryWhitespace(children)
		if err := p.next(); err != nil { // resync token stream past '}'
			return nil, err
		}
		return children, nil
	}
}

// parseNestedList resyncs the token stream to the lexer's current raw
// position (which is sitting exactly at "list"), then delegates to the
// ordinary token-based parseList, which itself consumes the leading
// "list" token — so this must NOT advance past it first.
func (p *Parser) parseNestedList() (*ListNode, error) {
	if err := p.next(); err != nil { // p.cur becomes the 'list' token
		return nil, err
	}
	return p.parseList()
}

// parseQuote parses "quote { ... }", rendered as a blockquote. A quote's
// body is prose (text plus b/i/code/strike/link inline elements) that
// may also directly contain a nested "quote { ... }" block, which is
// parsed as a genuinely nested QuoteNode child rather than literal text.
func (p *Parser) parseQuote() (*QuoteNode, error) {
	line, col := p.cur.Line, p.cur.Col
	if err := p.next(); err != nil { // consume 'quote'
		return nil, err
	}
	if p.cur.Type != TokLBrace {
		return nil, p.errorf("expected '{' after quote, got %q", p.cur.Value)
	}
	children, err := p.parseQuoteBody()
	if err != nil {
		return nil, err
	}
	return &QuoteNode{Line: line, Col: col, Children: children}, nil
}

// parseQuoteBody mirrors parseItemBody, but recognizes a nested "quote"
// block instead of a nested "list" block.
func (p *Parser) parseQuoteBody() ([]Node, error) {
	var children []Node
	for {
		text := collapseWhitespace(p.lex.ReadTextRunUntilBlock("quote"))
		if text != "" {
			children = append(children, &InlineNode{NodeKind: InlineText, Value: text})
		}
		if p.lex.AtEOF() {
			return nil, &ParseError{Line: p.lex.line, Col: p.lex.col, Message: "unterminated quote, expected '}'"}
		}
		if ident, ok := p.lex.AtInlineStart(); ok {
			inline, err := p.parseInlineRaw(ident)
			if err != nil {
				return nil, err
			}
			children = append(children, inline)
			continue
		}
		if p.lex.AtRawIdent("quote") {
			p.depth++
			if p.depth > maxInlineDepth {
				return nil, &ParseError{Line: p.lex.line, Col: p.lex.col, Message: fmt.Sprintf("quotes nested too deeply (limit %d)", maxInlineDepth)}
			}
			if err := p.next(); err != nil {
				p.depth--
				return nil, err
			}
			nested, err := p.parseQuote()
			p.depth--
			if err != nil {
				return nil, err
			}
			children = append(children, nested)
			// As with nested lists, parseQuote finishes via a token-based
			// resync one token past its closing '}', so plain raw-byte
			// ReadTextRun() scanning can no longer be trusted here. A
			// nested quote must be the last thing in its parent quote's
			// body: the only valid continuation is the parent's own
			// closing '}', consumed as a token.
			if p.cur.Type != TokRBrace {
				return nil, p.errorf("expected '}' to close quote after nested quote, got %q", p.cur.Value)
			}
			if err := p.next(); err != nil {
				return nil, err
			}
			trimBoundaryWhitespace(children)
			return children, nil
		}
		if err := p.lex.ConsumeRBrace(); err != nil {
			return nil, err
		}
		trimBoundaryWhitespace(children)
		if err := p.next(); err != nil { // resync token stream past '}'
			return nil, err
		}
		return children, nil
	}
}

// parseImage parses the self-closing "image(url, alt: "...")" block. It
// has no body — the URL and optional alt text are both attributes.
func (p *Parser) parseImage() (*ImageNode, error) {
	line, col := p.cur.Line, p.cur.Col
	if err := p.next(); err != nil { // consume 'image'
		return nil, err
	}
	if p.cur.Type != TokLParen {
		return nil, p.errorf("expected '(' after image, got %q", p.cur.Value)
	}
	if err := p.next(); err != nil { // consume '('
		return nil, err
	}
	srcTok, err := p.expect(TokString, "image source URL")
	if err != nil {
		return nil, err
	}
	alt := ""
	for p.cur.Type == TokComma {
		if err := p.next(); err != nil {
			return nil, err
		}
		key, err := p.expect(TokIdent, "attribute name")
		if err != nil {
			return nil, err
		}
		if _, err := p.expect(TokColon, "':'"); err != nil {
			return nil, err
		}
		val, err := p.expect(TokString, "attribute value")
		if err != nil {
			return nil, err
		}
		if key.Value == "alt" {
			alt = val.Value
		}
	}
	if _, err := p.expect(TokRParen, "')'"); err != nil {
		return nil, err
	}
	return &ImageNode{Line: line, Col: col, Src: srcTok.Value, Alt: alt}, nil
}

// parseTable parses "table { row { cell { ... } cell { ... } } ... }".
// Rows may be marked as a header row with row(header: true) { ... }.
func (p *Parser) parseTable() (*TableNode, error) {
	line, col := p.cur.Line, p.cur.Col
	if err := p.next(); err != nil { // consume 'table'
		return nil, err
	}
	if _, _, err := p.parseAttrs(); err != nil { // table-level attrs (e.g. class) parsed and discarded in Phase 1
		return nil, err
	}
	if _, err := p.expect(TokLBrace, "'{' after table"); err != nil {
		return nil, err
	}
	var rows []*RowNode
	for p.cur.Type != TokRBrace {
		if p.cur.Type != TokIdent || p.cur.Value != "row" {
			return nil, p.errorf("expected 'row' inside table, got %q", p.cur.Value)
		}
		row, err := p.parseRow()
		if err != nil {
			return nil, err
		}
		rows = append(rows, row)
	}
	if _, err := p.expect(TokRBrace, "'}' to close table"); err != nil {
		return nil, err
	}
	if len(rows) == 0 {
		return nil, &ParseError{Line: line, Col: col, Message: "table must contain at least one row"}
	}
	return &TableNode{Line: line, Col: col, Rows: rows}, nil
}

func (p *Parser) parseRow() (*RowNode, error) {
	line, col := p.cur.Line, p.cur.Col
	if err := p.next(); err != nil { // consume 'row'
		return nil, err
	}
	attrs, _, err := p.parseAttrs()
	if err != nil {
		return nil, err
	}
	if _, err := p.expect(TokLBrace, "'{' after row"); err != nil {
		return nil, err
	}
	var cells []*CellNode
	for p.cur.Type != TokRBrace {
		if p.cur.Type != TokIdent || p.cur.Value != "cell" {
			return nil, p.errorf("expected 'cell' inside row, got %q", p.cur.Value)
		}
		cell, err := p.parseCell()
		if err != nil {
			return nil, err
		}
		cells = append(cells, cell)
	}
	if _, err := p.expect(TokRBrace, "'}' to close row"); err != nil {
		return nil, err
	}
	return &RowNode{Line: line, Col: col, Header: attrs["header"] == "true", Cells: cells}, nil
}

func (p *Parser) parseCell() (*CellNode, error) {
	line, col := p.cur.Line, p.cur.Col
	if err := p.next(); err != nil { // consume 'cell'
		return nil, err
	}
	if p.cur.Type != TokLBrace {
		return nil, p.errorf("expected '{' after cell, got %q", p.cur.Value)
	}
	children, err := p.parseProseBlock()
	if err != nil {
		return nil, err
	}
	return &CellNode{Line: line, Col: col, Children: children}, nil
}

func isSpace(c byte) bool {
	return c == ' ' || c == '\t' || c == '\r' || c == '\n'
}

// collapseWhitespace normalizes internal newlines/tabs/runs of spaces into
// single spaces, matching typical markup-language text-flow semantics.
func collapseWhitespace(s string) string {
	out := make([]byte, 0, len(s))
	prevSpace := false
	for i := 0; i < len(s); i++ {
		c := s[i]
		if isSpace(c) {
			if !prevSpace {
				out = append(out, ' ')
			}
			prevSpace = true
			continue
		}
		prevSpace = false
		out = append(out, c)
	}
	return string(out)
}

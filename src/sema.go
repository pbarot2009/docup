package src

import "fmt"

type SemaError struct {
	Line, Col int
	Message   string
}

func (e *SemaError) Error() string {
	return fmt.Sprintf("semantic error at %d:%d: %s", e.Line, e.Col, e.Message)
}

// Analyze validates the document AST and returns a SemaError on the first
// violation found.
func Analyze(doc *DocumentNode) error {
	if doc.Meta != nil {
		if err := checkMetaFields(doc.Meta); err != nil {
			return err
		}
	}
	for _, block := range doc.Blocks {
		if err := analyzeBlock(block); err != nil {
			return err
		}
	}
	return nil
}

func checkMetaFields(m *MetaNode) error {
	for _, required := range []string{"title"} {
		if _, ok := m.Fields[required]; !ok {
			return &SemaError{Line: m.Line, Col: m.Col, Message: fmt.Sprintf("meta block is missing required field %q", required)}
		}
	}
	return nil
}

func analyzeBlock(n Node) error {
	switch b := n.(type) {
	case *HeadingNode:
		if b.Level < 1 || b.Level > 6 {
			return &SemaError{Line: b.Line, Col: b.Col, Message: fmt.Sprintf("heading level %d out of range, must be 1-6", b.Level)}
		}
		return analyzeInlines(b.Children)
	case *ParagraphNode:
		return analyzeInlines(b.Children)
	case *CodeBlockNode:
		if b.RawCode == "" {
			return &SemaError{Line: b.Line, Col: b.Col, Message: "codeblock has empty content"}
		}
		return nil
	case *HRNode:
		return nil
	case *ListNode:
		return analyzeList(b)
	case *QuoteNode:
		return analyzeInlines(b.Children)
	case *ImageNode:
		if b.Src == "" {
			return &SemaError{Line: b.Line, Col: b.Col, Message: "image is missing a source URL"}
		}
		return nil
	case *TableNode:
		return analyzeTable(b)
	case *MetaNode:
		return &SemaError{Line: b.Line, Col: b.Col, Message: "meta block must appear only once, at document root"}
	default:
		return fmt.Errorf("semantic error: unknown node kind %q", n.Kind())
	}
}

func analyzeList(list *ListNode) error {
	if len(list.Items) == 0 {
		return &SemaError{Line: list.Line, Col: list.Col, Message: "list must contain at least one item"}
	}
	for _, item := range list.Items {
		if err := analyzeItemChildren(item.Children); err != nil {
			return err
		}
	}
	return nil
}

func analyzeTable(t *TableNode) error {
	if len(t.Rows) == 0 {
		return &SemaError{Line: t.Line, Col: t.Col, Message: "table must contain at least one row"}
	}
	width := -1
	for _, row := range t.Rows {
		if len(row.Cells) == 0 {
			return &SemaError{Line: row.Line, Col: row.Col, Message: "row must contain at least one cell"}
		}
		if width == -1 {
			width = len(row.Cells)
		} else if len(row.Cells) != width {
			return &SemaError{Line: row.Line, Col: row.Col, Message: fmt.Sprintf("row has %d cells, expected %d to match the table's other rows", len(row.Cells), width)}
		}
		for _, cell := range row.Cells {
			if err := analyzeInlines(cell.Children); err != nil {
				return err
			}
		}
	}
	return nil
}

// analyzeItemChildren validates a list item's/task's children, which may
// mix inline content with a nested ListNode (from nested "list { ... }").
func analyzeItemChildren(children []Node) error {
	for _, c := range children {
		if nested, ok := c.(*ListNode); ok {
			if err := analyzeList(nested); err != nil {
				return err
			}
			continue
		}
		if err := analyzeInlines([]Node{c}); err != nil {
			return err
		}
	}
	return nil
}

func analyzeInlines(children []Node) error {
	for _, c := range children {
		if nested, ok := c.(*QuoteNode); ok {
			if err := analyzeInlines(nested.Children); err != nil {
				return err
			}
			continue
		}
		inline, ok := c.(*InlineNode)
		if !ok {
			return fmt.Errorf("semantic error: unexpected node kind %q in prose content", c.Kind())
		}
		switch inline.NodeKind {
		case InlineText, InlineCode:
			// leaf inline kinds, nothing further to validate
		case InlineBold, InlineItalic, InlineStrike:
			if err := analyzeInlines(inline.Children); err != nil {
				return err
			}
		case InlineLink:
			if inline.URL == "" {
				return &SemaError{Line: inline.Line, Col: inline.Col, Message: "link is missing a URL"}
			}
			if err := analyzeInlines(inline.Children); err != nil {
				return err
			}
		default:
			return &SemaError{Line: inline.Line, Col: inline.Col, Message: fmt.Sprintf("unknown inline kind %q", inline.NodeKind)}
		}
	}
	return nil
}

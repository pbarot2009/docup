package src

type Node interface {
	Kind() string
}

type DocumentNode struct {
	Meta   *MetaNode
	Blocks []Node
}

func (n *DocumentNode) Kind() string { return "Document" }

type MetaNode struct {
	Line, Col int
	Fields    map[string]string
}

func (n *MetaNode) Kind() string { return "Meta" }

type HeadingNode struct {
	Line, Col int
	Level     int
	Attrs     map[string]string
	Children  []Node
}

func (n *HeadingNode) Kind() string { return "Heading" }

type ParagraphNode struct {
	Line, Col int
	Children  []Node
}

func (n *ParagraphNode) Kind() string { return "Paragraph" }

type HRNode struct {
	Line, Col int
}

func (n *HRNode) Kind() string { return "HR" }

type ListNode struct {
	Line, Col int
	Ordered   bool
	Items     []*ItemNode
}

func (n *ListNode) Kind() string { return "List" }

type ItemNode struct {
	Line, Col int
	Children  []Node
}

func (n *ItemNode) Kind() string { return "Item" }

type QuoteNode struct {
	Line, Col int
	Children  []Node
}

func (n *QuoteNode) Kind() string { return "Quote" }

type ImageNode struct {
	Line, Col int
	Src       string
	Alt       string
}

func (n *ImageNode) Kind() string { return "Image" }

const (
	InlineText   = "Text"
	InlineBold   = "Bold"
	InlineItalic = "Italic"
	InlineCode   = "InlineCode"
	InlineLink   = "Link"
)

type InlineNode struct {
	Line, Col int
	NodeKind  string
	Value     string
	URL       string
	Children  []Node
}

func (n *InlineNode) Kind() string { return n.NodeKind }

type CodeBlockNode struct {
	Line, Col int
	Language  string
	File      string
	RawCode   string
}

func (n *CodeBlockNode) Kind() string { return "CodeBlock" }

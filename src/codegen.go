package src

import (
	"fmt"
	"html"
	"strings"
)

// pageCSS is intentionally minimal — plain, readable, light-mode markdown
// styling with no visual flourishes. A future version of DocUP will let
// documents supply their own stylesheet in place of this default.
const pageCSS = `
body {
  margin: 0 auto;
  max-width: 760px;
  padding: 2.5rem 1.25rem 4rem;
  font-family: "Google Sans Flex", "Google Sans", -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 1.6;
  color: #24292f;
  background: #ffffff;
}
* {
  box-sizing: border-box;
}
h1, h2, h3, h4, h5, h6 {
  font-weight: 600;
  line-height: 1.25;
  margin-top: 1.6em;
  margin-bottom: 0.6em;
  overflow-wrap: break-word;
}
h1 { font-size: 2em; border-bottom: 1px solid #eaecef; padding-bottom: 0.3em; }
h2 { font-size: 1.5em; border-bottom: 1px solid #eaecef; padding-bottom: 0.3em; }
h3 { font-size: 1.25em; }
h4 { font-size: 1.1em; }
h5 { font-size: 1em; }
h6 { font-size: 0.9em; color: #57606a; }
p {
  margin: 0.8em 0;
  overflow-wrap: break-word;
}
strong { font-weight: 600; }
em { font-style: italic; }
a {
  color: #0969da;
  text-decoration: none;
  overflow-wrap: break-word;
}
a:hover { text-decoration: underline; }
code {
  background: #f6f8fa;
  padding: 0.15em 0.4em;
  border-radius: 4px;
  font-family: "Google Sans Code", ui-monospace, SFMono-Regular, Consolas, Menlo, monospace;
  font-size: 0.9em;
  overflow-wrap: break-word;
}
hr {
  border: none;
  border-top: 1px solid #eaecef;
  margin: 2em 0;
}
ul, ol {
  margin: 0.8em 0;
  padding-left: 1.6em;
}
li {
  margin: 0.25em 0;
  overflow-wrap: break-word;
}
li.task-item { list-style: none; margin-left: -1.6em; }
li.task-item input[type="checkbox"] {
  margin-right: 0.5em;
}
blockquote {
  margin: 1em 0;
  padding: 0 1em;
  color: #57606a;
  border-left: 0.25em solid #d0d7de;
  overflow-wrap: break-word;
}
blockquote blockquote {
  margin: 0.6em 0;
}
img {
  max-width: 100%;
  height: auto;
  border-radius: 4px;
}
.table-wrap {
  width: 100%;
  overflow-x: auto;
  margin: 1em 0;
  -webkit-overflow-scrolling: touch;
}
table {
  border-collapse: collapse;
  width: 100%;
  min-width: max-content;
  margin: 0;
}
th, td {
  border: 1px solid #d0d7de;
  padding: 0.5em 0.9em;
  text-align: left;
  overflow-wrap: break-word;
}
th {
  background: #f6f8fa;
  font-weight: 600;
  white-space: nowrap;
}
s { color: #57606a; }
.codeblock {
  margin: 1em 0;
  border: 1px solid #eaecef;
  border-radius: 6px;
  overflow: hidden;
}
.codeblock-header {
  display: flex;
  justify-content: space-between;
  gap: 0.75em;
  padding: 0.4em 0.9em;
  background: #f6f8fa;
  border-bottom: 1px solid #eaecef;
  font-size: 0.8em;
  color: #57606a;
  font-family: "Google Sans Code", ui-monospace, SFMono-Regular, Consolas, Menlo, monospace;
  overflow-x: auto;
  white-space: nowrap;
}
.codeblock pre {
  margin: 0;
  padding: 1em;
  overflow-x: auto;
  background: #f6f8fa;
}
.codeblock code {
  background: none;
  padding: 0;
  font-size: 0.9em;
  font-family: "Google Sans Code", ui-monospace, SFMono-Regular, Consolas, Menlo, monospace;
  overflow-wrap: normal;
  white-space: pre;
}
.tok-keyword { color: #cf222e; font-weight: 600; }
.tok-type    { color: #953800; }
.tok-string  { color: #0a3069; }
.tok-comment { color: #6e7781; font-style: italic; }
.tok-number  { color: #0550ae; }

@media (max-width: 640px) {
  body {
    padding: 1.5rem 1rem 3rem;
    font-size: 15px;
  }
  h1 { font-size: 1.6em; }
  h2 { font-size: 1.35em; }
  h3 { font-size: 1.15em; }
  th, td { padding: 0.4em 0.6em; }
}
`

// googleFontsLink loads Google Sans Flex (body text) and Google Sans Code
// (monospace/code) from Google Fonts' CDN. Both are listed with a full
// system-font fallback in pageCSS above, so rendering is never blocked on
// the network — if the CDN is unreachable, the browser silently falls
// back to the platform's default UI and monospace fonts.
const googleFontsLink = `<link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Google+Sans+Flex:wght@100..900&amp;family=Google+Sans+Code:wght@300..800&amp;display=swap" rel="stylesheet">`

// Generate produces a complete standalone HTML5 document from the AST.
func Generate(doc *DocumentNode) string {
	var body strings.Builder
	for _, block := range doc.Blocks {
		renderBlock(&body, block)
	}

	title := "DocUP Document"
	var metaTags strings.Builder
	if doc.Meta != nil {
		if t, ok := doc.Meta.Fields["title"]; ok {
			title = t
		}
		for _, key := range []string{"author", "version"} {
			if v, ok := doc.Meta.Fields[key]; ok {
				metaTags.WriteString(fmt.Sprintf(`  <meta name="%s" content="%s">`+"\n", key, html.EscapeString(v)))
			}
		}
	}

	return fmt.Sprintf(`<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>%s</title>
%s  %s
  <style>%s</style>
</head>
<body>
%s</body>
</html>
`, html.EscapeString(title), metaTags.String(), googleFontsLink, pageCSS, body.String())
}

func renderBlock(w *strings.Builder, n Node) {
	switch b := n.(type) {
	case *HeadingNode:
		idAttr := ""
		classAttr := ""
		if id, ok := b.Attrs["id"]; ok {
			idAttr = fmt.Sprintf(` id="%s"`, html.EscapeString(id))
		}
		if class, ok := b.Attrs["class"]; ok {
			classAttr = fmt.Sprintf(` class="%s"`, html.EscapeString(class))
		}
		fmt.Fprintf(w, "<h%d%s%s>", b.Level, idAttr, classAttr)
		renderInlines(w, b.Children)
		fmt.Fprintf(w, "</h%d>\n", b.Level)
	case *ParagraphNode:
		w.WriteString("<p>")
		renderInlines(w, b.Children)
		w.WriteString("</p>\n")
	case *CodeBlockNode:
		renderCodeBlock(w, b)
	case *HRNode:
		w.WriteString("<hr>\n")
	case *ListNode:
		renderList(w, b)
	case *QuoteNode:
		w.WriteString("<blockquote>\n")
		renderQuoteBody(w, b.Children)
		w.WriteString("</blockquote>\n")
	case *ImageNode:
		fmt.Fprintf(w, `<img src="%s" alt="%s">`+"\n", html.EscapeString(b.Src), html.EscapeString(b.Alt))
	case *TableNode:
		renderTable(w, b)
	}
}

// renderQuoteBody renders a quote's children, which may mix plain prose
// (wrapped in a <p>, same as a paragraph) with nested QuoteNode children
// (rendered as nested <blockquote> elements). Consecutive inline nodes
// are grouped into a single <p> rather than one per node.
// trimPendingInlineEdges trims a leading space from the first text node
// and a trailing space from the last text node in a run of inline
// children that's about to be flushed as its own <p> or <li> content —
// needed because a text run immediately before a nested block (list or
// quote) keeps its natural trailing space from the source, which would
// otherwise render right before the block's opening tag.
func trimPendingInlineEdges(nodes []Node) {
	if len(nodes) == 0 {
		return
	}
	if first, ok := nodes[0].(*InlineNode); ok && first.NodeKind == InlineText {
		first.Value = strings.TrimPrefix(first.Value, " ")
	}
	if last, ok := nodes[len(nodes)-1].(*InlineNode); ok && last.NodeKind == InlineText {
		last.Value = strings.TrimSuffix(last.Value, " ")
	}
}

func renderQuoteBody(w *strings.Builder, children []Node) {
	var pending []Node
	flush := func() {
		if len(pending) == 0 {
			return
		}
		trimPendingInlineEdges(pending)
		w.WriteString("  <p>")
		renderInlines(w, pending)
		w.WriteString("</p>\n")
		pending = nil
	}
	for _, c := range children {
		if nested, ok := c.(*QuoteNode); ok {
			flush()
			w.WriteString("  <blockquote>\n")
			renderQuoteBody(w, nested.Children)
			w.WriteString("  </blockquote>\n")
			continue
		}
		pending = append(pending, c)
	}
	flush()
}

func renderList(w *strings.Builder, l *ListNode) {
	tag := "ul"
	if l.Ordered {
		tag = "ol"
	}
	fmt.Fprintf(w, "<%s>\n", tag)
	for _, item := range l.Items {
		renderItem(w, item)
	}
	fmt.Fprintf(w, "</%s>\n", tag)
}

// renderItem renders a single <li>, handling three cases: a plain item
// (inline content only), a task item (item.Done != nil, rendered with a
// checkbox), and an item whose body contains a nested ListNode (rendered
// as a nested <ul>/<ol> inside the <li>).
func renderItem(w *strings.Builder, item *ItemNode) {
	if item.Done != nil {
		checked := ""
		if *item.Done {
			checked = " checked"
		}
		fmt.Fprintf(w, `  <li class="task-item"><input type="checkbox" disabled%s> `, checked)
	} else {
		w.WriteString("  <li>")
	}

	var inline []Node
	for _, c := range item.Children {
		if nested, ok := c.(*ListNode); ok {
			trimPendingInlineEdges(inline)
			renderInlines(w, inline)
			inline = nil
			renderList(w, nested)
			continue
		}
		inline = append(inline, c)
	}
	renderInlines(w, inline)
	w.WriteString("</li>\n")
}

func renderTable(w *strings.Builder, t *TableNode) {
	w.WriteString(`<div class="table-wrap">` + "\n")
	w.WriteString("<table>\n")
	for _, row := range t.Rows {
		w.WriteString("  <tr>\n")
		cellTag := "td"
		if row.Header {
			cellTag = "th"
		}
		for _, cell := range row.Cells {
			fmt.Fprintf(w, "    <%s>", cellTag)
			renderInlines(w, cell.Children)
			fmt.Fprintf(w, "</%s>\n", cellTag)
		}
		w.WriteString("  </tr>\n")
	}
	w.WriteString("</table>\n")
	w.WriteString("</div>\n")
}

func renderInlines(w *strings.Builder, children []Node) {
	for _, c := range children {
		inline, ok := c.(*InlineNode)
		if !ok {
			continue
		}
		switch inline.NodeKind {
		case InlineText:
			w.WriteString(html.EscapeString(inline.Value))
		case InlineBold:
			w.WriteString("<strong>")
			renderInlines(w, inline.Children)
			w.WriteString("</strong>")
		case InlineItalic:
			w.WriteString("<em>")
			renderInlines(w, inline.Children)
			w.WriteString("</em>")
		case InlineStrike:
			w.WriteString("<s>")
			renderInlines(w, inline.Children)
			w.WriteString("</s>")
		case InlineCode:
			w.WriteString("<code>")
			w.WriteString(html.EscapeString(inline.Value))
			w.WriteString("</code>")
		case InlineLink:
			fmt.Fprintf(w, `<a href="%s">`, html.EscapeString(inline.URL))
			renderInlines(w, inline.Children)
			w.WriteString("</a>")
		}
	}
}

func renderCodeBlock(w *strings.Builder, b *CodeBlockNode) {
	w.WriteString(`<div class="codeblock">` + "\n")
	if b.File != "" || b.Language != "" {
		w.WriteString(`  <div class="codeblock-header">` + "\n")
		if b.File != "" {
			fmt.Fprintf(w, "    <span>%s</span>\n", html.EscapeString(b.File))
		} else {
			w.WriteString("    <span></span>\n")
		}
		if b.Language != "" {
			fmt.Fprintf(w, `    <span>%s</span>`+"\n", html.EscapeString(b.Language))
		}
		w.WriteString("  </div>\n")
	}
	langClass := ""
	if b.Language != "" {
		langClass = fmt.Sprintf(" language-%s", html.EscapeString(b.Language))
	}
	highlighted := HighlightCode(b.RawCode, b.Language)
	fmt.Fprintf(w, "  <pre><code class=\"%s\">%s</code></pre>\n", strings.TrimSpace(langClass), highlighted)
	w.WriteString("</div>\n")
}


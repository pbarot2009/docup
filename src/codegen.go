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
h1, h2, h3, h4, h5, h6 {
  font-weight: 600;
  line-height: 1.25;
  margin-top: 1.6em;
  margin-bottom: 0.6em;
}
h1 { font-size: 2em; border-bottom: 1px solid #eaecef; padding-bottom: 0.3em; }
h2 { font-size: 1.5em; border-bottom: 1px solid #eaecef; padding-bottom: 0.3em; }
h3 { font-size: 1.25em; }
h4 { font-size: 1.1em; }
h5 { font-size: 1em; }
h6 { font-size: 0.9em; color: #57606a; }
p { margin: 0.8em 0; }
strong { font-weight: 600; }
em { font-style: italic; }
a { color: #0969da; text-decoration: none; }
a:hover { text-decoration: underline; }
code {
  background: #f6f8fa;
  padding: 0.15em 0.4em;
  border-radius: 4px;
  font-family: "Google Sans Code", ui-monospace, SFMono-Regular, Consolas, Menlo, monospace;
  font-size: 0.9em;
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
li { margin: 0.25em 0; }
blockquote {
  margin: 1em 0;
  padding: 0 1em;
  color: #57606a;
  border-left: 0.25em solid #d0d7de;
}
blockquote blockquote {
  margin: 0.6em 0;
}
img {
  max-width: 100%;
  height: auto;
  border-radius: 4px;
}
.codeblock {
  margin: 1em 0;
  border: 1px solid #eaecef;
  border-radius: 6px;
  overflow: hidden;
}
.codeblock-header {
  display: flex;
  justify-content: space-between;
  padding: 0.4em 0.9em;
  background: #f6f8fa;
  border-bottom: 1px solid #eaecef;
  font-size: 0.8em;
  color: #57606a;
  font-family: "Google Sans Code", ui-monospace, SFMono-Regular, Consolas, Menlo, monospace;
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
}
.tok-keyword { color: #cf222e; font-weight: 600; }
.tok-type    { color: #953800; }
.tok-string  { color: #0a3069; }
.tok-comment { color: #6e7781; font-style: italic; }
.tok-number  { color: #0550ae; }
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
		renderBlockquoteBody(w, b.Children)
		w.WriteString("</blockquote>\n")
	case *ImageNode:
		fmt.Fprintf(w, `<img src="%s" alt="%s">`+"\n", html.EscapeString(b.Src), html.EscapeString(b.Alt))
	}
}

// renderBlockquoteBody wraps quote content in a <p> the same way a
// paragraph would, since quote{} bodies are plain prose in Phase 1.
func renderBlockquoteBody(w *strings.Builder, children []Node) {
	w.WriteString("  <p>")
	renderInlines(w, children)
	w.WriteString("</p>\n")
}

func renderList(w *strings.Builder, l *ListNode) {
	tag := "ul"
	if l.Ordered {
		tag = "ol"
	}
	fmt.Fprintf(w, "<%s>\n", tag)
	for _, item := range l.Items {
		w.WriteString("  <li>")
		renderInlines(w, item.Children)
		w.WriteString("</li>\n")
	}
	fmt.Fprintf(w, "</%s>\n", tag)
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


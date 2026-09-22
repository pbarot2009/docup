use crate::ast::{
    BlockNode, CodeBlockNode, DocumentNode, InlineKind, InlineNode, ItemChild, ItemNode, ListNode,
    QuoteChild, TableNode, trim_inline_edges,
};
use crate::highlight::{escape_html, escape_html_into, highlight_code};

/// PAGE_CSS provides minimal, light-mode document styling[span_1](start_span)[span_1](end_span).
pub const PAGE_CSS: &str = r#"
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
"#;

/// GOOGLE_FONTS_LINK preloads Google Sans Flex and Google Sans Code fonts[span_2](start_span)[span_2](end_span).
pub const GOOGLE_FONTS_LINK: &str = r#"<link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Google+Sans+Flex:wght@100..900&amp;family=Google+Sans+Code:wght@300..800&amp;display=swap" rel="stylesheet">"#;

/// Generate produces a complete standalone HTML5 document from the AST[span_3](start_span)[span_3](end_span).
pub fn generate(doc: &DocumentNode) -> String {
    let mut body = String::new();
    for block in &doc.blocks {
        render_block(&mut body, block);
    }

    let mut title = "DocUP Document".to_string();
    let mut meta_tags = String::new();

    if let Some(ref meta) = doc.meta {
        if let Some(t) = meta.fields.get("title") {
            title = t.clone();
        }
        for key in ["author", "version"] {
            if let Some(v) = meta.fields.get(key) {
                meta_tags.push_str(&format!(
                    "  <meta name=\"{}\" content=\"{}\">\n",
                    key,
                    escape_html(v)
                ));
            }
        }
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{}</title>
{}  {}
  <style>{}</style>
</head>
<body>
{}</body>
</html>
"#,
        escape_html(&title),
        meta_tags,
        GOOGLE_FONTS_LINK,
        PAGE_CSS,
        body
    )
}

fn render_block(w: &mut String, block: &BlockNode) {
    match block {
        BlockNode::Heading(b) => {
            let id_attr = match b.attrs.get("id") {
                Some(id) => format!(" id=\"{}\"", escape_html(id)),
                None => String::new(),
            };
            let class_attr = match b.attrs.get("class") {
                Some(class) => format!(" class=\"{}\"", escape_html(class)),
                None => String::new(),
            };
            w.push_str(&format!("<h{}{}{}>", b.level, id_attr, class_attr));
            render_inlines(w, &b.children);
            w.push_str(&format!("</h{}>\n", b.level));
        }
        BlockNode::Paragraph(b) => {
            w.push_str("<p>");
            render_inlines(w, &b.children);
            w.push_str("</p>\n");
        }
        BlockNode::CodeBlock(b) => render_code_block(w, b),
        BlockNode::HR(_) => w.push_str("<hr>\n"),
        BlockNode::List(b) => render_list(w, b),
        BlockNode::Quote(b) => {
            w.push_str("<blockquote>\n");
            render_quote_body(w, &b.children);
            w.push_str("</blockquote>\n");
        }
        BlockNode::Image(b) => {
            w.push_str(&format!(
                "<img src=\"{}\" alt=\"{}\">\n",
                escape_html(&b.src),
                escape_html(&b.alt)
            ));
        }
        BlockNode::Table(b) => render_table(w, b),
    }
}

fn render_quote_body(w: &mut String, children: &[QuoteChild]) {
    let mut pending: Vec<InlineNode> = Vec::new();
    let mut flush = |w: &mut String, pending: &mut Vec<InlineNode>| {
        if pending.is_empty() {
            return;
        }
        trim_inline_edges(pending);
        w.push_str("  <p>");
        render_inlines(w, pending);
        w.push_str("</p>\n");
        pending.clear();
    };

    for c in children {
        match c {
            QuoteChild::Quote(nested) => {
                flush(w, &mut pending);
                w.push_str("  <blockquote>\n");
                render_quote_body(w, &nested.children);
                w.push_str("  </blockquote>\n");
            }
            QuoteChild::Inline(inline) => {
                pending.push(inline.clone());
            }
        }
    }
    flush(w, &mut pending);
}

fn render_list(w: &mut String, l: &ListNode) {
    let tag = if l.ordered { "ol" } else { "ul" };
    w.push_str(&format!("<{tag}>\n"));
    for item in &l.items {
        render_item(w, item);
    }
    w.push_str(&format!("</{tag}>\n"));
}

fn render_item(w: &mut String, item: &ItemNode) {
    if let Some(done) = item.done {
        let checked = if done { " checked" } else { "" };
        w.push_str(&format!(
            "  <li class=\"task-item\"><input type=\"checkbox\" disabled{checked}> "
        ));
    } else {
        w.push_str("  <li>");
    }

    let mut inline: Vec<InlineNode> = Vec::new();
    for c in &item.children {
        match c {
            ItemChild::List(nested) => {
                trim_inline_edges(&mut inline);
                render_inlines(w, &inline);
                inline.clear();
                render_list(w, nested);
            }
            ItemChild::Inline(node) => {
                inline.push(node.clone());
            }
        }
    }
    render_inlines(w, &inline);
    w.push_str("</li>\n");
}

fn render_table(w: &mut String, t: &TableNode) {
    w.push_str("<div class=\"table-wrap\">\n");
    w.push_str("<table>\n");
    for row in &t.rows {
        w.push_str("  <tr>\n");
        let cell_tag = if row.header { "th" } else { "td" };
        for cell in &row.cells {
            w.push_str(&format!("    <{cell_tag}>"));
            render_inlines(w, &cell.children);
            w.push_str(&format!("</{cell_tag}>\n"));
        }
        w.push_str("  </tr>\n");
    }
    w.push_str("</table>\n");
    w.push_str("</div>\n");
}

fn render_inlines(w: &mut String, children: &[InlineNode]) {
    for inline in children {
        match &inline.kind {
            InlineKind::Text(val) => {
                escape_html_into(val, w);
            }
            InlineKind::Bold(inner) => {
                w.push_str("<strong>");
                render_inlines(w, inner);
                w.push_str("</strong>");
            }
            InlineKind::Italic(inner) => {
                w.push_str("<em>");
                render_inlines(w, inner);
                w.push_str("</em>");
            }
            InlineKind::Strike(inner) => {
                w.push_str("<s>");
                render_inlines(w, inner);
                w.push_str("</s>");
            }
            InlineKind::Code(val) => {
                w.push_str("<code>");
                escape_html_into(val, w);
                w.push_str("</code>");
            }
            InlineKind::Link { url, children } => {
                w.push_str(&format!("<a href=\"{}\">", escape_html(url)));
                render_inlines(w, children);
                w.push_str("</a>");
            }
        }
    }
}

fn render_code_block(w: &mut String, b: &CodeBlockNode) {
    w.push_str("<div class=\"codeblock\">\n");
    if !b.file.is_empty() || !b.language.is_empty() {
        w.push_str("  <div class=\"codeblock-header\">\n");
        if !b.file.is_empty() {
            w.push_str(&format!("    <span>{}</span>\n", escape_html(&b.file)));
        } else {
            w.push_str("    <span></span>\n");
        }
        if !b.language.is_empty() {
            w.push_str(&format!("    <span>{}</span>\n", escape_html(&b.language)));
        }
        w.push_str("  </div>\n");
    }
    let lang_class = if !b.language.is_empty() {
        format!(" language-{}", escape_html(&b.language))
    } else {
        String::new()
    };
    let highlighted = highlight_code(&b.raw_code, &b.language);
    w.push_str(&format!(
        "  <pre><code class=\"{}\">{}</code></pre>\n",
        lang_class.trim_start(),
        highlighted
    ));
    w.push_str("</div>\n");
}

use std::collections::{HashMap, HashSet};

use crate::ast::{
    trim_inline_edges, BlockNode, CalloutKind, CodeBlockNode, DefListEntry, DefListNode,
    DetailsNode, DocumentNode, FigureNode, FootnoteDefNode, InlineKind, InlineNode, ItemChild,
    ItemNode, ListNode, QuoteChild, TableNode,
};
use crate::highlight::{escape_html, escape_html_into, highlight_code};
use crate::theme::ThemeKind;

pub const BASE_CSS: &str = r#"
* {
  box-sizing: border-box;
}

body {
  margin: 0 auto;
  max-width: 760px;
  padding: 2.5rem 1.25rem 4rem;
  font-family: var(--font-body, "Google Sans Flex", "Google Sans", -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif);
  font-size: 16px;
  line-height: 1.6;
  color: var(--text);
  background: var(--bg);
  transition: background-color 0.2s ease, color 0.2s ease;
}

h1, h2, h3, h4, h5, h6 {
  font-weight: 600;
  line-height: 1.25;
  margin-top: 1.6em;
  margin-bottom: 0.6em;
  overflow-wrap: break-word;
  color: var(--text);
  scroll-margin-top: 1em;
}
h1 { font-size: 2em; border-bottom: 1px solid var(--border); padding-bottom: 0.3em; }
h2 { font-size: 1.5em; border-bottom: 1px solid var(--border); padding-bottom: 0.3em; }
h3 { font-size: 1.25em; }
h4 { font-size: 1.1em; }
h5 { font-size: 1em; }
h6 { font-size: 0.9em; color: var(--text-muted); }

p {
  margin: 0.8em 0;
  overflow-wrap: break-word;
}
strong { font-weight: 600; }
em { font-style: italic; }
a {
  color: var(--link);
  text-decoration: none;
  overflow-wrap: break-word;
}
a:hover { text-decoration: underline; }

code {
  background: var(--code-bg);
  padding: 0.15em 0.4em;
  border-radius: 4px;
  font-family: var(--font-code, "Google Sans Code", ui-monospace, SFMono-Regular, Consolas, Menlo, monospace);
  font-size: 0.9em;
  overflow-wrap: break-word;
}

hr {
  border: none;
  border-top: 1px solid var(--border);
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
  color: var(--text-muted);
  border-left: 0.25em solid var(--border-muted);
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

.callout {
  margin: 1.2em 0;
  padding: 0.85em 1.2em;
  border-left: 4px solid;
  border-radius: 4px;
  background: var(--code-bg);
}
.callout-note { border-color: var(--callout-note); }
.callout-tip { border-color: var(--callout-tip); }
.callout-warning { border-color: var(--callout-warning); }
.callout-danger { border-color: var(--callout-danger); }
.callout-title {
  font-weight: 600;
  text-transform: uppercase;
  font-size: 0.75em;
  letter-spacing: 0.05em;
  margin-bottom: 0.4em;
}
.callout-note .callout-title { color: var(--callout-note); }
.callout-tip .callout-title { color: var(--callout-tip); }
.callout-warning .callout-title { color: var(--callout-warning); }
.callout-danger .callout-title { color: var(--callout-danger); }
.callout-body {
  overflow-wrap: break-word;
}

.math-block {
  margin: 1.2em 0;
  text-align: center;
  overflow-x: auto;
}

.toc {
  margin: 1.5em 0;
  padding: 1em 1.25em;
  background: var(--code-bg);
  border: 1px solid var(--border);
  border-radius: 6px;
}
.toc-title {
  font-weight: 600;
  margin-bottom: 0.5em;
  color: var(--text);
  font-size: 0.85em;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.toc ul {
  list-style: none;
  padding-left: 0;
  margin: 0;
}
.toc li {
  margin: 0.35em 0;
  font-size: 0.95em;
}
.toc li a {
  color: var(--text);
}
.toc li a:hover {
  color: var(--link);
  text-decoration: underline;
}
.toc li.toc-level-1 { font-weight: 600; }
.toc li.toc-level-2 { padding-left: 1.2em; }
.toc li.toc-level-3 { padding-left: 2.4em; }
.toc li.toc-level-4 { padding-left: 3.6em; }
.toc li.toc-level-5 { padding-left: 4.8em; }
.toc li.toc-level-6 { padding-left: 6em; }

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
  border: 1px solid var(--border-muted);
  padding: 0.5em 0.9em;
  text-align: left;
  overflow-wrap: break-word;
}
th {
  background: var(--code-bg);
  font-weight: 600;
  white-space: nowrap;
}
s { color: var(--text-muted); }

.codeblock {
  margin: 1.2em 0;
  border: 1px solid var(--border);
  border-radius: 6px;
  overflow: hidden;
}
.codeblock-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 0.75em;
  padding: 0.4em 0.9em;
  background: var(--code-bg);
  border-bottom: 1px solid var(--border);
  font-size: 0.8em;
  color: var(--text-muted);
  font-family: var(--font-code, "Google Sans Code", ui-monospace, SFMono-Regular, Consolas, Menlo, monospace);
}
.codeblock-actions {
  display: flex;
  align-items: center;
  gap: 0.75em;
}
.codeblock-copy-btn {
  background: transparent;
  border: 1px solid var(--border-muted);
  border-radius: 4px;
  color: var(--text-muted);
  padding: 0.15em 0.55em;
  font-size: 0.85em;
  font-family: inherit;
  cursor: pointer;
  transition: all 0.15s ease;
}
.codeblock-copy-btn:hover {
  color: var(--text);
  border-color: var(--text-muted);
  background: var(--border);
}
.codeblock pre {
  margin: 0;
  padding: 0;
  overflow-x: auto;
  background: var(--code-bg);
}
.codeblock code {
  display: grid;
  grid-template-columns: minmax(max-content, 100%);
  justify-items: stretch;
  box-sizing: content-box;
  min-width: 100%;
  background: none;
  padding: 0.85em 0;
  font-size: 0.9em;
  line-height: 1.5;
  font-family: var(--font-code, "Google Sans Code", ui-monospace, SFMono-Regular, Consolas, Menlo, monospace);
  overflow-wrap: normal;
  white-space: pre;
}
.code-line {
  display: block;
  box-sizing: border-box;
  min-height: 1.5em;
  line-height: 1.5;
  padding: 0 1em;
}
.code-line.highlighted-line {
  background: rgba(88, 166, 255, 0.15);
}
.line-numbers .code-line {
  counter-increment: line;
}
.line-numbers .code-line::before {
  content: counter(line);
  display: inline-block;
  width: 2.2em;
  padding-right: 0.8em;
  margin-right: 0.8em;
  color: var(--text-muted);
  text-align: right;
  user-select: none;
  border-right: 1px solid var(--border);
}

.footnote-ref {
  font-size: 0.75em;
  line-height: 0;
  vertical-align: super;
  padding-left: 0.1em;
}
.footnotes {
  margin-top: 3em;
  padding-top: 1em;
  font-size: 0.9em;
  color: var(--text-muted);
}
.footnotes ol {
  padding-left: 1.4em;
}
.footnotes li {
  margin: 0.5em 0;
}
.footnote-back {
  margin-left: 0.4em;
  text-decoration: none;
}

.tok-keyword { color: var(--tok-keyword); font-weight: 600; }
.tok-type    { color: var(--tok-type); }
.tok-string  { color: var(--tok-string); }
.tok-comment { color: var(--tok-comment); font-style: italic; }
.tok-number  { color: var(--tok-number); }


figure {
  margin: 1.2em 0;
}
figure img {
  display: block;
}
figcaption {
  margin-top: 0.4em;
  font-size: 0.9em;
  color: var(--text-muted);
}
dl {
  margin: 0.8em 0;
}
dt {
  font-weight: 600;
  margin-top: 0.6em;
}
dd {
  margin: 0.2em 0 0.6em 1.4em;
}
details {
  margin: 1em 0;
  padding: 0.6em 0.9em;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--code-bg);
}
details summary {
  cursor: pointer;
  font-weight: 600;
}
.quote-attr {
  margin-top: 0.4em;
  font-size: 0.9em;
  color: var(--text-muted);
}

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

pub const PAGE_CSS: &str = BASE_CSS;

pub const GOOGLE_FONTS_LINK: &str = r#"<link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Google+Sans+Flex:wght@100..900&amp;family=Google+Sans+Code:wght@300..800&amp;display=swap" rel="stylesheet">"#;

pub const KATEX_HEAD: &str = r#"  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.css">
  <script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.js"></script>
  <script defer src="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/contrib/auto-render.min.js" onload="renderMathInElement(document.body);"></script>
"#;

pub const COPY_SCRIPT: &str = r#"  <script>
    function copyCode(btn) {
      const code = btn.closest('.codeblock').querySelector('code').innerText;
      navigator.clipboard.writeText(code).then(() => {
        const orig = btn.innerText;
        btn.innerText = 'Copied!';
        setTimeout(() => { btn.innerText = orig; }, 2000);
      });
    }
  </script>
"#;

/// Compiles full theme CSS: typography stacks, color variables, base structure, and theme-specific rules.
pub fn build_page_css(theme: ThemeKind) -> String {
    let (body_font, code_font) = theme.font_stacks();
    let palette = theme.css_palette();
    let custom_rules = theme.custom_rules();

    format!(
        ":root {{\n  --font-body: {body_font};\n  --font-code: {code_font};\n}}\n{palette}\n{BASE_CSS}\n{custom_rules}"
    )
}

struct HeadingMeta {
    level: usize,
    id: String,
    title: String,
}

pub fn generate(doc: &DocumentNode, custom_css: Option<&str>) -> String {
    let heading_metas = collect_headings(doc);
    let (footnote_numbers, footnote_order) = collect_footnotes(doc);

    let mut body = String::new();
    let mut heading_idx = 0;
    for block in &doc.blocks {
        render_block(
            &mut body,
            block,
            &heading_metas,
            &mut heading_idx,
            &footnote_numbers,
        );
    }

    render_footnotes_section(&mut body, doc, &footnote_numbers, &footnote_order);

    let mut title = "DocUP Document".to_string();
    let mut lang = "en".to_string();
    let mut theme_attr = String::new();
    let mut meta_tags = String::new();
    let mut stylesheet_link = String::new();
    let mut theme_kind = ThemeKind::Default;

    if let Some(ref meta) = doc.meta {
        if let Some(t) = meta.fields.get("title") {
            title = t.clone();
        }
        if let Some(l) = meta.fields.get("lang") {
            lang = l.clone();
        }
        if let Some(theme) = meta.fields.get("theme") {
            theme_kind = ThemeKind::parse(theme);
            theme_attr = format!(" data-theme=\"{}\"", escape_html(theme.trim()));
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

        if let Some(desc) = meta.fields.get("description") {
            let escaped_desc = escape_html(desc);
            meta_tags.push_str(&format!(
                "  <meta name=\"description\" content=\"{escaped_desc}\">\n"
            ));
            meta_tags.push_str(&format!(
                "  <meta property=\"og:description\" content=\"{escaped_desc}\">\n"
            ));
            meta_tags.push_str(&format!(
                "  <meta name=\"twitter:description\" content=\"{escaped_desc}\">\n"
            ));
        }
        if let Some(keywords) = meta.fields.get("keywords") {
            meta_tags.push_str(&format!(
                "  <meta name=\"keywords\" content=\"{}\">\n",
                escape_html(keywords)
            ));
        }

        if let Some(canonical) = meta.fields.get("canonical") {
            let escaped_canonical = escape_html(canonical);
            meta_tags.push_str(&format!(
                "  <link rel=\"canonical\" href=\"{escaped_canonical}\">\n"
            ));
            meta_tags.push_str(&format!(
                "  <meta property=\"og:url\" content=\"{escaped_canonical}\">\n"
            ));
        }
        if let Some(img) = meta.fields.get("image") {
            let escaped_img = escape_html(img);
            meta_tags.push_str(&format!(
                "  <meta property=\"og:image\" content=\"{escaped_img}\">\n"
            ));
            meta_tags.push_str(&format!(
                "  <meta name=\"twitter:image\" content=\"{escaped_img}\">\n"
            ));
        }

        if let Some(sheet) = meta.fields.get("stylesheet") {
            stylesheet_link = format!(
                "  <link rel=\"stylesheet\" href=\"{}\">\n",
                escape_html(sheet)
            );
        }
    }

    let escaped_title = escape_html(&title);
    meta_tags.push_str(&format!(
        "  <meta property=\"og:title\" content=\"{escaped_title}\">\n"
    ));
    meta_tags.push_str("  <meta property=\"og:type\" content=\"article\">\n");
    meta_tags.push_str("  <meta name=\"twitter:card\" content=\"summary_large_image\">\n");
    meta_tags.push_str(&format!(
        "  <meta name=\"twitter:title\" content=\"{escaped_title}\">\n"
    ));
    meta_tags.push_str(&format!(
        "  <meta name=\"generator\" content=\"DocUP v{}\">\n",
        crate::cmd::VERSION
    ));

    let font_head_tags = theme_kind.font_head_tags();
    let page_css = build_page_css(theme_kind);

    let katex_tags = if doc_has_math(doc) { KATEX_HEAD } else { "" };

    let custom_style_tag = match custom_css {
        Some(css) if !css.trim().is_empty() => {
            format!("  <style id=\"docup-custom-css\">\n{css}\n  </style>\n")
        }
        _ => String::new(),
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="{lang}"{theme_attr}>
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{escaped_title}</title>
{meta_tags}{font_head_tags}
  <style>{page_css}</style>
{katex_tags}{stylesheet_link}{custom_style_tag}</head>
<body>
{body}{COPY_SCRIPT}</body>
</html>
"#
    )
}

fn collect_headings(doc: &DocumentNode) -> Vec<HeadingMeta> {
    let mut metas = Vec::new();
    let mut used_ids = HashSet::new();

    for block in &doc.blocks {
        if let BlockNode::Heading(h) = block {
            let title = inlines_to_plain_text(&h.children);
            let base_id = if let Some(id) = h.attrs.get("id") {
                id.clone()
            } else {
                slugify(&title)
            };
            let unique_id = unique_slug(&base_id, &mut used_ids);
            metas.push(HeadingMeta {
                level: h.level,
                id: unique_id,
                title,
            });
        }
    }
    metas
}

fn slugify(s: &str) -> String {
    let mut slug = String::new();
    let mut prev_dash = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if (c == ' ' || c == '-' || c == '_') && !prev_dash && !slug.is_empty() {
            slug.push('-');
            prev_dash = true;
        }
    }
    slug.trim_end_matches('-').to_string()
}

fn unique_slug(base: &str, used: &mut HashSet<String>) -> String {
    let base = if base.is_empty() { "section" } else { base };
    if used.insert(base.to_string()) {
        return base.to_string();
    }
    let mut count = 1;
    loop {
        let candidate = format!("{base}-{count}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
        count += 1;
    }
}

fn inlines_to_plain_text(inlines: &[InlineNode]) -> String {
    let mut out = String::new();
    for inline in inlines {
        match &inline.kind {
            InlineKind::Text(s) | InlineKind::Code(s) | InlineKind::Math(s) => out.push_str(s),
            InlineKind::Bold(children)
            | InlineKind::Italic(children)
            | InlineKind::Strike(children)
            | InlineKind::Link { children, .. } => out.push_str(&inlines_to_plain_text(children)),
            InlineKind::FootnoteRef(_) | InlineKind::Break => {}
        }
    }
    out
}

fn collect_footnotes(doc: &DocumentNode) -> (HashMap<String, usize>, Vec<String>) {
    let mut map = HashMap::new();
    let mut order = Vec::new();
    for block in &doc.blocks {
        collect_block_footnotes(block, &mut map, &mut order);
    }
    (map, order)
}

fn collect_block_footnotes(
    block: &BlockNode,
    map: &mut HashMap<String, usize>,
    order: &mut Vec<String>,
) {
    match block {
        BlockNode::Heading(h) => collect_inlines_footnotes(&h.children, map, order),
        BlockNode::Paragraph(p) => collect_inlines_footnotes(&p.children, map, order),
        BlockNode::Callout(c) => collect_inlines_footnotes(&c.children, map, order),
        BlockNode::Table(t) => {
            for row in &t.rows {
                for cell in &row.cells {
                    collect_inlines_footnotes(&cell.children, map, order);
                }
            }
        }
        BlockNode::List(l) => {
            for item in &l.items {
                for child in &item.children {
                    match child {
                        ItemChild::Inline(i) => collect_inline_footnotes(i, map, order),
                        ItemChild::List(nested) => {
                            collect_block_footnotes(&BlockNode::List(nested.clone()), map, order)
                        }
                    }
                }
            }
        }
        BlockNode::Quote(q) => {
            for child in &q.children {
                match child {
                    QuoteChild::Inline(i) => collect_inline_footnotes(i, map, order),
                    QuoteChild::Quote(nested) => {
                        collect_block_footnotes(&BlockNode::Quote(nested.clone()), map, order)
                    }
                }
            }
        }
        BlockNode::Details(d) => {
            collect_inlines_footnotes(&d.summary, map, order);
            collect_inlines_footnotes(&d.children, map, order);
        }
        BlockNode::DefList(d) => {
            for entry in &d.entries {
                match entry {
                    DefListEntry::Term { children, .. } | DefListEntry::Desc { children, .. } => {
                        collect_inlines_footnotes(children, map, order);
                    }
                }
            }
        }
        BlockNode::CodeBlock(_)
        | BlockNode::HR(_)
        | BlockNode::Image(_)
        | BlockNode::Figure(_)
        | BlockNode::Raw(_)
        | BlockNode::Math(_)
        | BlockNode::TOC(_)
        | BlockNode::Footnote(_)
        | BlockNode::Include(_) => {}
    }
}

fn collect_inlines_footnotes(
    inlines: &[InlineNode],
    map: &mut HashMap<String, usize>,
    order: &mut Vec<String>,
) {
    for inline in inlines {
        collect_inline_footnotes(inline, map, order);
    }
}

fn collect_inline_footnotes(
    inline: &InlineNode,
    map: &mut HashMap<String, usize>,
    order: &mut Vec<String>,
) {
    match &inline.kind {
        InlineKind::FootnoteRef(id) => {
            if !map.contains_key(id) {
                let num = map.len() + 1;
                map.insert(id.clone(), num);
                order.push(id.clone());
            }
        }
        InlineKind::Bold(children)
        | InlineKind::Italic(children)
        | InlineKind::Strike(children)
        | InlineKind::Link { children, .. } => {
            collect_inlines_footnotes(children, map, order);
        }
        InlineKind::Text(_) | InlineKind::Code(_) | InlineKind::Math(_) | InlineKind::Break => {}
    }
}

fn render_block(
    w: &mut String,
    block: &BlockNode,
    heading_metas: &[HeadingMeta],
    heading_idx: &mut usize,
    footnote_numbers: &HashMap<String, usize>,
) {
    match block {
        BlockNode::Heading(b) => {
            let id = if *heading_idx < heading_metas.len() {
                &heading_metas[*heading_idx].id
            } else {
                "section"
            };
            *heading_idx += 1;

            let class_attr = match b.attrs.get("class") {
                Some(class) => format!(" class=\"{}\"", escape_html(class)),
                None => String::new(),
            };
            w.push_str(&format!(
                "<h{} id=\"{}\"{}>",
                b.level,
                escape_html(id),
                class_attr
            ));
            render_inlines(w, &b.children, footnote_numbers);
            w.push_str(&format!("</h{}>\n", b.level));
        }
        BlockNode::Paragraph(b) => {
            w.push_str("<p>");
            render_inlines(w, &b.children, footnote_numbers);
            w.push_str("</p>\n");
        }
        BlockNode::CodeBlock(b) => render_code_block(w, b),
        BlockNode::HR(_) => w.push_str("<hr>\n"),
        BlockNode::List(b) => render_list(w, b, footnote_numbers),
        BlockNode::Quote(b) => render_quote(w, b, footnote_numbers),
        BlockNode::Image(b) => render_image(w, b),
        BlockNode::Table(b) => render_table(w, b, footnote_numbers),
        BlockNode::Callout(c) => {
            let kind_str = c.kind.as_str();
            let title = if !c.title.trim().is_empty() {
                escape_html(&c.title)
            } else {
                match c.kind {
                    CalloutKind::Note => "Note".to_string(),
                    CalloutKind::Tip => "Tip".to_string(),
                    CalloutKind::Warning => "Warning".to_string(),
                    CalloutKind::Danger => "Danger".to_string(),
                }
            };
            w.push_str(&format!("<div class=\"callout callout-{kind_str}\">\n"));
            w.push_str(&format!("  <div class=\"callout-title\">{title}</div>\n"));
            w.push_str("  <div class=\"callout-body\">");
            render_inlines(w, &c.children, footnote_numbers);
            w.push_str("</div>\n</div>\n");
        }
        BlockNode::Raw(r) => {
            w.push_str(&r.html);
            w.push('\n');
        }
        BlockNode::Math(m) => {
            w.push_str("<div class=\"math-block\">\\[\n");
            escape_html_into(&m.latex, w);
            w.push_str("\n\\]</div>\n");
        }
        BlockNode::TOC(_) => render_toc(w, heading_metas),
        BlockNode::Figure(f) => render_figure(w, f),
        BlockNode::DefList(d) => render_deflist(w, d, footnote_numbers),
        BlockNode::Details(d) => render_details(w, d, footnote_numbers),
        BlockNode::Footnote(_) | BlockNode::Include(_) => {}
    }
}

fn render_toc(w: &mut String, headings: &[HeadingMeta]) {
    if headings.is_empty() {
        return;
    }
    w.push_str("<nav class=\"toc\" aria-label=\"Table of contents\">\n");
    w.push_str("  <div class=\"toc-title\">Table of Contents</div>\n");
    w.push_str("  <ul>\n");
    for h in headings {
        w.push_str(&format!(
            "    <li class=\"toc-level-{}\"><a href=\"#{}\">{}</a></li>\n",
            h.level,
            escape_html(&h.id),
            escape_html(&h.title)
        ));
    }
    w.push_str("  </ul>\n");
    w.push_str("</nav>\n");
}

fn render_quote_body(
    w: &mut String,
    children: &[QuoteChild],
    footnote_numbers: &HashMap<String, usize>,
) {
    let mut pending: Vec<InlineNode> = Vec::new();
    let flush = |w: &mut String,
                 pending: &mut Vec<InlineNode>,
                 footnote_numbers: &HashMap<String, usize>| {
        if pending.is_empty() {
            return;
        }
        trim_inline_edges(pending);
        w.push_str("  <p>");
        render_inlines(w, pending, footnote_numbers);
        w.push_str("</p>\n");
        pending.clear();
    };

    for c in children {
        match c {
            QuoteChild::Quote(nested) => {
                flush(w, &mut pending, footnote_numbers);
                w.push_str("  <blockquote>\n");
                render_quote_body(w, &nested.children, footnote_numbers);
                w.push_str("  </blockquote>\n");
            }
            QuoteChild::Inline(inline) => {
                pending.push(inline.clone());
            }
        }
    }
    flush(w, &mut pending, footnote_numbers);
}

fn render_list(w: &mut String, l: &ListNode, footnote_numbers: &HashMap<String, usize>) {
    let tag = if l.ordered { "ol" } else { "ul" };
    if l.ordered && l.start > 1 {
        w.push_str(&format!("<{tag} start=\"{}\">\n", l.start));
    } else {
        w.push_str(&format!("<{tag}>\n"));
    }
    for item in &l.items {
        render_item(w, item, footnote_numbers);
    }
    w.push_str(&format!("</{tag}>\n"));
}

fn render_item(w: &mut String, item: &ItemNode, footnote_numbers: &HashMap<String, usize>) {
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
                render_inlines(w, &inline, footnote_numbers);
                inline.clear();
                render_list(w, nested, footnote_numbers);
            }
            ItemChild::Inline(node) => {
                inline.push(node.clone());
            }
        }
    }
    render_inlines(w, &inline, footnote_numbers);
    w.push_str("</li>\n");
}

fn render_table(w: &mut String, t: &TableNode, footnote_numbers: &HashMap<String, usize>) {
    w.push_str("<div class=\"table-wrap\">\n");
    w.push_str("<table>\n");
    if !t.caption.is_empty() {
        w.push_str(&format!(
            "  <caption>{}</caption>\n",
            escape_html(&t.caption)
        ));
    }
    for row in &t.rows {
        w.push_str("  <tr>\n");
        let cell_tag = if row.header { "th" } else { "td" };
        for cell in &row.cells {
            let mut attrs = String::new();
            if cell.colspan > 1 {
                attrs.push_str(&format!(" colspan=\"{}\"", cell.colspan));
            }
            if cell.rowspan > 1 {
                attrs.push_str(&format!(" rowspan=\"{}\"", cell.rowspan));
            }
            w.push_str(&format!("    <{cell_tag}{attrs}>"));
            render_inlines(w, &cell.children, footnote_numbers);
            w.push_str(&format!("</{cell_tag}>\n"));
        }
        w.push_str("  </tr>\n");
    }
    w.push_str("</table>\n");
    w.push_str("</div>\n");
}

fn render_inlines(
    w: &mut String,
    children: &[InlineNode],
    footnote_numbers: &HashMap<String, usize>,
) {
    for inline in children {
        match &inline.kind {
            InlineKind::Text(val) => {
                escape_html_into(val, w);
            }
            InlineKind::Bold(inner) => {
                w.push_str("<strong>");
                render_inlines(w, inner, footnote_numbers);
                w.push_str("</strong>");
            }
            InlineKind::Italic(inner) => {
                w.push_str("<em>");
                render_inlines(w, inner, footnote_numbers);
                w.push_str("</em>");
            }
            InlineKind::Strike(inner) => {
                w.push_str("<s>");
                render_inlines(w, inner, footnote_numbers);
                w.push_str("</s>");
            }
            InlineKind::Code(val) => {
                w.push_str("<code>");
                escape_html_into(val, w);
                w.push_str("</code>");
            }
            InlineKind::Link { url, children } => {
                w.push_str(&format!("<a href=\"{}\">", escape_html(url)));
                render_inlines(w, children, footnote_numbers);
                w.push_str("</a>");
            }
            InlineKind::Math(val) => {
                w.push_str("<span class=\"math-inline\">\\(");
                escape_html_into(val, w);
                w.push_str("\\)</span>");
            }
            InlineKind::Break => {
                w.push_str("<br>");
            }
            InlineKind::FootnoteRef(id) => {
                let num = footnote_numbers.get(id).copied().unwrap_or(1);
                let escaped_id = escape_html(id);
                w.push_str(&format!(
                    "<sup class=\"footnote-ref\"><a href=\"#fn-{escaped_id}\" id=\"fnref-{escaped_id}\">[{num}]</a></sup>"
                ));
            }
        }
    }
}

fn render_code_block(w: &mut String, b: &CodeBlockNode) {
    w.push_str("<div class=\"codeblock\">\n");
    w.push_str("  <div class=\"codeblock-header\">\n");
    if !b.file.is_empty() {
        w.push_str(&format!("    <span>{}</span>\n", escape_html(&b.file)));
    } else {
        w.push_str("    <span></span>\n");
    }
    w.push_str("    <div class=\"codeblock-actions\">\n");
    if !b.language.is_empty() {
        w.push_str(&format!(
            "      <span>{}</span>\n",
            escape_html(&b.language)
        ));
    }
    w.push_str("      <button type=\"button\" class=\"codeblock-copy-btn\" onclick=\"copyCode(this)\" title=\"Copy code\">Copy</button>\n");
    w.push_str("    </div>\n");
    w.push_str("  </div>\n");

    let mut classes = Vec::new();
    if !b.language.is_empty() {
        classes.push(format!("language-{}", escape_html(&b.language)));
    }
    if b.line_numbers {
        classes.push("line-numbers".to_string());
    }
    let class_attr = if !classes.is_empty() {
        format!(" class=\"{}\"", classes.join(" "))
    } else {
        String::new()
    };

    let highlighted = highlight_code(&b.raw_code, &b.language);
    let wrapped = wrap_code_lines(&highlighted, b.line_numbers, &b.highlight_lines);
    w.push_str(&format!(
        "  <pre><code{class_attr}>{wrapped}</code></pre>\n"
    ));
    w.push_str("</div>\n");
}

fn wrap_code_lines(highlighted: &str, line_numbers: bool, highlight_lines: &[usize]) -> String {
    if !line_numbers && highlight_lines.is_empty() {
        return highlighted.to_string();
    }

    let raw_lines: Vec<&str> = highlighted.split('\n').collect();
    let mut lines: Vec<&str> = raw_lines
        .iter()
        .map(|l| l.strip_suffix('\r').unwrap_or(l))
        .collect();

    // Remove any trailing empty line slice generated by a terminal newline
    while lines.len() > 1 && lines.last().map_or(false, |l| l.trim().is_empty()) {
        lines.pop();
    }

    let mut out = String::with_capacity(highlighted.len() * 2);
    let mut open_tags: Vec<String> = Vec::new();

    for (idx, line) in lines.iter().enumerate() {
        let line_num = idx + 1;
        let is_highlighted = highlight_lines.contains(&line_num);
        let hl_class = if is_highlighted {
            " highlighted-line"
        } else {
            ""
        };

        out.push_str("<span class=\"code-line");
        out.push_str(hl_class);
        out.push_str("\">");

        for tag in &open_tags {
            out.push_str(tag);
        }

        let mut i = 0;
        let bytes = line.as_bytes();
        while i < bytes.len() {
            if bytes[i] == b'<' {
                if i + 1 < bytes.len() && bytes[i + 1] == b'/' {
                    if let Some(close_end) = line[i..].find('>') {
                        open_tags.pop();
                        i += close_end + 1;
                        continue;
                    }
                } else if line[i..].starts_with("<span") {
                    if let Some(open_end) = line[i..].find('>') {
                        let tag = &line[i..i + open_end + 1];
                        open_tags.push(tag.to_string());
                        i += open_end + 1;
                        continue;
                    }
                }
            }
            i += 1;
        }

        out.push_str(line);

        for _ in 0..open_tags.len() {
            out.push_str("</span>");
        }

        out.push_str("</span>");
    }
    out
}

fn render_footnotes_section(
    w: &mut String,
    doc: &DocumentNode,
    footnote_numbers: &HashMap<String, usize>,
    footnote_order: &[String],
) {
    let mut defs: HashMap<String, &FootnoteDefNode> = HashMap::new();
    for block in &doc.blocks {
        if let BlockNode::Footnote(f) = block {
            defs.insert(f.id.clone(), f);
        }
    }

    if defs.is_empty() {
        return;
    }

    w.push_str("<section class=\"footnotes\" aria-label=\"Footnotes\">\n");
    w.push_str("  <hr>\n");
    w.push_str("  <ol>\n");

    let mut rendered_ids = HashSet::new();
    for id in footnote_order {
        if let Some(def) = defs.get(id) {
            rendered_ids.insert(id.clone());
            let escaped_id = escape_html(id);
            w.push_str(&format!("    <li id=\"fn-{escaped_id}\">"));
            render_inlines(w, &def.children, footnote_numbers);
            w.push_str(&format!(
                " <a href=\"#fnref-{escaped_id}\" class=\"footnote-back\" aria-label=\"Back to reference\">↩</a></li>\n"
            ));
        }
    }

    for (id, def) in &defs {
        if !rendered_ids.contains(id) {
            let escaped_id = escape_html(id);
            w.push_str(&format!("    <li id=\"fn-{escaped_id}\">"));
            render_inlines(w, &def.children, footnote_numbers);
            w.push_str(&format!(
                " <a href=\"#fnref-{escaped_id}\" class=\"footnote-back\" aria-label=\"Back to reference\">↩</a></li>\n"
            ));
        }
    }

    w.push_str("  </ol>\n");
    w.push_str("</section>\n");
}

fn doc_has_math(doc: &DocumentNode) -> bool {
    doc.blocks.iter().any(block_has_math)
}

fn block_has_math(block: &BlockNode) -> bool {
    match block {
        BlockNode::Math(_) => true,
        BlockNode::Heading(h) => inlines_have_math(&h.children),
        BlockNode::Paragraph(p) => inlines_have_math(&p.children),
        BlockNode::Callout(c) => inlines_have_math(&c.children),
        BlockNode::Footnote(f) => inlines_have_math(&f.children),
        BlockNode::Table(t) => t
            .rows
            .iter()
            .any(|r| r.cells.iter().any(|c| inlines_have_math(&c.children))),
        BlockNode::List(l) => list_has_math(l),
        BlockNode::Quote(q) => quote_has_math(&q.children),
        BlockNode::DefList(d) => d.entries.iter().any(|e| match e {
            DefListEntry::Term { children, .. } | DefListEntry::Desc { children, .. } => {
                inlines_have_math(children)
            }
        }),
        BlockNode::Details(d) => inlines_have_math(&d.summary) || inlines_have_math(&d.children),
        BlockNode::CodeBlock(_)
        | BlockNode::HR(_)
        | BlockNode::Image(_)
        | BlockNode::Figure(_)
        | BlockNode::Raw(_)
        | BlockNode::TOC(_)
        | BlockNode::Include(_) => false,
    }
}

fn list_has_math(l: &ListNode) -> bool {
    for item in &l.items {
        for child in &item.children {
            match child {
                ItemChild::Inline(i) => {
                    if inline_has_math(i) {
                        return true;
                    }
                }
                ItemChild::List(nested) => {
                    if list_has_math(nested) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn quote_has_math(children: &[QuoteChild]) -> bool {
    for child in children {
        match child {
            QuoteChild::Inline(i) => {
                if inline_has_math(i) {
                    return true;
                }
            }
            QuoteChild::Quote(nested) => {
                if quote_has_math(&nested.children) {
                    return true;
                }
            }
        }
    }
    false
}

fn inlines_have_math(inlines: &[InlineNode]) -> bool {
    inlines.iter().any(inline_has_math)
}

fn inline_has_math(inline: &InlineNode) -> bool {
    match &inline.kind {
        InlineKind::Math(_) => true,
        InlineKind::Bold(children)
        | InlineKind::Italic(children)
        | InlineKind::Strike(children)
        | InlineKind::Link { children, .. } => inlines_have_math(children),
        InlineKind::Text(_)
        | InlineKind::Code(_)
        | InlineKind::FootnoteRef(_)
        | InlineKind::Break => false,
    }
}

fn render_quote(
    w: &mut String,
    q: &crate::ast::QuoteNode,
    footnote_numbers: &HashMap<String, usize>,
) {
    if !q.cite.is_empty() {
        w.push_str(&format!("<blockquote cite=\"{}\">\n", escape_html(&q.cite)));
    } else {
        w.push_str("<blockquote>\n");
    }
    render_quote_body(w, &q.children, footnote_numbers);
    if !q.author.is_empty() || !q.cite.is_empty() {
        w.push_str("  <p class=\"quote-attr\">");
        if !q.author.is_empty() {
            escape_html_into(&q.author, w);
        }
        if !q.cite.is_empty() {
            if !q.author.is_empty() {
                w.push_str(" \u{2014} ");
            }
            w.push_str(&format!(
                "<a href=\"{}\">{}</a>",
                escape_html(&q.cite),
                escape_html(&q.cite)
            ));
        }
        w.push_str("</p>\n");
    }
    w.push_str("</blockquote>\n");
}

fn render_image(w: &mut String, b: &crate::ast::ImageNode) {
    let img = format!(
        "<img src=\"{}\" alt=\"{}\">",
        escape_html(&b.src),
        escape_html(&b.alt)
    );
    if b.href.is_empty() {
        w.push_str(&img);
        w.push('\n');
    } else {
        w.push_str(&format!("<a href=\"{}\">{img}</a>\n", escape_html(&b.href)));
    }
}

fn render_figure(w: &mut String, f: &FigureNode) {
    w.push_str("<figure>\n");
    w.push_str(&format!(
        "  <img src=\"{}\" alt=\"{}\">\n",
        escape_html(&f.src),
        escape_html(&f.alt)
    ));
    if !f.caption.is_empty() {
        w.push_str(&format!(
            "  <figcaption>{}</figcaption>\n",
            escape_html(&f.caption)
        ));
    }
    w.push_str("</figure>\n");
}

fn render_deflist(w: &mut String, d: &DefListNode, footnote_numbers: &HashMap<String, usize>) {
    w.push_str("<dl>\n");
    for entry in &d.entries {
        match entry {
            DefListEntry::Term { children, .. } => {
                w.push_str("  <dt>");
                render_inlines(w, children, footnote_numbers);
                w.push_str("</dt>\n");
            }
            DefListEntry::Desc { children, .. } => {
                w.push_str("  <dd>");
                render_inlines(w, children, footnote_numbers);
                w.push_str("</dd>\n");
            }
        }
    }
    w.push_str("</dl>\n");
}

fn render_details(w: &mut String, d: &DetailsNode, footnote_numbers: &HashMap<String, usize>) {
    if d.open {
        w.push_str("<details open>\n");
    } else {
        w.push_str("<details>\n");
    }
    w.push_str("  <summary>");
    render_inlines(w, &d.summary, footnote_numbers);
    w.push_str("</summary>\n");
    if !d.children.is_empty() {
        w.push_str("  <p>");
        render_inlines(w, &d.children, footnote_numbers);
        w.push_str("</p>\n");
    }
    w.push_str("</details>\n");
}

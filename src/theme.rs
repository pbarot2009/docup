#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeKind {
    Textbook,
    Default,
    Sepia,
    Nord,
    Solarized,
}

impl ThemeKind {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "textbook" | "latex" | "academic" | "book" => ThemeKind::Textbook,
            "sepia" | "warm" | "editorial" => ThemeKind::Sepia,
            "nord" | "arctic" => ThemeKind::Nord,
            "solarized" => ThemeKind::Solarized,
            _ => ThemeKind::Default,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ThemeKind::Textbook => "textbook",
            ThemeKind::Default => "default",
            ThemeKind::Sepia => "sepia",
            ThemeKind::Nord => "nord",
            ThemeKind::Solarized => "solarized",
        }
    }

    /// Generates head tags for preconnecting and loading fonts needed by this theme.
    pub fn font_head_tags(&self) -> &'static str {
        match self {
            ThemeKind::Textbook => {
                r#"  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Literata:ital,opsz,wght@0,7..72,300..800;1,7..72,300..800&display=swap" rel="stylesheet">
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@fontsource/cascadia-code/index.css">"#
            }
            ThemeKind::Default => {
                r#"  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Google+Sans+Flex:wght@100..900&family=Google+Sans+Code:wght@300..800&display=swap" rel="stylesheet">"#
            }
            ThemeKind::Sepia => {
                r#"  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Merriweather:ital,wght@0,300;0,400;0,700;1,300;1,400&family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet">"#
            }
            ThemeKind::Nord => {
                r#"  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet">"#
            }
            ThemeKind::Solarized => {
                r#"  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Fira+Sans:ital,wght@0,300;0,400;0,500;0,600;1,400&family=Fira+Code:wght@400;500;600&display=swap" rel="stylesheet">"#
            }
        }
    }

    /// Font stacks for body prose and monospace blocks.
    pub fn font_stacks(&self) -> (&'static str, &'static str) {
        match self {
            ThemeKind::Textbook => (
                r#""Literata", "Latin Modern Roman", "Computer Modern", Georgia, "Times New Roman", serif"#,
                r#""Cascadia Code", ui-monospace, SFMono-Regular, Consolas, monospace"#,
            ),
            ThemeKind::Default => (
                r#""Google Sans Flex", "Google Sans", -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif"#,
                r#""Google Sans Code", ui-monospace, SFMono-Regular, Consolas, Menlo, monospace"#,
            ),
            ThemeKind::Sepia => (
                r#""Merriweather", Georgia, "Times New Roman", serif"#,
                r#""JetBrains Mono", ui-monospace, SFMono-Regular, Consolas, monospace"#,
            ),
            ThemeKind::Nord => (
                r#""Inter", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif"#,
                r#""JetBrains Mono", ui-monospace, SFMono-Regular, Consolas, monospace"#,
            ),
            ThemeKind::Solarized => (
                r#""Fira Sans", -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif"#,
                r#""Fira Code", ui-monospace, SFMono-Regular, Consolas, monospace"#,
            ),
        }
    }

    /// Generates CSS color variable definitions for light and dark schemes.
    pub fn css_palette(&self) -> &'static str {
        match self {
            ThemeKind::Textbook => {
                r#"
:root {
  --bg: #fcfbf9;
  --text: #1b1b1b;
  --border: #dcd7cc;
  --border-muted: #e8e3d8;
  --link: #8b151b;
  --code-bg: #f3efe6;
  --text-muted: #6b665f;
  --tok-keyword: #8b151b;
  --tok-type: #7d4e2d;
  --tok-string: #2b5b3f;
  --tok-comment: #7d7d7d;
  --tok-number: #185a9d;
  --callout-note: #415a77;
  --callout-tip: #2c6e49;
  --callout-warning: #a44a04;
  --callout-danger: #8b151b;
}

@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --bg: #1c1b1a;
    --text: #ebe6df;
    --border: #3d3a36;
    --border-muted: #2d2b28;
    --link: #e57373;
    --code-bg: #272523;
    --text-muted: #9e978e;
    --tok-keyword: #e57373;
    --tok-type: #ffb74d;
    --tok-string: #81c784;
    --tok-comment: #9e978e;
    --tok-number: #64b5f6;
    --callout-note: #90caf9;
    --callout-tip: #81c784;
    --callout-warning: #ffb74d;
    --callout-danger: #e57373;
  }
}

:root[data-theme="dark"] {
  --bg: #1c1b1a;
  --text: #ebe6df;
  --border: #3d3a36;
  --border-muted: #2d2b28;
  --link: #e57373;
  --code-bg: #272523;
  --text-muted: #9e978e;
  --tok-keyword: #e57373;
  --tok-type: #ffb74d;
  --tok-string: #81c784;
  --tok-comment: #9e978e;
  --tok-number: #64b5f6;
  --callout-note: #90caf9;
  --callout-tip: #81c784;
  --callout-warning: #ffb74d;
  --callout-danger: #e57373;
}

:root[data-theme="light"] {
  --bg: #fcfbf9;
  --text: #1b1b1b;
  --border: #dcd7cc;
  --border-muted: #e8e3d8;
  --link: #8b151b;
  --code-bg: #f3efe6;
  --text-muted: #6b665f;
  --tok-keyword: #8b151b;
  --tok-type: #7d4e2d;
  --tok-string: #2b5b3f;
  --tok-comment: #7d7d7d;
  --tok-number: #185a9d;
  --callout-note: #415a77;
  --callout-tip: #2c6e49;
  --callout-warning: #a44a04;
  --callout-danger: #8b151b;
}
"#
            }
            ThemeKind::Default => {
                r#"
:root {
  --bg: #ffffff;
  --text: #24292f;
  --border: #eaecef;
  --border-muted: #d0d7de;
  --link: #0969da;
  --code-bg: #f6f8fa;
  --text-muted: #57606a;
  --tok-keyword: #cf222e;
  --tok-type: #953800;
  --tok-string: #0a3069;
  --tok-comment: #6e7781;
  --tok-number: #0550ae;
  --callout-note: #0969da;
  --callout-tip: #1a7f37;
  --callout-warning: #9a6700;
  --callout-danger: #cf222e;
}

@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --bg: #0d1117;
    --text: #c9d1d9;
    --border: #30363d;
    --border-muted: #21262d;
    --link: #58a6ff;
    --code-bg: #161b22;
    --text-muted: #8b949e;
    --tok-keyword: #ff7b72;
    --tok-type: #ffa657;
    --tok-string: #a5d6ff;
    --tok-comment: #8b949e;
    --tok-number: #79c0ff;
    --callout-note: #58a6ff;
    --callout-tip: #3fb950;
    --callout-warning: #d29922;
    --callout-danger: #f85149;
  }
}

:root[data-theme="dark"] {
  --bg: #0d1117;
  --text: #c9d1d9;
  --border: #30363d;
  --border-muted: #21262d;
  --link: #58a6ff;
  --code-bg: #161b22;
  --text-muted: #8b949e;
  --tok-keyword: #ff7b72;
  --tok-type: #ffa657;
  --tok-string: #a5d6ff;
  --tok-comment: #8b949e;
  --tok-number: #79c0ff;
  --callout-note: #58a6ff;
  --callout-tip: #3fb950;
  --callout-warning: #d29922;
  --callout-danger: #f85149;
}

:root[data-theme="light"] {
  --bg: #ffffff;
  --text: #24292f;
  --border: #eaecef;
  --border-muted: #d0d7de;
  --link: #0969da;
  --code-bg: #f6f8fa;
  --text-muted: #57606a;
  --tok-keyword: #cf222e;
  --tok-type: #953800;
  --tok-string: #0a3069;
  --tok-comment: #6e7781;
  --tok-number: #0550ae;
  --callout-note: #0969da;
  --callout-tip: #1a7f37;
  --callout-warning: #9a6700;
  --callout-danger: #cf222e;
}
"#
            }
            ThemeKind::Sepia => {
                r#"
:root {
  --bg: #f7f1e5;
  --text: #2f2723;
  --border: #dfd4c2;
  --border-muted: #ece3d2;
  --link: #a0522d;
  --code-bg: #ede4d3;
  --text-muted: #6e6259;
  --tok-keyword: #8c2d19;
  --tok-type: #965313;
  --tok-string: #3b663b;
  --tok-comment: #8c7b70;
  --tok-number: #2d5a88;
  --callout-note: #416788;
  --callout-tip: #3c7d4e;
  --callout-warning: #b06216;
  --callout-danger: #a52822;
}

@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --bg: #231e1a;
    --text: #d8cebe;
    --border: #3d352e;
    --border-muted: #2e2621;
    --link: #e28e47;
    --code-bg: #2b2520;
    --text-muted: #95897c;
    --tok-keyword: #e57368;
    --tok-type: #e09f58;
    --tok-string: #8cb88c;
    --tok-comment: #95897c;
    --tok-number: #6fa6de;
    --callout-note: #6fa6de;
    --callout-tip: #79bf8a;
    --callout-warning: #e28e47;
    --callout-danger: #e57368;
  }
}

:root[data-theme="dark"] {
  --bg: #231e1a;
  --text: #d8cebe;
  --border: #3d352e;
  --border-muted: #2e2621;
  --link: #e28e47;
  --code-bg: #2b2520;
  --text-muted: #95897c;
  --tok-keyword: #e57368;
  --tok-type: #e09f58;
  --tok-string: #8cb88c;
  --tok-comment: #95897c;
  --tok-number: #6fa6de;
  --callout-note: #6fa6de;
  --callout-tip: #79bf8a;
  --callout-warning: #e28e47;
  --callout-danger: #e57368;
}

:root[data-theme="light"] {
  --bg: #f7f1e5;
  --text: #2f2723;
  --border: #dfd4c2;
  --border-muted: #ece3d2;
  --link: #a0522d;
  --code-bg: #ede4d3;
  --text-muted: #6e6259;
  --tok-keyword: #8c2d19;
  --tok-type: #965313;
  --tok-string: #3b663b;
  --tok-comment: #8c7b70;
  --tok-number: #2d5a88;
  --callout-note: #416788;
  --callout-tip: #3c7d4e;
  --callout-warning: #b06216;
  --callout-danger: #a52822;
}
"#
            }
            ThemeKind::Nord => {
                r#"
:root {
  --bg: #eceff4;
  --text: #2e3440;
  --border: #d8dee9;
  --border-muted: #e5e9f0;
  --link: #5e81ac;
  --code-bg: #e5e9f0;
  --text-muted: #4c566a;
  --tok-keyword: #bf616a;
  --tok-type: #d08770;
  --tok-string: #a3be8c;
  --tok-comment: #616e88;
  --tok-number: #b48ead;
  --callout-note: #5e81ac;
  --callout-tip: #a3be8c;
  --callout-warning: #ebcb8b;
  --callout-danger: #bf616a;
}

@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --bg: #2e3440;
    --text: #eceff4;
    --border: #434c5e;
    --border-muted: #3b4252;
    --link: #88c0d0;
    --code-bg: #3b4252;
    --text-muted: #d8dee9;
    --tok-keyword: #bf616a;
    --tok-type: #d08770;
    --tok-string: #a3be8c;
    --tok-comment: #7b88a1;
    --tok-number: #b48ead;
    --callout-note: #88c0d0;
    --callout-tip: #a3be8c;
    --callout-warning: #ebcb8b;
    --callout-danger: #bf616a;
  }
}

:root[data-theme="dark"] {
  --bg: #2e3440;
  --text: #eceff4;
  --border: #434c5e;
  --border-muted: #3b4252;
  --link: #88c0d0;
  --code-bg: #3b4252;
  --text-muted: #d8dee9;
  --tok-keyword: #bf616a;
  --tok-type: #d08770;
  --tok-string: #a3be8c;
  --tok-comment: #7b88a1;
  --tok-number: #b48ead;
  --callout-note: #88c0d0;
  --callout-tip: #a3be8c;
  --callout-warning: #ebcb8b;
  --callout-danger: #bf616a;
}

:root[data-theme="light"] {
  --bg: #eceff4;
  --text: #2e3440;
  --border: #d8dee9;
  --border-muted: #e5e9f0;
  --link: #5e81ac;
  --code-bg: #e5e9f0;
  --text-muted: #4c566a;
  --tok-keyword: #bf616a;
  --tok-type: #d08770;
  --tok-string: #a3be8c;
  --tok-comment: #616e88;
  --tok-number: #b48ead;
  --callout-note: #5e81ac;
  --callout-tip: #a3be8c;
  --callout-warning: #ebcb8b;
  --callout-danger: #bf616a;
}
"#
            }
            ThemeKind::Solarized => {
                r#"
:root {
  --bg: #fdf6e3;
  --text: #586e75;
  --border: #eee8d5;
  --border-muted: #e0d7be;
  --link: #268bd2;
  --code-bg: #eee8d5;
  --text-muted: #839496;
  --tok-keyword: #dc322f;
  --tok-type: #b58900;
  --tok-string: #2aa198;
  --tok-comment: #93a1a1;
  --tok-number: #d33682;
  --callout-note: #268bd2;
  --callout-tip: #859900;
  --callout-warning: #b58900;
  --callout-danger: #dc322f;
}

@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --bg: #002b36;
    --text: #839496;
    --border: #073642;
    --border-muted: #0b4352;
    --link: #2aa198;
    --code-bg: #073642;
    --text-muted: #586e75;
    --tok-keyword: #dc322f;
    --tok-type: #b58900;
    --tok-string: #2aa198;
    --tok-comment: #586e75;
    --tok-number: #d33682;
    --callout-note: #268bd2;
    --callout-tip: #859900;
    --callout-warning: #b58900;
    --callout-danger: #dc322f;
  }
}

:root[data-theme="dark"] {
  --bg: #002b36;
  --text: #839496;
  --border: #073642;
  --border-muted: #0b4352;
  --link: #2aa198;
  --code-bg: #073642;
  --text-muted: #586e75;
  --tok-keyword: #dc322f;
  --tok-type: #b58900;
  --tok-string: #2aa198;
  --tok-comment: #586e75;
  --tok-number: #d33682;
  --callout-note: #268bd2;
  --callout-tip: #859900;
  --callout-warning: #b58900;
  --callout-danger: #dc322f;
}

:root[data-theme="light"] {
  --bg: #fdf6e3;
  --text: #586e75;
  --border: #eee8d5;
  --border-muted: #e0d7be;
  --link: #268bd2;
  --code-bg: #eee8d5;
  --text-muted: #839496;
  --tok-keyword: #dc322f;
  --tok-type: #b58900;
  --tok-string: #2aa198;
  --tok-comment: #93a1a1;
  --tok-number: #d33682;
  --callout-note: #268bd2;
  --callout-tip: #859900;
  --callout-warning: #b58900;
  --callout-danger: #dc322f;
}
"#
            }
        }
    }

    /// Theme-specific CSS rules (e.g. LaTeX booktabs tables, theorem-style callout formatting).
    pub fn custom_rules(&self) -> &'static str {
        match self {
            ThemeKind::Textbook => {
                r#"
/* LaTeX Textbook Aesthetic Styling */
body {
  font-feature-settings: "kern" 1, "liga" 1, "calt" 1;
  text-rendering: optimizeLegibility;
}

h1, h2, h3, h4, h5, h6 {
  font-family: var(--font-body);
  letter-spacing: -0.01em;
}

h1 {
  border-bottom: 2px solid var(--text);
  font-weight: 700;
  padding-bottom: 0.25em;
}

h2 {
  border-bottom: 1px solid var(--border);
  font-weight: 600;
  padding-bottom: 0.2em;
}

/* Classical LaTeX "Booktabs" Tables: thick top/bottom rules, no vertical borders */
table {
  border-top: 2px solid var(--text) !important;
  border-bottom: 2px solid var(--text) !important;
  border-collapse: collapse !important;
}

table th {
  border-top: none !important;
  border-left: none !important;
  border-right: none !important;
  border-bottom: 1px solid var(--text) !important;
  background: transparent !important;
  font-variant: small-caps;
  letter-spacing: 0.05em;
}

table td {
  border-top: none !important;
  border-left: none !important;
  border-right: none !important;
  border-bottom: 1px solid var(--border-muted) !important;
}

table tr:last-child td {
  border-bottom: none !important;
}

/* Mathematical Theorem / Lemma / Remark Style Callouts */
.callout {
  border-left-width: 3px;
  background: var(--code-bg);
  border-radius: 2px;
  padding: 0.9em 1.25em;
}

.callout-title {
  font-family: var(--font-body);
  font-size: 0.85em;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.callout-note .callout-title::before { content: "§ "; }
.callout-tip .callout-title::before { content: "◈ "; }
.callout-warning .callout-title::before { content: "▲ "; }
.callout-danger .callout-title::before { content: "■ "; }

.callout-body {
  font-style: normal;
}
"#
            }
            _ => "",
        }
    }
}

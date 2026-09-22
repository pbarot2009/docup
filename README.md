<p align="center">
  <img src="banner.png" alt="DocUP Banner" width="100%">
</p>

<h1 align="center">DocUP</h1>

<p align="center">
  <strong>Document Unambiguous Precise</strong><br>
  A document markup language that compiles to a standalone HTML5 file.
</p>

<p align="center">
  <img src="https://img.shields.io/github/license/pbarot2009/docup?style=flat-square" alt="License">
  <img src="https://img.shields.io/github/stars/pbarot2009/docup?style=flat-square" alt="Stars">
  <img src="https://img.shields.io/github/forks/pbarot2009/docup?style=flat-square" alt="Forks">
  <img src="https://img.shields.io/github/issues/pbarot2009/docup?style=flat-square" alt="Issues">
  <img src="https://img.shields.io/github/last-commit/pbarot2009/docup?style=flat-square" alt="Last Commit">
  <img src="https://img.shields.io/github/languages/top/pbarot2009/docup?style=flat-square" alt="Top Language">
</p>

---

## Table of Contents

- [About](#about)
- [Build from Source](#build-from-source)
- [Example Document](#example-document)
- [Usage](#usage)
- [Features (v0.2.0)](#features-v020)
- [Roadmap: v0.3.0](#roadmap-v030)
- [Documentation](#documentation)
- [Important Links](#important-links)
- [Contributing](#contributing)
- [License](#license)

---

## About

This repository is the DocUP compiler.

**Doc**ument **U**nambiguous **P**recise.

DocUP is specified like a programming language: named blocks, explicit scopes, a single-pass parse, and a grammar with one reading for a given input.

It is not a Markdown replacement. The point is that markup should be predictable, explicit, fast to parse, and easy to extend.

The compiler writes one HTML5 file. Built-in CSS ships in the output. KaTeX and Google Fonts are linked from CDNs only when the generated page needs them.

---

## Build from Source

```bash
./build.sh
```

Clean:

```bash
./build.sh clean
```

---

## Example Document

```
meta {
    title: "Hello DocUP",
    author: "Prathmesh",
    version: "1.0.0"
}

h(1) { Hello, DocUP }

p {
    This is a paragraph with b{bold}, i{italic}, and inline code{let x = 1;}.
}

codeblock(lang: "go", file: "main.go") {!
package main

func main() {
    println("Hello, DocUP!")
}
!}
```

Output is a single `.html` file.

---

## Usage

```
docup build hello.du
docup build hello.du -o hello.html
docup build hello.du -o dist/index.html -s style.css
docup build hello.du --verbose
docup build hello.du --quiet

docup watch hello.du
docup watch hello.du --port 3000

docup version
docup help
```

`build` compiles once. `watch` rebuilds on change and serves `127.0.0.1:8080` by default.

Flags:

| Flag | Meaning |
| --- | --- |
| `-o`, `--output` | Output HTML path (default: input path with `.html`) |
| `-s`, `--style` | CSS file copied into the output |
| `-p`, `--port` | Watch-mode port (default `8080`) |
| `-V`, `--verbose` | Stage timings |
| `-q`, `--quiet` | No progress lines |

`--verbose` and `--quiet` cannot be used together.

---

## Features (v0.2.0)

- [x] Metadata (`meta`) — `title` required when `meta` is present; also `lang`, `theme`, `author`, `version`, `description`, `keywords`, `canonical`, `image`, `stylesheet`
- [x] Headings (`h(1)`–`h(6)`, `id` / `class`)
- [x] Paragraphs (`p`)
- [x] Inlines: `b`, `i`, `strike`, `code`, `m`, `link`, `fn`
- [x] Code blocks (`codeblock`) — `lang`, `file`, `line_numbers`, `highlight`
- [x] Horizontal rule (`hr`)
- [x] Lists (`list`, `item`) — unordered, `ordered: true`, nestable
- [x] Task lists (`task`, `done`)
- [x] Blockquotes (`quote`, nestable)
- [x] Images (`image`)
- [x] Tables (`table`, `row`, `cell`, `header`)
- [x] Callouts (`callout`) — `note`, `tip`, `warning`, `danger`
- [x] Raw HTML (`raw`)
- [x] Display math (`math`) and inline math (`m`)
- [x] Table of contents (`toc`)
- [x] Footnotes (`footnote`, `fn`)
- [x] Includes (`include`)
- [x] Custom CSS (`--style` and meta `stylesheet`)
- [x] Watch mode with live reload

Language reference: `docs/index.du` in this repo.

---

## Roadmap: v0.3.0

Language:

- [ ] `list(ordered: true, start: N)` — ordered list start index
- [ ] `br {}` — explicit line break inside prose
- [ ] Prose escapes — `\{`, `\}`, `\!` so reserved characters are literal outside strings
- [ ] `image("src", href: "url", alt: "...")` — image wrapped in a link
- [ ] `figure("src", alt: "...", caption: "...")` — image plus caption
- [ ] `quote(cite: "url", author: "name")` — attribution on a blockquote
- [ ] `callout(type: "note", title: "...")` — custom callout title
- [ ] `table(caption: "...")` — table caption
- [ ] `cell(colspan: N, rowspan: N)` — merged cells
- [ ] `deflist { term { } desc { } }` — definition list
- [ ] `details { summary { } ... }` — collapse block
- [ ] `columns { col { } col { } }` — side-by-side columns
- [ ] `sup { }` / `sub { }` — superscript and subscript
- [ ] `mark { }` — highlight span
- [ ] `kbd { }` — keyboard key
- [ ] `ref("heading-id")` — in-document cross reference, resolved after includes
- [ ] `include "file.du" (shift: 1)` — include and bump heading levels
- [ ] Numbered display math — `math(id: "eq1")` plus `ref("eq1")`
- [ ] `comment { }` — source-only block, dropped from HTML
- [ ] `id` / `class` on `p`, `list`, `quote`, `table`, `codeblock`

Compiler and output:

- [ ] `docup fmt` — rewrite a `.du` file with stable layout
- [ ] `docup ast` — print the document AST as JSON
- [ ] `docup build --fragment` — emit body HTML only, no page shell
- [ ] Multipage build — one output HTML per root `.du` in a directory
- [ ] Heading auto-numbers — `meta { numbering: "true" }`, reflected in `toc`
- [ ] Theme toggle in the page — button that sets `data-theme`
- [ ] Heading permalink control — `meta { permalinks: "true" }`
- [ ] `dir` and `lang` per block — `p(lang: "hi", dir: "ltr")`

---

## Documentation

The reference is written in DocUP:

```
cd docs
docup build index.du
```

`docs/index.du` is the root file. The other `.du` files are pulled in with `include`.

---

## Important Links

- Repository: https://github.com/pbarot2009/docup
- License: [LICENSE](LICENSE)
- Creator: https://github.com/pbarot2009

---

## Contributing

Open an issue or a pull request.

---

## License

Apache 2.0. See [LICENSE](LICENSE).

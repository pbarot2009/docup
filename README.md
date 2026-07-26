# DocUP

**Document Markup, Unambiguous & Precise**

A document markup language with programming-language semantics single-pass parsing, zero ambiguity, explicit scoping. 

Compiles `.du` files to standalone HTML5.

![License](https://img.shields.io/github/license/pbarot2009/docup?style=flat-square)
![Stars](https://img.shields.io/github/stars/pbarot2009/docup?style=flat-square)
![Forks](https://img.shields.io/github/forks/pbarot2009/docup?style=flat-square)
![Issues](https://img.shields.io/github/issues/pbarot2009/docup?style=flat-square)
![Last Commit](https://img.shields.io/github/last-commit/pbarot2009/docup?style=flat-square)
![Top Language](https://img.shields.io/github/languages/top/pbarot2009/docup?style=flat-square)

---

## Table of Contents

- [About](#about)
- [Example Document](#example-document)
- [Usage](#usage)
- [Features (v0.1.0)](#features-v010)
- [Roadmap: v0.2.0](#roadmap-v020)
- [Important Links](#important-links)
- [Contributing](#contributing)
- [License](#license)

---

## About

> This is the official GitHub repository for the **DocUP** compiler source code.

## **Doc**ument **U**nambiguous **P**recise

DocUP is a document markup language designed the way a programming language is designed: O(N) single-pass parsing, explicit scoping, and zero ambiguity in the grammar.

**NOTE:** DocUP is not trying to replace Markdown. DocUP is built on the philosophy that document markup should be **Predictable**, **Explicit**, **Fast to parse**, and **Easy to extend**.

**DocUP** compiles to a standalone HTML5 document with minimal default CSS, syntax-highlighted code blocks, and no external runtime dependencies.

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

**Compiled output:** a single `.html` file — no build step, no client-side JS, no external assets required to view it.

---

## Build from Source

Clone this repo and run the following script:

```bash
$ ./build.sh
```

to clean:

```bash
$ ./build.sh clean
```

---

## Usage

```
docup build hello.du -o hello.html
```

```
docup build hello.du --verbose
docup build hello.du --quiet
docup version
docup help
```

---

## Features (v0.1.0)

- [x] Metadata block (`meta`)
- [x] Headings (`h(1)` – `h(6)`, with `id`/`class` attributes)
- [x] Paragraphs (`p`)
- [x] Inline styling (`b`, `i`, `code`)
- [x] Raw code blocks (`codeblock`) with syntax highlighting
- [x] Horizontal rule (`hr`)
- [x] Lists (`list`, `item`, ordered/unordered)
- [x] Links (`link`)
- [x] Blockquotes (`quote`)
- [x] Images (`image`)

---

## Roadmap: v0.2.0

Target: 25 total features.

- [x] Metadata block (`meta`)
- [x] Headings (`h(1)`–`h(6)`)
- [x] Paragraphs (`p`)
- [x] Bold (`b`)
- [x] Italic (`i`)
- [x] Inline code (`code`)
- [x] Raw code blocks (`codeblock`)
- [x] Horizontal rule (`hr`)
- [x] Lists (`list`, `item`)
- [x] Links (`link`)
- [x] Blockquotes (`quote`)
- [x] Images (`image`)
- [ ] Strikethrough (`strike`)
- [ ] Nested blockquotes
- [ ] Nested lists
- [ ] Ordered lists with custom start index
- [ ] Task lists (`task`, `done`)
- [ ] Tables (`table`, `row`, `cell`)
- [ ] Footnotes (`fnref`, `fndef`)
- [ ] Raw embeds (`raw`, e.g. `type: "html"`)
- [ ] Linked images
- [ ] Custom CSS injection (document-supplied stylesheet)
- [ ] Table of contents generation
- [ ] Cross-document links / includes
- [ ] Line breaks within paragraphs
- [ ] Escape sequences for reserved characters

---

## Important Links

- **Repository:** https://github.com/pbarot2009/docup
- **License:** [LICENSE](LICENSE)
- **Creator GitHub:** https://github.com/pbarot2009

---

## Contributing

Contributions are welcome. Feel free to open an issue or submit a pull request to help improve DocUP.

---

## License

This project is licensed under the terms of the Apache 2.0 License. See the [LICENSE](LICENSE) file for details.

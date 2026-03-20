# md-previewer Roadmap

A fast, minimal Rust CLI to preview Markdown files as HTML in the browser.

---

## Progress Overview

| Phase | Description | Status |
|-------|-------------|--------|
| [Phase 1](#-phase-1--mvp) | MVP — Markdown → HTML | ✅ Complete |
| [Phase 2](#-phase-2--cli-usability) | CLI usability | ✅ Complete |
| [Phase 3](#-phase-3--browser-preview) | Browser preview | ✅ Complete |
| [Phase 4](#-phase-4--live-reload) | Live reload | ✅ Complete |
| [Phase 5](#-phase-5--styling--themes) | Styling & themes | ✅ Complete |
| [Phase 6](#-phase-6--advanced-features) | Advanced features | 🔄 In progress |
| [Phase 7](#-phase-7--polish--release) | Polish & release | 🔲 Upcoming |

---

## Phase 1 — MVP

**Goal:** Convert a Markdown file to HTML and print it to the terminal.

- [x] Initialise Rust project (`cargo new md-previewer`)
- [x] Read a file path from a CLI argument
- [x] Parse Markdown using `pulldown-cmark`
- [x] Convert to HTML
- [x] Print HTML to terminal

---

## Phase 2 — CLI Usability

**Goal:** Make it usable as a real command-line tool.

- [x] Add argument parsing with `clap`
- [x] Handle errors gracefully (missing file, invalid input)
- [x] Expose a `--help` flag with usage examples
- [x] Refactor into a clean module structure (separate functions per concern)

---

## Phase 3 — Browser Preview

**Goal:** Write rendered HTML to a temp file and open it in the default browser.

- [x] Generate a temporary `.html` file
- [x] Wrap output in a full HTML template (charset, viewport, basic reset)
- [x] Open the file in the default browser via the `webbrowser` crate
- [x] Clean up temp files on exit (optional)

---

## Phase 4 — Live Reload

**Goal:** Re-render and refresh the browser automatically on every file save.

- [x] Watch for file changes using the `notify` crate
- [x] Rebuild HTML on file save
- [x] Trigger a browser refresh (inject a small WebSocket snippet)
- [x] Add a `--watch` flag

> **Note:** Fix typo in source — "filse changes" → "file changes".

---

## Phase 5 — Styling & Themes

**Goal:** Make previews look good out of the box.

- [x] Embed a tasteful default CSS stylesheet
- [x] Support `--theme dark | light` flag
- [x] Ship a GitHub-flavored Markdown theme
- [x] Allow a user-supplied stylesheet via `--css <path>`

---

## Phase 6 — Advanced Features

**Goal:** Stand out from basic implementations.

- [x] Preview a folder of Markdown files with a sidebar file index
- [ ] Auto-generate a table of contents from headings
- [ ] Syntax-highlight fenced code blocks using `syntect`
- [ ] Support custom HTML templates via `--template <path>`

---

## Phase 7 — Polish & Release

**Goal:** Make it production-ready and publicly available.

- [x] Write a comprehensive `README.md` with badges and usage examples
- [ ] Record a demo GIF using `vhs` or `asciinema`
- [x] Add `.gitignore`
- [x] Build release binaries (`cargo build --release`)
- [x] Publish on GitHub Releases with prebuilt binaries for major platforms
- [ ] ~~Submit to `crates.io`~~ *(not planned)*

---

## Future Ideas

- [ ] Serve via `localhost` (e.g. using `axum` or `actix-web`)
- [ ] Package as a desktop app with Tauri
- [ ] Export to PDF (headless Chrome or `wkhtmltopdf`)
- [ ] Math rendering support (MathJax / KaTeX)
- [ ] Mermaid diagram rendering
- [ ] Plugin system for custom renderers

---

## Learning Outcomes

Working through this project covers:

- Rust fundamentals — ownership, lifetimes, error handling
- CLI application development with `clap`
- File I/O, file watching, and parsing
- Working with external crates (`pulldown-cmark`, `notify`, `webbrowser`)
- Building and publishing real-world developer tooling

---

> **Tip:** Commit after completing each task for a clean Git history.

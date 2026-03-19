# md-previewer

A fast, minimal CLI tool written in Rust to preview Markdown files as HTML in your browser — with optional live reload.

Built as a learning project. See [ROADMAP](ROADMAP.md) for planned features and [CHANGELOG](CHANGELOG.md) for release history.

---

## Features

- Renders Markdown to HTML and opens it in your default browser
- Live reload on file save (`--watch`)
- Prints raw HTML to stdout (`--no-open`)
- Optionally keeps the generated preview file (`--save`)

## Installation

**Recommended — download a prebuilt binary** from the [Releases](https://github.com/qazaroth/md-previewer/releases) tab. No Rust toolchain required.

Alternatively, build from source:

```bash
git clone https://github.com/your-username/md-previewer
cd md-previewer
cargo build --release
```

The binary will be at `./target/release/md-previewer`.

## Usage

```bash
# Open a preview in the browser
md-previewer --file README.md

# Live reload on save
md-previewer --file README.md --watch

# Print HTML to stdout
md-previewer --file README.md --no-open

# Keep the generated preview.html after exit
md-previewer --file README.md --save
```

## Options

| Flag | Description |
|------|-------------|
| `--file <PATH>` | Path to the Markdown file *(required)* |
| `--watch` | Re-render on every file save |
| `--no-open` | Print HTML to stdout instead of opening a browser |
| `--save` | Keep the temporary `preview.html` after exit |
| `--verbose` | Print diagnostic output |
| `--theme` | Specifies the theme of the preview |
| `--css` | Specifies your own custom css theme |

## Built With

- [`pulldown-cmark`](https://github.com/raphlinus/pulldown-cmark) — Markdown parsing
- [`clap`](https://github.com/clap-rs/clap) — CLI argument parsing
- [`notify`](https://github.com/notify-rs/notify) — File watching
- [`webbrowser`](https://github.com/amodm/webbrowser-rs) — Opening the browser
- [`ctrlc`](https://github.com/Detegr/rust-ctrlc) — Ctrl-C handling

## Acknowledgements

AI tooling (Claude) was used as an assistance tool during development — for code review, refactoring suggestions, and documentation.

## License

MIT

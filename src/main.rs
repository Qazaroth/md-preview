mod args;
mod browser;
mod config;
mod markdown;

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, mpsc::channel},
};

const CSS_LIGHT: &str = include_str!("themes/light.css");
const CSS_DARK: &str = include_str!("themes/dark.css");
const CSS_GIT: &str = include_str!("themes/github.css");

// ---------------------------------------------------------------------------
// TempPreview — RAII guard that deletes the preview file on drop
// ---------------------------------------------------------------------------

pub struct TempPreview {
    path: PathBuf,
    save: bool,
}

impl TempPreview {
    pub fn new(path: PathBuf, save: bool) -> Self {
        Self { path, save }
    }

    fn cleanup(&self) {
        if !self.save {
            let _ = fs::remove_file(&self.path);
        }
    }
}

impl Drop for TempPreview {
    fn drop(&mut self) {
        self.cleanup();
    }
}

// ---------------------------------------------------------------------------
// Core logic helpers
// ---------------------------------------------------------------------------

fn resolve_css(args: &args::Args, config: &config::Config) -> Result<String, Box<dyn Error>> {
    if let Some(path) = &args.css {
        return Ok(fs::read_to_string(path)?);
    }

    let theme = args
        .theme
        .as_deref()
        .or(config.theme.as_deref())
        .unwrap_or("light");

    Ok(match theme {
        "dark" => CSS_DARK.to_string(),
        "github" => CSS_GIT.to_string(),
        _ => CSS_LIGHT.to_string(),
    })
}

fn resolve_output_filename(args: &args::Args, config: &config::Config) -> String {
    args.output
        .clone()
        .or_else(|| config.output_filename.clone())
        .unwrap_or_else(|| "preview.html".to_string())
}

/// Render a Markdown file to a full HTML page.
fn render_markdown_file(path: &Path, css: &str) -> Result<String, Box<dyn Error>> {
    let markdown = markdown::read_markdown(path)?;
    Ok(markdown::markdown_to_html(&markdown, css))
}

/// Watch `src` for modifications and re-write the rendered HTML to `dest`
/// on every change.
fn watch_file(src: &Path, dest: &Path, css: &str) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Config::default())?;
    watcher.watch(src, RecursiveMode::NonRecursive)?;

    println!("Watching {} for changes…", src.display());

    for event in rx {
        match event {
            Ok(ev) if matches!(ev.kind, EventKind::Modify(_)) => {
                println!("File changed — regenerating HTML…");
                let html = render_markdown_file(src, css)?;
                fs::write(dest, html)?;
            }
            Ok(_) => {}
            Err(e) => eprintln!("Watch error: {e:?}"),
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::parse_args()?;
    let config = config::Config::load();

    if args.verbose {
        println!("{args:?}");
        println!("{config:?}");
    }

    let css = resolve_css(&args, &config)?;

    // --no-open: print HTML to stdout and exit immediately.
    if args.no_open {
        let html = render_markdown_file(&args.file, &css)?;
        println!("{html}");
        return Ok(());
    }

    let filename = resolve_output_filename(&args, &config);
    let save = args.save.or(config.save).unwrap_or(false);

    let html = render_markdown_file(&args.file, &css)?;
    let preview_path = browser::open_html_and_wait(&html, &filename)?;

    // `preview` deletes the file on drop (unless --save was set).
    let preview = Arc::new(TempPreview::new(preview_path.clone(), save));

    // Mirror the same cleanup in the Ctrl-C handler so the file is removed
    // even when the process is interrupted before `preview` drops normally.
    let preview_for_handler = Arc::clone(&preview);
    ctrlc::set_handler(move || {
        if args.verbose {
            println!("Ctrl+C detected — cleaning up…");
        }
        preview_for_handler.cleanup();
        std::process::exit(0);
    })?;

    if args.watch {
        watch_file(&args.file, &preview_path, &css)?;
    }

    Ok(())
}

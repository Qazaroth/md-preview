mod args;
mod browser;
mod markdown;

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, mpsc::channel},
};

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

/// Render a Markdown file to an HTML string.
fn render_markdown_file(path: &Path) -> Result<String, Box<dyn Error>> {
    let markdown = markdown::read_markdown(path)?;
    Ok(markdown::markdown_to_html(&markdown))
}

/// Watch `src` for modifications and re-write the rendered HTML to `dest`
/// on every change.
fn watch_file(src: &Path, dest: &Path) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Config::default())?;
    watcher.watch(src, RecursiveMode::NonRecursive)?;

    println!("Watching {} for changes…", src.display());

    for event in rx {
        match event {
            Ok(ev) if matches!(ev.kind, EventKind::Modify(_)) => {
                println!("File changed — regenerating HTML…");
                let html = render_markdown_file(src)?;
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

    if args.verbose {
        println!("{args:?}");
    }

    // --no-open: print HTML to stdout and exit immediately.
    if args.no_open {
        let html = render_markdown_file(&args.file)?;
        println!("{html}");
        return Ok(());
    }

    let html = render_markdown_file(&args.file)?;
    let preview_path = browser::open_html_and_wait(&html)?;

    // `_preview` deletes the file on drop (unless --save was passed).
    let preview = Arc::new(TempPreview::new(preview_path.clone(), args.save));

    // Mirror the same cleanup in the Ctrl-C handler so the file is removed
    // even when the process is interrupted before `_preview` drops normally.
    let preview_for_handler = Arc::clone(&preview);
    ctrlc::set_handler(move || {
        if args.verbose {
            println!("Ctrl+C detected — cleaning up…");
        }
        preview_for_handler.cleanup();
        std::process::exit(0);
    })?;

    if args.watch {
        watch_file(&args.file, &preview_path)?;
    }

    Ok(())
}

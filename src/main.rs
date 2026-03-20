mod args;
mod browser;
mod config;
mod folder;
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

macro_rules! verbose {
    ($verbose:expr, $($arg:tt)*) => {
        if $verbose { eprintln!("[verbose] {}", format!($($arg)*)); }
    };
}

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
        verbose!(args.verbose, "using custom CSS from: {}", path.display());
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
fn watch_file(src: &Path, dest: &Path, css: &str, verbose: bool) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Config::default())?;
    watcher.watch(src, RecursiveMode::NonRecursive)?;

    println!("Watching {} for changes…", src.display());

    for event in rx {
        match event {
            Ok(ev) if matches!(ev.kind, EventKind::Modify(_)) => {
                println!("File changed — regenerating HTML…");
                verbose!(verbose, "event: {ev:?}");
                let html = render_markdown_file(src, css)?;
                verbose!(verbose, "rendered {} bytes", html.len());
                fs::write(dest, html)?;
                verbose!(verbose, "wrote to: {}", dest.display());
            }
            Ok(_) => {}
            Err(e) => eprintln!("Watch error: {e:?}"),
        }
    }

    Ok(())
}

fn setup_ctrlc(preview: &Arc<TempPreview>, verbose: bool) -> Result<(), Box<dyn Error>> {
    let preview_for_handler = Arc::clone(preview);
    ctrlc::set_handler(move || {
        verbose!(verbose, "Ctrl+C detected — cleaning up…");
        preview_for_handler.cleanup();
        std::process::exit(0);
    })?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::parse_args()?;
    let config = config::Config::load();

    if args.verbose {
        eprintln!("[verbose] args:   {args:?}");
        eprintln!("[verbose] config: {config:?}");
    }

    let css = resolve_css(&args, &config)?;
    let theme = args
        .theme
        .as_deref()
        .or(config.theme.as_deref())
        .unwrap_or("light");
    verbose!(args.verbose, "theme:           {theme}");

    let filename = resolve_output_filename(&args, &config);
    let save = args.save.or(config.save).unwrap_or(false);
    verbose!(args.verbose, "output filename: {filename}");
    verbose!(args.verbose, "save on exit:    {save}");

    let path = args.file.canonicalize()?;

    // --- Directory mode ---
    if path.is_dir() {
        let files = folder::render_folder(&path, &css)?;
        if files.is_empty() {
            return Err("No Markdown files found in directory.".into());
        }
        verbose!(args.verbose, "found {} markdown files", files.len());

        let html = folder::build_folder_html(&files, &css);
        let preview_path = browser::open_html_and_wait(&html, &filename)?;
        verbose!(
            args.verbose,
            "preview written to: {}",
            preview_path.display()
        );

        let preview = Arc::new(TempPreview::new(preview_path, save));
        setup_ctrlc(&preview, args.verbose)?;

        // Park the thread — nothing else to do in folder mode
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    }

    // --- Single file mode ---

    // --no-open: print HTML to stdout and exit immediately.
    if args.no_open {
        let html = render_markdown_file(&path, &css)?;
        verbose!(args.verbose, "rendered {} bytes of HTML", html.len());
        println!("{html}");
        return Ok(());
    }

    let html = render_markdown_file(&path, &css)?;
    verbose!(args.verbose, "rendered {} bytes of HTML", html.len());

    let preview_path = browser::open_html_and_wait(&html, &filename)?;
    verbose!(
        args.verbose,
        "preview written to: {}",
        preview_path.display()
    );

    let preview = Arc::new(TempPreview::new(preview_path.clone(), save));
    setup_ctrlc(&preview, args.verbose)?;

    if args.watch {
        watch_file(&path, &preview_path, &css, args.verbose)?;
    }

    Ok(())
}

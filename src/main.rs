mod args;
mod config;
mod folder;
mod markdown;
mod server;

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
// Core logic helpers
// ---------------------------------------------------------------------------

fn resolve_template(
    args: &args::Args,
    config: &config::Config,
) -> Result<Option<String>, Box<dyn Error>> {
    if let Some(path) = &args.template {
        return Ok(Some(fs::read_to_string(path)?));
    }
    if let Some(path) = &config.template {
        return Ok(Some(fs::read_to_string(path)?));
    }
    Ok(None)
}

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

fn title_from_path(path: &Path) -> &str {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Preview")
}

/// Render a Markdown file to a full HTML page.
fn render_markdown_file(
    path: &Path,
    css: &str,
    build_toc: bool,
    verbose: bool,
    template: Option<&str>,
) -> Result<String, Box<dyn Error>> {
    let markdown = markdown::read_markdown(path)?;
    let title = title_from_path(path);
    Ok(markdown::markdown_to_html(
        &markdown, css, build_toc, verbose, template, title,
    ))
}

/// Save rendered HTML to disk if --save is set.
fn save_html_if_needed(
    html: &str,
    args: &args::Args,
    config: &config::Config,
    verbose: bool,
) -> Result<(), Box<dyn Error>> {
    let save = args.save.or(config.save).unwrap_or(false);
    if !save {
        return Ok(());
    }
    let save_path = args
        .output
        .clone()
        .or_else(|| config.output_filename.clone())
        .unwrap_or_else(|| "preview.html".to_string());
    fs::write(&save_path, html)?;
    println!("Saved HTML to: {save_path}");
    verbose!(verbose, "saved {} bytes to {save_path}", html.len());
    Ok(())
}

/// Watch `src` for file changes and push a reload via the server state.
fn watch_and_serve(
    src: &Path,
    recursive: bool,
    css: String,
    build_toc: bool,
    verbose: bool,
    template: Option<String>,
    state: Arc<server::ServerState>,
    is_dir: bool,
    dir: Option<PathBuf>,
) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Config::default())?;

    let mode = if recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };

    watcher.watch(src, mode)?;
    println!("Watching {} for changes…", src.display());

    for event in rx {
        match event {
            Ok(ev) if matches!(ev.kind, EventKind::Modify(_)) => {
                println!("File changed — regenerating HTML…");
                verbose!(verbose, "event: {ev:?}");

                let html = if is_dir {
                    let dir = dir.as_deref().unwrap();
                    match folder::render_folder(dir, &css, build_toc, verbose, template.as_deref())
                    {
                        Ok(files) => folder::build_folder_html(&files, &css),
                        Err(e) => {
                            eprintln!("Render error: {e}");
                            continue;
                        }
                    }
                } else {
                    match render_markdown_file(src, &css, build_toc, verbose, template.as_deref()) {
                        Ok(html) => html,
                        Err(e) => {
                            eprintln!("Render error: {e}");
                            continue;
                        }
                    }
                };

                verbose!(verbose, "rendered {} bytes", html.len());

                let rt = tokio::runtime::Handle::current();
                let state = Arc::clone(&state);
                rt.spawn(async move {
                    state.update(html).await;
                });
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = args::parse_args()?;
    let config = config::Config::load();
    let port = args.port.or(config.port).unwrap_or(3000);

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
    verbose!(args.verbose, "theme: {theme}");

    let template = resolve_template(&args, &config)?;
    let template_ref = template.as_deref();

    let path = args.file.canonicalize()?;
    let watch = args.watch;
    let build_toc = args.toc;
    let verbose = args.verbose;

    // --no-open: print HTML to stdout and exit immediately (no server needed).
    if args.no_open {
        let html = render_markdown_file(&path, &css, build_toc, verbose, template_ref)?;
        verbose!(verbose, "rendered {} bytes of HTML", html.len());
        println!("{html}");
        return Ok(());
    }

    // --- Render initial HTML ---
    let initial_html = if path.is_dir() {
        let files = folder::render_folder(&path, &css, build_toc, verbose, template_ref)?;
        if files.is_empty() {
            return Err("No Markdown files found in directory.".into());
        }
        verbose!(verbose, "found {} markdown files", files.len());
        folder::build_folder_html(&files, &css)
    } else {
        render_markdown_file(&path, &css, build_toc, verbose, template_ref)?
    };

    verbose!(verbose, "rendered {} bytes of HTML", initial_html.len());

    // --- Save HTML to disk if --save is set ---
    save_html_if_needed(&initial_html, &args, &config, verbose)?;

    // --- Start server ---
    let state = server::ServerState::new(initial_html);
    let state_for_server = Arc::clone(&state);

    tokio::spawn(async move {
        server::start(state_for_server, port).await;
    });

    // Open browser
    let url = format!("http://127.0.0.1:{port}");
    verbose!(verbose, "opening browser at {url}");
    webbrowser::open(&url)?;

    // --- Watch loop (if --watch) ---
    if watch {
        let is_dir = path.is_dir();
        let dir = if is_dir { Some(path.clone()) } else { None };
        let css_owned = css.clone();
        let template_owned = template.clone();
        let state_for_watch = Arc::clone(&state);

        tokio::task::spawn_blocking(move || {
            if let Err(e) = watch_and_serve(
                &path,
                is_dir,
                css_owned,
                build_toc,
                verbose,
                template_owned,
                state_for_watch,
                is_dir,
                dir,
            ) {
                eprintln!("Watch error: {e}");
            }
        })
        .await?;
    } else {
        println!("Press Ctrl-C to stop the server…");
        tokio::signal::ctrl_c().await?;
        verbose!(verbose, "Ctrl-C received — shutting down…");
    }

    Ok(())
}

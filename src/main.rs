mod args;
mod browser;
mod markdown;

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::error::Error;
use std::sync::mpsc::channel;

fn watch_file(path: &std::path::Path) -> Result<(), Box<dyn Error>> {
    // Channel to receive filesystem events
    let (tx, rx) = channel();

    // Create watcher with 500ms debounce
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Config::default())?;
    watcher.watch(path, RecursiveMode::NonRecursive)?;

    println!("Watching {} for changes...", path.display());

    let markdown_input = markdown::read_markdown(path)?;
    let html_output = markdown::markdown_to_html(&markdown_input);
    let preview_path = browser::open_html_and_wait(&html_output, true)?;

    loop {
        match rx.recv() {
            Ok(event) => {
                if let EventKind::Modify(_) = event.unwrap().kind {
                    println!("File changed| Regenerating HTML...");

                    let markdown_input = markdown::read_markdown(path)?;
                    let html_output = markdown::markdown_to_html(&markdown_input);

                    std::fs::write(&preview_path, html_output)?;
                }
            }
            Err(e) => eprintln!("Watch error: {:?}", e),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::parse_args()?;

    if args.no_open {
        let markdown_input = markdown::read_markdown(&args.file)?;
        let html_output = markdown::markdown_to_html(&markdown_input);

        println!("{}", html_output);

        return Ok(());
    }

    let _ = watch_file(&args.file);

    Ok(())
}

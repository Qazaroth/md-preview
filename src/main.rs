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

    loop {
        match rx.recv() {
            Ok(event) => {
                // Only respond to modifications
                if let EventKind::Modify(_) = event.unwrap().kind {
                    println!("File changed! Regenerating HTML...");

                    let markdown_input = markdown::read_markdown(path)?;
                    let html_output = markdown::markdown_to_html(&markdown_input);

                    let _temp_file = browser::open_html_and_wait(&html_output)?;

                    std::thread::sleep(std::time::Duration::from_secs(10));
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

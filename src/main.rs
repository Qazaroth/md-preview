mod args;
mod browser;
mod markdown;

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::channel;

pub struct TempPreview {
    pub path: PathBuf,
    pub save: bool,
}

impl TempPreview {
    pub fn new(path: PathBuf, save: bool) -> Self {
        TempPreview { path, save }
    }
}

impl Drop for TempPreview {
    fn drop(&mut self) {
        if !self.save {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn watch_file(path: &Path, preview_path: &Path) -> Result<(), Box<dyn Error>> {
    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Config::default())?;
    watcher.watch(path, RecursiveMode::NonRecursive)?;

    println!("Watching {} for changes...", path.display());

    loop {
        match rx.recv() {
            Ok(event) => {
                if let Ok(event) = event {
                    if matches!(event.kind, EventKind::Modify(_)) {
                        println!("File changed | regenerating HTML...");

                        let markdown_input = markdown::read_markdown(path)?;
                        let html_output = markdown::markdown_to_html(&markdown_input);

                        fs::write(preview_path, html_output)?;
                    }
                }
            }
            Err(e) => eprintln!("Watch error: {:?}", e),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::parse_args()?;

    let verbose = args.verbose;

    if args.no_open {
        let markdown_input = markdown::read_markdown(&args.file)?;
        let html_output = markdown::markdown_to_html(&markdown_input);
        println!("{}", html_output);
        return Ok(());
    }

    if verbose {
        println!("{:?}", args);
    }

    let markdown_input = markdown::read_markdown(&args.file)?;
    let html_output = markdown::markdown_to_html(&markdown_input);

    let preview_path = browser::open_html_and_wait(&html_output)?;

    let _temp = TempPreview::new(preview_path.clone(), args.save);

    let handler_path = Arc::new(preview_path.clone());
    let save_flag = args.save;

    ctrlc::set_handler(move || {
        if verbose {
            println!("Ctrl+C detected. Cleaning up...");
        }
        if !save_flag {
            let _ = fs::remove_file(&*handler_path);
        }
        std::process::exit(0);
    })?;

    if args.watch {
        watch_file(&args.file, &preview_path)?;
    }

    Ok(())
}

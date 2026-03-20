use clap::Parser;
use std::{error::Error, path::PathBuf};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Path to the Markdown file to preview
    #[arg(long)]
    pub file: PathBuf,

    /// Print HTML to stdout instead of opening a browser
    #[arg(long)]
    pub no_open: bool,

    /// Re-render on every file save
    #[arg(long)]
    pub watch: bool,

    /// Keep the temporary preview file after exit
    #[arg(long)]
    pub save: Option<bool>,

    /// Print extra diagnostic output
    #[arg(long)]
    pub verbose: bool,

    /// Theme to use: "light" (default), "dark" or "github"
    #[arg(long, default_value = "light")]
    pub theme: Option<String>,

    /// Path to a custom CSS file (overrides --theme)
    #[arg(long)]
    pub css: Option<PathBuf>,

    /// Output filename for the preview HTML
    #[arg(long)]
    pub output: Option<String>,

    /// Auto-generate a table of contents
    #[arg(long)]
    pub toc: bool,

    /// Path to a custom HTML template file
    #[arg(long)]
    pub template: Option<PathBuf>,

    /// Port to serve the preview on (default: 3000)
    #[arg(long, default_value = "3000")]
    pub port: Option<u16>,
}

pub fn parse_args() -> Result<Args, Box<dyn Error>> {
    let args = Args::parse();

    if !args.file.exists() {
        return Err(format!("Path not found: {}", args.file.display()).into());
    }

    Ok(args)
}

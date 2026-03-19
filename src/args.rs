use clap::Parser;
use std::error::Error;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Path to markdown file
    #[arg(long)]
    pub file: PathBuf,

    /// Do not open in browser
    #[arg(long)]
    pub no_open: bool,

    /// Watch markdown file
    #[arg(long)]
    pub watch: bool,

    /// Preserve preview file or delete once done
    #[arg(long)]
    pub save: bool,

    pub verbose: bool,
}

pub fn parse_args() -> Result<Args, Box<dyn Error>> {
    let args = Args::parse();

    if !args.file.exists() {
        return Err("File does not exist.".into());
    }

    Ok(args)
}

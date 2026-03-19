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
}

pub fn parse_args() -> Result<Args, Box<dyn Error>> {
    let args = Args::parse();

    if !args.file.exists() {
        return Err("File does not exist.".into());
    }

    Ok(args)
}

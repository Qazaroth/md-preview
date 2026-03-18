use clap::Parser;
use pulldown_cmark::{Parser as MdParser, html};
use std::fs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to markdown file
    #[arg(long, value_parser = clap::value_parser!(std::path::PathBuf))]
    file: std::path::PathBuf,
}

fn main() {
    let args = Args::parse();

    if !args.file.exists() {
        eprintln!("File does not exist.");
        std::process::exit(1);
    }

    let file_path = args.file;
    let markdown_input = fs::read_to_string(file_path).expect("Failed to read file.");
    let parser = MdParser::new(&markdown_input);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    println!("{}", html_output);
}

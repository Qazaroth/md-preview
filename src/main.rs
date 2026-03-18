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

fn parse_args() -> std::path::PathBuf {
    let args = Args::parse();

    if !args.file.exists() {
        eprintln!("File does not exist.");
        std::process::exit(1);
    }

    args.file
}

fn read_markdown(file_path: &std::path::Path) -> String {
    fs::read_to_string(file_path).unwrap_or_else(|err| {
        eprintln!("Failed to read file {}: {}", file_path.display(), err);
        std::process::exit(1);
    })
}

fn markdown_to_html(markdown_input: &str) -> String {
    let parser = MdParser::new(markdown_input);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

fn main() {
    let file_path = parse_args();
    let markdown_input = read_markdown(&file_path);
    let html_output = markdown_to_html(&markdown_input);

    println!("{}", html_output);
}

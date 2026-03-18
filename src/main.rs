use pulldown_cmark::{Parser, html};
use std::fs;

fn main() {
    let file_path = std::env::args().nth(1).expect("Provide a file path");
    let markdown_input = fs::read_to_string(file_path).expect("Failed to read file.");
    let parser = Parser::new(&markdown_input);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    println!("{}", html_output);
}

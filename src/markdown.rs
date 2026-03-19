use pulldown_cmark::{Parser as MdParser, html};
use std::{error::Error, fs, path::Path};

pub fn read_markdown(path: &Path) -> Result<String, Box<dyn Error>> {
    Ok(fs::read_to_string(path)?)
}

pub fn markdown_to_html(input: &str) -> String {
    let parser = MdParser::new(input);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

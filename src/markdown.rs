use pulldown_cmark::{Options, Parser, html};
use std::{error::Error, fs, path::Path};

pub fn read_markdown(path: &Path) -> Result<String, Box<dyn Error>> {
    Ok(fs::read_to_string(path)?)
}

pub fn markdown_to_html(input: &str, css: &str) -> String {
    let parser = Parser::new_ext(input, Options::all());
    let mut body = String::with_capacity(input.len() * 2);
    html::push_html(&mut body, parser);

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <style>{css}</style>
</head>
<body>
{body}
</body>
</html>"#
    )
}

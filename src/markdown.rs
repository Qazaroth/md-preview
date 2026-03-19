use pulldown_cmark::{Options, Parser, html};
use std::{error::Error, fs, path::Path};

pub fn read_markdown(path: &Path) -> Result<String, Box<dyn Error>> {
    Ok(fs::read_to_string(path)?)
}

pub fn markdown_to_html(input: &str, css: &str) -> String {
    let parser = Parser::new_ext(input, Options::all());
    let mut body = String::with_capacity(input.len() * 2);
    html::push_html(&mut body, parser);

    let body = inject_heading_ids(&body);

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

fn inject_heading_ids(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(start) = rest.find("<h") {
        output.push_str(&rest[..start]);
        rest = &rest[start..];

        // Make sure it's actually a heading tag (h1–h6)
        let level = rest.chars().nth(2);
        if !matches!(level, Some('1'..='6')) || rest.chars().nth(3) != Some('>') {
            output.push_str(&rest[..3]);
            rest = &rest[3..];
            continue;
        }

        let tag_end = match rest.find('>') {
            Some(i) => i,
            None => {
                output.push_str(rest);
                return output;
            }
        };

        let tag = &rest[..=tag_end]; // e.g. "<h2>"
        let level_char = &tag[2..3]; // e.g. "2"
        let close = format!("</h{}>", level_char);

        rest = &rest[tag_end + 1..];

        let content_end = match rest.find(&*close) {
            Some(i) => i,
            None => {
                output.push_str(tag);
                continue;
            }
        };

        let content = &rest[..content_end];
        let id = slugify(&strip_tags(content));

        output.push_str(&format!(
            "<h{} id=\"{}\">{}</h{}>",
            level_char, id, content, level_char
        ));

        rest = &rest[content_end + close.len()..];
    }

    output.push_str(rest);
    output
}

fn strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

fn slugify(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-')
        .map(|c| {
            if c.is_whitespace() {
                '-'
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Phase 1 — MVP"), "phase-1--mvp");
    }

    #[test]
    fn test_heading_ids_injected() {
        let html = "<h1>Hello World</h1><h2>Section</h2>";
        let result = inject_heading_ids(html);
        assert!(result.contains(r#"id="hello-world""#));
        assert!(result.contains(r#"id="section""#));
    }

    #[test]
    fn test_markdown_renders_heading() {
        let html = markdown_to_html("# Title", "");
        assert!(html.contains("<h1"));
        assert!(html.contains("Title"));
    }
}

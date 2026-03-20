use pulldown_cmark::{Options, Parser, html};
use std::{error::Error, fs, path::Path};

macro_rules! verbose {
    ($verbose:expr, $($arg:tt)*) => {
        if $verbose { eprintln!("[verbose] {}", format!($($arg)*)); }
    };
}

struct Heading {
    level: u8,
    text: String,
    id: String,
}

fn has_toc(html: &str) -> bool {
    let lower = html.to_lowercase();
    ["id=\"table-of-contents\"", "id=\"contents\"", "id=\"toc\""]
        .iter()
        .any(|m| lower.contains(m))
}

fn extract_headings(html: &str) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut rest = html;

    while let Some(start) = rest.find("<h") {
        let tag_level = rest[start..].chars().nth(2);
        if !matches!(tag_level, Some('1'..='6')) {
            rest = &rest[start + 1..];
            continue;
        }

        let level = tag_level.unwrap().to_digit(10).unwrap() as u8;

        // Skip h1 - Usually page title
        if level == 1 {
            rest = &rest[start + 1..];
            continue;
        }

        let close = format!("</h{}>", level);
        let content_start = match rest[start..].find('>') {
            Some(i) => start + i + 1,
            None => break,
        };
        let content_end = match rest[content_start..].find(&*close) {
            Some(i) => content_start + i,
            None => break,
        };
        let text = strip_tags(&rest[content_start..content_end]);
        let id = slugify(&text);

        headings.push(Heading { level, text, id });
        rest = &rest[content_end..];
    }

    headings
}

fn build_toc(headings: &[Heading]) -> String {
    if headings.is_empty() {
        return String::new();
    }

    let mut toc = String::from("<nav class=\"toc\"><h2>Table of Contents</h2><ul>\n");
    let base_level = headings.iter().map(|h| h.level).min().unwrap_or(2);

    for h in headings {
        let indent = " ".repeat((h.level - base_level) as usize);
        toc.push_str(&format!(
            "{}<li><a href=\"#{}\">{}</a></li>\n",
            indent, h.id, h.text
        ));
    }

    toc.push_str("</ul></nav>\n");
    toc
}

fn inject_toc(html: &str, toc: &str) -> String {
    // Try to insert after the closing </h1>
    if let Some(pos) = html.find("</h1>") {
        let insert_at = pos + "</h1>".len();
        let mut result = html.to_string();
        result.insert_str(insert_at, toc);
        return result;
    }
    // No h1 — prepend to the top
    format!("{toc}{html}")
}

pub fn read_markdown(path: &Path) -> Result<String, Box<dyn Error>> {
    Ok(fs::read_to_string(path)?)
}

pub fn markdown_to_html(input: &str, css: &str, need_toc: bool, verbose: bool) -> String {
    let parser = Parser::new_ext(input, Options::all());
    let mut body = String::with_capacity(input.len() * 2);
    html::push_html(&mut body, parser);
    let mut body = inject_heading_ids(&body);

    if need_toc {
        verbose!(
            verbose,
            "checking for existing TOC: has_toc={}",
            has_toc(&body)
        );
        body = if !has_toc(&body) {
            let headings = extract_headings(&body);
            verbose!(verbose, "extracted {} headings", headings.len());
            let toc = build_toc(&headings);
            inject_toc(&body, &toc)
        } else {
            verbose!(verbose, "skipping TOC — one already exists");
            body.to_string()
        };
    }

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
        let html = markdown_to_html("# Title", "", true, false);
        assert!(html.contains("<h1"));
        assert!(html.contains("Title"));
    }

    #[test]
    fn test_has_toc_not_fooled_by_table() {
        let html = "<table><th>Flag</th><th>Description</th></table>";
        assert!(!has_toc(html));
    }

    #[test]
    fn test_has_toc_detects_real_toc() {
        let html = inject_heading_ids("<h2>Table of Contents</h2>");
        eprintln!("inject_heading_ids output: {html}");
        eprintln!("has_toc result: {}", has_toc(&html));
        assert!(has_toc(&html));
    }
}

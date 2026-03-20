use crate::markdown;
use std::{error::Error, fs, path::Path};

pub struct RenderedFile {
    pub name: String, // display name
    pub id: String,   // used as HTML id
    pub html: String, // well, the html
}

fn slugify(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

pub fn render_folder(
    dir: &Path,
    css: &str,
    build_toc: bool,
    verbose: bool,
) -> Result<Vec<RenderedFile>, Box<dyn Error>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_string();
        let id = slugify(&name);
        let markdown = fs::read_to_string(&path)?;
        let html = markdown::markdown_to_html(&markdown, css, build_toc, verbose);

        files.push(RenderedFile { name, id, html });
    }

    files.sort_by(|a, b| {
        let a_readme = a.name.to_lowercase() == "readme";
        let b_readme = b.name.to_lowercase() == "readme";
        match (a_readme, b_readme) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });

    Ok(files)
}

pub fn build_folder_html(files: &[RenderedFile], css: &str) -> String {
    let sidebar_links: String = files
        .iter()
        .map(|f| {
            format!(
                "<li><a href=\"#\" onclick=\"show('{}'); return false;\">{}</a></li>",
                f.id, f.name
            )
        })
        .collect();

    let content_sections: String = files
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let display = if i == 0 { "block" } else { "none" };
            format!(
                "<section id=\"{}\" style=\"display:{}\">{}</section>",
                f.id, display, f.html
            )
        })
        .collect();

    let first_id = files.first().map(|f| f.id.as_str()).unwrap_or("");

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <style>
    {css}
    body {{ display: flex; margin: 0; padding: 0; }}
    #sidebar {{
      width: 220px;
      min-height: 100vh;
      padding: 1rem;
      border-right: 1px solid #e0e0e0;
      flex-shrink: 0;
    }}
    #sidebar ul {{ list-style: none; padding: 0; margin: 0; }}
    #sidebar li {{ margin: 0.4rem 0; }}
    #sidebar a {{ text-decoration: none; }}
    #sidebar a:hover {{ text-decoration: underline; }}
    #content {{ flex: 1; padding: 2rem; max-width: 800px; }}
    .active {{ font-weight: bold; }}
  </style>
</head>
<body>
  <nav id="sidebar">
    <ul>{sidebar_links}</ul>
  </nav>
  <main id="content">
    {content_sections}
  </main>
  <script>
    let current = '{first_id}';
    function show(id) {{
      document.getElementById(current).style.display = 'none';
      document.getElementById(id).style.display = 'block';
      current = id;
    }}
  </script>
</body>
</html>"#
    )
}

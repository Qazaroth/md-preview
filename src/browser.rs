use std::{error::Error, fs, path::PathBuf};

const PREVIEW_FILE: &str = "preview.html";

pub fn open_html_and_wait(html: &str) -> Result<PathBuf, Box<dyn Error>> {
    let preview_path = PathBuf::from(PREVIEW_FILE);
    fs::write(&preview_path, html)?;

    let path_str = preview_path
        .to_str()
        .ok_or("Preview path contains invalid UTF-8")?;

    webbrowser::open(path_str)?;
    Ok(preview_path)
}

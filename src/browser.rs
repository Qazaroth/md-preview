use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub fn open_html_and_wait(html: &str) -> Result<PathBuf, Box<dyn Error>> {
    let preview_path = PathBuf::from("preview.html");

    fs::write(&preview_path, html)?;

    webbrowser::open(preview_path.to_str().unwrap())?;

    Ok(preview_path)
}

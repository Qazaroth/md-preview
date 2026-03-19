use std::error::Error;
use std::io::Write;
use tempfile::NamedTempFile;

pub fn open_html_and_wait(html: &str) -> Result<NamedTempFile, Box<dyn Error>> {
    let mut file = NamedTempFile::new()?;
    write!(file, "{}", html)?;

    let path = file.path().to_str().unwrap().to_string();

    webbrowser::open(&path)?;

    // file auto-deletes here
    Ok(file)
}

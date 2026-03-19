use std::error::Error;
use std::io::Write;
use tempfile::NamedTempFile;

pub fn open_html_and_wait(html: &str) -> Result<(), Box<dyn Error>> {
    let mut file = NamedTempFile::new()?;
    write!(file, "{}", html)?;

    let path = file.path().to_str().unwrap().to_string();

    webbrowser::open(&path)?;

    println!("Press Enter after closing the browser...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // file auto-deletes here
    Ok(())
}

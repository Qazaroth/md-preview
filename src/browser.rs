use std::error::Error;
use std::fs;
use std::path::PathBuf;

pub fn open_html_and_wait(html: &str, delete_after: bool) -> Result<PathBuf, Box<dyn Error>> {
    let preview_path = PathBuf::from("preview.html");

    fs::write(&preview_path, html)?;

    webbrowser::open(preview_path.to_str().unwrap())?;

    if delete_after {
        let path_clone = preview_path.clone();
        ctrlc::set_handler(move || {
            let _ = std::fs::remove_file(&path_clone);
            std::process::exit(0);
        })?;
    }

    Ok(preview_path)
}

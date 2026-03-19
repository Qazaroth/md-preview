mod args;
mod browser;
mod markdown;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = args::parse_args()?;
    let markdown_input = markdown::read_markdown(&args.file)?;
    let html_output = markdown::markdown_to_html(&markdown_input);

    if args.no_open {
        println!("{}", html_output);
    } else {
        browser::open_html_and_wait(&html_output)?;
    }

    Ok(())
}

//! Renders Markdown, etc .. to HTML

use std::{fs, path::Path};

use anyhow::{Context, Result};
use pulldown_cmark::{html, Options, Parser};

/// HTML renderer
#[derive(Debug, Clone)]
pub struct HtmlRenderer {}

impl HtmlRenderer {
    /// Initializes the HTML renderer
    pub fn new() -> Self {
        HtmlRenderer {}
    }

    /// Render a source file to a target format
    pub fn render<P: AsRef<Path>>(&self, p: P) -> Result<String> {
        let input = fs::read_to_string(p).context("Invalid file")?;

        let options = Options::empty();
        let parser = Parser::new_ext(&input, options);

        let mut output = String::new();
        html::push_html(&mut output, parser);

        Ok(output)
    }
}

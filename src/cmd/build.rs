//! Builds the site

use std::process::exit;

use clap::Parser;
use colored::Colorize;

use crate::{
    build::PageBuilder,
    config::{self, Config},
    render::HtmlRenderer,
    template::TemplatesRegistry,
};

/// `init` CLI arguments
#[derive(Debug, Parser)]
pub struct Args {
    /// Working directory
    #[clap(long)]
    workdir: Option<String>,
}

/// Builds the site
pub async fn run(args: &Args) {
    // get CWD
    match config::set_current_dir(&args.workdir) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{}", "✗ Cannot set working directory".bright_red());
            eprintln!("{}", format!("{err}").bright_red());
            exit(1);
        }
    }

    // load Config
    let config = match Config::load() {
        Ok(c) => c,
        Err(err) => {
            eprintln!("{}", "✗ Failed to load config".bright_red());
            eprintln!("{}", format!("{err}").bright_red());
            exit(1);
        }
    };
    // eprintln!("{config:#?}");

    // build site
    let registry = match TemplatesRegistry::init(&config) {
        Ok(r) => r,
        Err(err) => {
            eprintln!("{}", "✗ Failed to load config".bright_red());
            eprintln!("{}", format!("{err}").bright_red());
            exit(1);
        }
    };
    let builder = PageBuilder {
        src_dir: config.src_dir(),
        build_dir: config.build_dir(),
        template_dir: config.template_dir(),
        renderer: HtmlRenderer::new(),
        registry: registry,
    };

    match builder.build_all() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{}", "✗ Failed to build site".bright_red());
            eprintln!("{}", format!("{err}").bright_red());
            exit(1);
        }
    };

    eprintln!("{}", "✓ Build OK".bright_green());
}

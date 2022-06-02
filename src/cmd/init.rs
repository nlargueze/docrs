//! Initializes a folder

use std::process::exit;

use clap::Parser;
use colored::Colorize;

use crate::config::{self, Config};

/// `init` CLI arguments
#[derive(Debug, Parser)]
pub struct Args {
    /// Working directory
    #[clap(short, long)]
    wordir: Option<String>,
    /// Force the re-initialization
    #[clap(short, long)]
    force: bool,
}

/// Initializes a folder
pub async fn run(args: &Args) {
    // get CWD
    match config::set_current_dir(&args.wordir) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{}", "✗ Cannot set working directory".bright_red());
            eprintln!("{}", format!("{err}").bright_red());
            exit(1);
        }
    }

    // reset if forced, or abort
    let config = Config::default();
    if config.is_initialized() {
        if args.force {
            match config.reset_repo() {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("{}", "✗ Cannot delete config folder".bright_red());
                    eprintln!("{}", format!("{err}").bright_red());
                    exit(1);
                }
            };
        } else {
            eprintln!(
                "{}",
                "✗ Repo is already intialized, user --force option".bright_red()
            );
            exit(1);
        }
    }

    // init
    match config.init_repo() {
        Ok(_) => {
            eprintln!("{}", "✓ Repository initialized".bright_green());
        }
        Err(err) => {
            eprintln!("{}", "✗ Cannot init repo".bright_red());
            eprintln!("{}", format!("{err}").bright_red());
            exit(1);
        }
    }
}

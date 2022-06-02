//! Serves the build

use std::{net::SocketAddr, process::exit};

use clap::Parser;
use colored::Colorize;
use tokio::sync::broadcast;

use crate::{
    config::{self, Config},
    server::start_server,
};

/// `init` CLI arguments
#[derive(Debug, Parser)]
pub struct Args {
    /// Working directory
    #[clap(long)]
    workdir: Option<String>,
    /// Port
    #[clap(short, long)]
    port: Option<u16>,
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

    // open browser
    let port = args.port.unwrap_or(5002);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    match webbrowser::open(format!("http://localhost:{port}").as_str()) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{}", "✗ Failed to open browser".bright_red());
            eprintln!("{}", format!("{err}").bright_red());
            exit(1);
        }
    }

    // SSE channel
    let (tx_event, _) = broadcast::channel(1);

    // // NB: Testing
    // let sse = SeeEventManager::new(tx_event.clone());
    // thread::spawn(move || loop {
    //     thread::sleep(Duration::from_millis(1_000));
    //     eprintln!("Sent SSE event");
    //     sse.send_reload();
    // });

    // start server
    let build_dir = config.build_dir();
    eprintln!(
        "{}",
        format!("✓ File server started on {addr}").bright_green()
    );
    start_server(&build_dir, &addr, tx_event).await;
}

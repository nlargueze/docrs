//! Builds the site

use std::{net::SocketAddr, path::PathBuf, process::exit};

use clap::Parser;
use colored::Colorize;
use hotwatch::Hotwatch;
use tokio::sync::{broadcast, mpsc};

use crate::{
    build::PageBuilder,
    config::{self, Config},
    render::HtmlRenderer,
    server::start_server,
    sse::SeeEventManager,
    template::TemplatesRegistry,
};

/// `init` CLI arguments
#[derive(Debug, Parser)]
pub struct Args {
    /// Working directory
    #[clap(long)]
    workdir: Option<String>,
    /// Port to serve the site on
    #[clap(short, long)]
    port: Option<u16>,
}

/// A Watch Event is an event for the file watcher
#[derive(Debug, Clone)]
enum WatchEvent {
    /// The template is updated
    TemplateUpdate,
    /// A source file is create or update
    FileUpdate(PathBuf),
    /// A source file is deleted
    FileRemoved(PathBuf),
    /// A source file is renamed, from --> to
    FileRenamed(PathBuf, PathBuf),
}

/// Builds and serves the site in dev mode
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
    let mut builder = PageBuilder {
        src_dir: config.src_dir().clone(),
        build_dir: config.build_dir().clone(),
        template_dir: config.clone().template_dir(),
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

    // Watch event channel
    let (tx_watch_event, mut rx_watch_event) = mpsc::unbounded_channel::<WatchEvent>();

    // Watch files in the template folder
    let mut template_watcher = Hotwatch::new().unwrap();
    let tx_watch_event_2 = tx_watch_event.clone();
    template_watcher
        .watch(&config.template_dir(), move |_event| {
            eprintln!("i Template updated");
            match tx_watch_event_2.send(WatchEvent::TemplateUpdate) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("{}", format!("{err}").bright_red());
                    panic!("{err}");
                }
            }
        })
        .unwrap();
    eprintln!("{}", format!("✓ Watching template folder").bright_green());

    // Watch files in the src folder
    let mut src_watcher = Hotwatch::new().unwrap();
    let tx_watch_event_3 = tx_watch_event.clone();
    src_watcher
        .watch(&config.src_dir(), move |event| {
            // println!("i Source event = {event:#?}");
            match event {
                hotwatch::Event::Create(p) | hotwatch::Event::NoticeWrite(p) => {
                    eprintln!(
                        "{}",
                        format!("i File updated: {:?}", p.file_name().unwrap_or_default())
                    );
                    tx_watch_event_3.send(WatchEvent::FileUpdate(p)).unwrap();
                }
                hotwatch::Event::Remove(p) => {
                    eprintln!(
                        "{}",
                        format!("i File removed: {:?}", p.file_name().unwrap_or_default())
                    );
                    tx_watch_event_3.send(WatchEvent::FileRemoved(p)).unwrap();
                }
                hotwatch::Event::Rename(from, to) => {
                    eprintln!(
                        "{}",
                        format!("i File renamed: {:?}", from.file_name().unwrap_or_default())
                    );
                    tx_watch_event_3
                        .send(WatchEvent::FileRenamed(from, to))
                        .unwrap();
                }
                hotwatch::Event::Error(err, _p_opt) => {
                    panic!("{}", err);
                }
                // hotwatch::Event::Write(_) => todo!(),
                // hotwatch::Event::NoticeRemove(_) => todo!(),
                // hotwatch::Event::Chmod(_) => todo!(),
                // hotwatch::Event::Rescan => todo!(),
                _ => {}
            }
        })
        .expect("failed to watch files");
    eprintln!("{}", format!("✓ Watching source folder").bright_green());

    // SSE channel
    let (tx_event, _) = broadcast::channel(1);
    let sse = SeeEventManager::new(tx_event.clone());

    tokio::spawn(async move {
        loop {
            let event = rx_watch_event.recv().await.unwrap();
            match event {
                WatchEvent::TemplateUpdate => {
                    eprintln!("{}", format!("↳ Rebuilding all pages"));
                    builder.registry.reload_templates().unwrap();
                    match builder.build_all() {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("{}", "✗ Failed to rebuild page".bright_red());
                            eprintln!("{}", format!("{err}").bright_red());
                        }
                    };
                    sse.reload();
                }
                WatchEvent::FileUpdate(p) => {
                    eprintln!(
                        "{}",
                        format!("↳ Rebuilding page: {:?}", p.file_name().unwrap_or_default())
                    );
                    match builder.build_page(p) {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("{}", "✗ Failed to rebuild page".bright_red());
                            eprintln!("{}", format!("{err}").bright_red());
                        }
                    };
                    sse.reload();
                }
                WatchEvent::FileRemoved(p) => {
                    eprintln!(
                        "{}",
                        format!("↳ Removing page: {:?}", p.file_name().unwrap_or_default())
                    );
                    match builder.remove_page(p) {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("{}", "✗ Failed to remove page".bright_red());
                            eprintln!("{}", format!("{err}").bright_red());
                        }
                    };
                }
                WatchEvent::FileRenamed(from, to) => {
                    eprintln!(
                        "{}",
                        format!(
                            "↳ Renaming page: {:?}",
                            from.file_name().unwrap_or_default()
                        )
                    );
                    match builder.rename_page(from, to) {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!("{}", "✗ Failed to rename page".bright_red());
                            eprintln!("{}", format!("{err}").bright_red());
                        }
                    };
                }
            }
        }
    });

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

    // start server
    let build_dir = config.build_dir();
    eprintln!(
        "{}",
        format!("✓ File server started on {addr}").bright_green()
    );
    start_server(&build_dir, &addr, tx_event).await;
}

//! `docrs` executable

use clap::{Parser, Subcommand};

use docrs::cmd;

/// Generates a static site (docs, blog, etc...)
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Commands
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initializes the config
    Init(cmd::init::Args),
    /// Builds the site in dev mode
    Dev(cmd::dev::Args),
    /// Builds the site
    Build(cmd::build::Args),
    /// Serves the site
    Serve(cmd::serve::Args),
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init(args) => cmd::init::run(args).await,
        Commands::Dev(args) => cmd::dev::run(args).await,
        Commands::Build(args) => cmd::build::run(args).await,
        Commands::Serve(args) => cmd::serve::run(args).await,
    }
}

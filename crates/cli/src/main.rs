use std::process::exit;

use clap::Parser;
use cli::Cli;

mod cli;
mod cli_chatter;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = cli.run().await {
        eprintln!("An error occurred while running: {}", e);
        exit(1);
    }
}

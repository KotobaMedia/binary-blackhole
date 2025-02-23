use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::cli_chatter::CliChatter;

#[derive(Parser)]
#[command(name = "BinaryBlackhole CLI")]
#[command(about = "Run BinaryBlackhole on the command line")]
pub struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Run the chatter")]
    Chatter,
}

impl Cli {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            Some(Commands::Chatter) => {
                let mut chatter = CliChatter::new().await?;
                chatter.run().await?;
            }
            None => {
                println!("No command provided");
            }
        }
        Ok(())
    }
}

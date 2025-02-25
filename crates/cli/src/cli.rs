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
    Chatter {
        /// Show backend request messages during operation
        #[arg(long)]
        show_messages: bool,
    },
}

impl Cli {
    pub async fn run(&self) -> Result<()> {
        match &self.command {
            Some(Commands::Chatter { show_messages }) => {
                let mut chatter = CliChatter::new(*show_messages).await?;
                chatter.run().await?;
            }
            None => {
                println!("No command provided");
            }
        }
        Ok(())
    }
}

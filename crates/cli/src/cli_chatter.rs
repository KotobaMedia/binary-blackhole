use anyhow::{Ok, Result};
use chatter::chatter::Chatter;
use dialoguer::Input;

pub struct CliChatter {
    chatter: Chatter,
}

impl CliChatter {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            chatter: Chatter::new().await?,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        println!("Running the Chatter...");
        loop {
            self.run_one().await?;
        }
    }

    async fn run_one(&mut self) -> Result<()> {
        let msg = Input::<String>::new()
            .with_prompt("message")
            .interact_text()?;
        // println!("You said: {}", msg);
        self.chatter.context.add_user_message(&msg);
        let response = self.chatter.execute().await?;

        if let Some(content) = response.content {
            println!("→ {}", content);
        } else {
            println!("[no content] {:?}", response);
        }
        Ok(())
    }
}

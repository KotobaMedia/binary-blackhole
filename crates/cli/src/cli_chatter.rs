use anyhow::{Ok, Result};
use chatter::chatter::Chatter;
use dialoguer::Input;

pub struct CliChatter {
    chatter: Chatter,
    show_messages: bool,
}

impl CliChatter {
    pub async fn new(show_messages: bool) -> Result<Self> {
        Ok(Self {
            chatter: Chatter::new().await?,
            show_messages,
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

        if self.show_messages {
            self.chatter.context.messages.iter().for_each(|m| {
                println!("{}: {:?}", m.role, m);
            });
        }

        if let Some(content) = response.content {
            println!("→ {}", content);
        } else {
            println!("[no content] {:?}", response);
        }
        Ok(())
    }
}

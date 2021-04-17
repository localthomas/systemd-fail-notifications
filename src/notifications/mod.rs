use anyhow::{anyhow, Context, Result};
use discord::Discord;

use crate::{config::Config, status::UnitStatus};

pub mod discord;

pub trait NotificationProvider {
    // TODO: allow multiple Results?
    fn execute(&mut self, states: Vec<UnitStatus>) -> Result<()>;
}

pub fn create_notifications(config: &Config) -> Result<Vec<Box<dyn NotificationProvider>>> {
    let mut notifications: Vec<Box<dyn NotificationProvider>> = Vec::new();
    if let Some(discord_webhook_url) = &config.discord_webhook_url {
        let discord = Discord::new(discord_webhook_url)
            .context("could not create discord notification provider")?;
        notifications.push(Box::new(discord));
    }
    if notifications.len() == 0 {
        return Err(anyhow!(
            "no notification provider could be created. Is the configuration correctly set?"
        ));
    }
    Ok(notifications)
}

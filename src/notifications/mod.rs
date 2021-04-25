/*
SPDX-FileCopyrightText: 2021 localthomas

SPDX-License-Identifier: MIT OR Apache-2.0
*/

use anyhow::{anyhow, Context, Result};
use discord::Discord;

use crate::{config::Config, status::UnitStatus};

pub mod discord;

pub trait NotificationProvider {
    // TODO: allow multiple Results?
    fn execute(&self, states: Vec<UnitStatus>) -> Box<dyn Fn() -> Result<()> + '_ + Sync + Send>;
    fn execute_error(
        &self,
        error: &anyhow::Error,
    ) -> Box<dyn Fn() -> Result<()> + '_ + Sync + Send>;
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

fn http_post(
    url: &url::Url,
    query_params: Vec<(&str, &str)>,
    payload: serde_json::Value,
) -> Result<()> {
    let timeout_duration = std::time::Duration::from_secs(15);
    let agent = ureq::AgentBuilder::new()
        .timeout_read(timeout_duration)
        .timeout_write(timeout_duration)
        .build();

    let mut request = agent.request_url("POST", url);
    for query_param in query_params {
        request = request.query(query_param.0, query_param.1);
    }

    let response = request
        .send_json(payload)
        .context("could not execute POST on HTTP request")?;
    if response.status() < 200 || response.status() > 299 {
        return Err(anyhow!(
            "HTTP POST request had not-ok status code: {}",
            response.status()
        ));
    }
    Ok(())
}

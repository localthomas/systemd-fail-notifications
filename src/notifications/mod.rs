/*
SPDX-FileCopyrightText: 2021 localthomas

SPDX-License-Identifier: MIT OR Apache-2.0
*/

use anyhow::{anyhow, Context, Result};
use discord::Discord;

use crate::{config::Config, status::UnitStatus};

pub mod discord;

/// Provides execution closures for notifications of multiple events.
/// Should be thread safe, so that it can be invoked from multiple threads at the same time.
/// Ideally, the implementations of this trait only hold the configuration necessary to construct
/// the closures.
pub trait NotificationProvider: Send + Sync {
    // TODO: allow multiple Results?
    /// Execute produces a closure that when executed, notifies the user of the status of the given
    /// units.
    /// The closure that is created, can be executed in a different thread if desired.
    fn execute(&self, states: Vec<UnitStatus>) -> Box<dyn FnOnce() -> Result<()> + 'static + Send>;

    /// Produces a closure for notifying the user of an application error that ocurred in this program.
    /// Should be treated as alerts every time, if the notification system allows priority distinctions.
    fn execute_error(
        &self,
        error: &anyhow::Error,
    ) -> Box<dyn FnOnce() -> Result<()> + 'static + Send>;
}

/// Creates a default set of notification providers with the given configuration.
/// The list is not empty, if the `Ok` is returned.
/// Any errors during the creation of any notification provider is returned and no list is created.
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

/// Executes a generic HTTP POST request to the given URL and with the query parameters applied.
/// The payload is always JSON and the correct headers are automatically set for this type of payload.
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

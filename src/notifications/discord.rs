/*
SPDX-FileCopyrightText: 2021 localthomas

SPDX-License-Identifier: MIT OR Apache-2.0
*/

use anyhow::{Context, Result};
use serde_json::json;
use url::Url;

use crate::status::UnitStatus;

use super::NotificationProvider;

#[derive(Clone)]
pub struct Discord {
    webhook_url: Url,
}

impl Discord {
    /// Creates a new Discord notification provider with the given webhook URL as string.
    /// The string must be a in a valid format for an URL.
    pub fn new(webhook_url: &str) -> Result<Self> {
        let url = Url::parse(webhook_url).context(format!(
            "could not parse discord webhook url '{}'",
            webhook_url
        ))?;
        Ok(Self { webhook_url: url })
    }

    /// Sends the status of one unit to the specified Discord webhook URL.
    fn send_status(&self, status: &UnitStatus) -> Result<()> {
        let payload = DiscordMessage {
            content: "Unit Status changed!".to_string(),
            title: format!("{} has failed!", status.name()),
            description: "The following unit has entered a new state:".to_string(),
            color: 13631488,
            fields: vec![
                DiscordMessageField {
                    name: "Name".to_string(),
                    value: status.name().to_string(),
                },
                DiscordMessageField {
                    name: "Description".to_string(),
                    value: status.description().to_string(),
                },
                DiscordMessageField {
                    name: "Load State".to_string(),
                    value: format!("{}", status.load_state()),
                },
                DiscordMessageField {
                    name: "Active State".to_string(),
                    value: format!("{}", status.active_state()),
                },
                DiscordMessageField {
                    name: "Sub State".to_string(),
                    value: status.sub_state().to_string(),
                },
            ],
        };
        self.send(payload)
    }

    /// Sends the given webhook message for Discord to the configured webhook URL.
    fn send(&self, payload: DiscordMessage) -> Result<()> {
        super::http_post(&self.webhook_url, vec![("wait", "true")], payload.to_json())
            .context("could not execute discord webhook")?;
        Ok(())
    }
}

impl NotificationProvider for Discord {
    fn execute(&self, states: Vec<UnitStatus>) -> Box<dyn FnOnce() -> Result<()> + 'static + Send> {
        // to make the closure being able to be send to another thread,
        // the Discord config needs to be cloned, so that it can be transferred to the thread
        let new_self: Discord = (*self).clone();

        Box::new(move || {
            for status in &states {
                new_self.send_status(&status)?;
            }
            Ok(())
        })
    }

    fn execute_error(
        &self,
        error: &anyhow::Error,
    ) -> Box<dyn FnOnce() -> Result<()> + 'static + Send> {
        // to make the closure being able to be send to another thread,
        // the Discord config needs to be cloned, so that it can be transferred to the thread
        let new_self = (*self).clone();

        // create the description string outside the closure,
        // so that the error does not need to be send to the thread
        let description = format!("{:?}", error);

        Box::new(move || {
            let payload = DiscordMessage {
                content: "systemd-fail-notifications internal error!".to_string(),
                title: "Internal Error!".to_string(),
                description: description.clone(),
                color: 13631488,
                fields: vec![],
            };
            new_self.send(payload)
        })
    }
}

struct DiscordMessage {
    content: String,
    title: String,
    description: String,
    color: u32,
    fields: Vec<DiscordMessageField>,
}

struct DiscordMessageField {
    name: String,
    value: String,
}

impl DiscordMessage {
    fn to_json(&self) -> serde_json::Value {
        use chrono::{DateTime, Utc};
        let now = std::time::SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let fields: Vec<serde_json::Value> = self
            .fields
            .iter()
            .map(|field| {
                json!({
                    "name": field.name,
                    "value": field.value,
                    "inline": true,
                })
            })
            .collect();
        json!({
            "content": self.content,
            "embeds": [
                {
                    "author": {
                        "name": "systemd-fail-notifications",
                    },
                    "title": self.title,
                    "description": self.description,
                    "timestamp": now.to_rfc3339(),
                    "color": self.color,
                    "fields": fields,
                    "footer": {
                        "text": format!("message created at system time: {}", now.format("%Y-%m-%d %H:%M:%S %:z")),
                    },
                }
            ],
        })
    }
}

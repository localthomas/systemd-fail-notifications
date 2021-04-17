use anyhow::{anyhow, Context, Result};
use reqwest::Url;
use serde_json::json;

use crate::status::UnitStatus;

use super::NotificationProvider;

pub struct Discord {
    webhook_url: Url,
}

impl Discord {
    pub fn new(webhook_url: &str) -> Result<Self> {
        let url = Url::parse(webhook_url).context(format!(
            "could not parse discord webhook url '{}'",
            webhook_url
        ))?;
        Ok(Self { webhook_url: url })
    }

    fn send(&self, status: &UnitStatus) -> Result<()> {
        use chrono::{DateTime, Utc};
        let now = std::time::SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let json_value = json!({
            "content": "Unit Status changed!",
            "embeds": [
                {
                    "author": {
                        "name": "systemd-fail-notifications",
                    },
                    "title": format!("{} has failed!", status.name()),
                    "description": "The following unit has entered a new state:",
                    "timestamp": now.to_rfc3339(),
                    "color": 13631488,
                    "fields": [
                        {
                            "name": "Name",
                            "value": status.name(),
                            "inline": true,
                        },
                        {
                            "name": "Description",
                            "value": status.description(),
                            "inline": true,
                        },
                        {
                            "name": "Load State",
                            "value": format!("{}",status.load_state()),
                            "inline": true,
                        },
                        {
                            "name": "Active State",
                            "value": format!("{}", status.active_state()),
                            "inline": true,
                        },
                        {
                            "name": "Sub State",
                            "value": status.sub_state(),
                            "inline": true,
                        },
                    ],
                    "footer": {
                        "text": format!("message created at system time: {}", now.format("%Y-%m-%d %H:%M:%S %:z")),
                    },
                }
            ],
        });

        let client = reqwest::blocking::Client::new();
        let res = client
            .post(self.webhook_url.clone())
            .query(&[("wait", "true")])
            .json(&json_value)
            .timeout(std::time::Duration::from_secs(15))
            .send()
            .context("could not execute POST on discord webhook")?;
        if !res.status().is_success() {
            return Err(anyhow!(
                "discord webhook POST request had not-ok status code: {}",
                res.status()
            ));
        }
        Ok(())
    }
}

impl NotificationProvider for Discord {
    fn execute(&mut self, states: Vec<UnitStatus>) -> Result<()> {
        for status in states {
            self.send(&status)?;
        }
        Ok(())
    }
}
/*
SPDX-FileCopyrightText: 2021 localthomas

SPDX-License-Identifier: MIT OR Apache-2.0
*/

use anyhow::{anyhow, Result};

const DISCORD_WEBHOOK_URL: &str = "SYSTEMD_FAIL_NOTIFICATIONS_DISCORD_WEBHOOK_URL";

pub struct Config {
    pub discord_webhook_url: Option<String>,
}

impl Config {
    pub fn new() -> Result<Self> {
        let discord_webhook_url = env_var(DISCORD_WEBHOOK_URL)?;
        Ok(Self {
            discord_webhook_url,
        })
    }
}

fn env_var(key: &str) -> Result<Option<String>> {
    match std::env::var_os(key) {
        Some(string) => Ok(Some(match string.into_string() {
            Ok(string) => string,
            Err(_) => return Err(anyhow!("{} does not contain valid unicode data", key)),
        })),
        None => Ok(None),
    }
}

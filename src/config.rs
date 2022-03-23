/*
SPDX-FileCopyrightText: 2021 localthomas

SPDX-License-Identifier: MIT OR Apache-2.0
*/

use anyhow::Result;

/// Holds the static configuration for the program.
/// Can be used to alter the behavior of the execution or to configure notification provider.
pub struct Config {
    pub discord_webhook_url: Option<String>,
    pub about: bool,
    pub disable_start_notification: bool,
}

impl Config {
    /// Read command line arguments and flags, as well as environment variables to parse the configuration.
    pub fn new() -> Result<Self> {
        const ABOUT: (&str, char, &str) = (
            "about",
            'a',
            "if set, print the licensing information as HTML and exit",
        );
        const DISABLE_START_NOTIFICATION: (&str, &str) = (
            "disable-start-notification",
            "disables the initial notification about the application starting",
        );
        const DISCORD_WEBHOOK_URL: (&str, &str, &str) = (
            "discord-webhook-url",
            "SYSTEMD_FAIL_NOTIFICATIONS_DISCORD_WEBHOOK_URL",
            "the webhook-URL of the Discord webhook like 'https://discord.com/api/webhooks/<id>/<token>'",
        );
        use clap::{Arg, Command};
        let matches = Command::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .arg(
                Arg::new(ABOUT.0)
                    .short(ABOUT.1)
                    .long(ABOUT.0)
                    .help(ABOUT.2)
                    .takes_value(false),
            )
            .arg(
                Arg::new(DISABLE_START_NOTIFICATION.0)
                    .long(DISABLE_START_NOTIFICATION.0)
                    .help(DISABLE_START_NOTIFICATION.1)
                    .takes_value(false),
            )
            .arg(
                Arg::new(DISCORD_WEBHOOK_URL.0)
                    .long(DISCORD_WEBHOOK_URL.0)
                    .env(DISCORD_WEBHOOK_URL.1)
                    .help(DISCORD_WEBHOOK_URL.2)
                    .takes_value(true),
            )
            .get_matches();

        Ok(Self {
            discord_webhook_url: option_str_to_string(matches.value_of(DISCORD_WEBHOOK_URL.0)),
            about: matches.is_present(ABOUT.0),
            disable_start_notification: matches.is_present(DISABLE_START_NOTIFICATION.0),
        })
    }
}

fn option_str_to_string(value: Option<&str>) -> Option<String> {
    match value {
        Some(val) => Some(val.to_string()),
        None => None,
    }
}

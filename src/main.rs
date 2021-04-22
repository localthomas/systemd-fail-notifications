/*
SPDX-FileCopyrightText: 2021 localthomas

SPDX-License-Identifier: MIT OR Apache-2.0
*/

mod config;
mod dbus_systemd;
mod filter;
mod notifications;
mod state;
mod status;

use std::{thread, time};

use anyhow::{Context, Result};
use clap::Clap;
use config::Config;
use dbus_systemd::dbus::Connection;
use dbus_systemd::SystemdConnection;
use filter::FilterState;
use state::{AppState, SystemdState};
use status::UnitStatus;

/// systemd-fail-notifications is a standalone binary that listens on the system bus and
/// talks to systemd to identify failed units.
/// Any configuration is done using environment variables.
#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Options {
    /// if set, print the licensing information as HTML and exit
    #[clap(short, long)]
    about: bool,
}

fn main() -> Result<()> {
    let opts: Options = Options::parse();

    if opts.about {
        let about = include_str!("../license.html");
        println!("{}", about);
        return Ok(());
    }

    let mut filter = FilterState::new();
    let conn = Connection::new().context("could not create connection")?;
    let conf = Config::new().context("could not create configuration")?;
    let mut notifications = notifications::create_notifications(&conf)
        .context("could not create notifications provider")?;

    let mut state = SystemdState::new(|changes| {
        let filtered: Vec<UnitStatus> = changes
            .iter()
            .cloned()
            .filter(|status| filter.filter_function(status))
            .collect();
        if filtered.len() == 0 {
            return;
        }

        for service in &filtered {
            println!(
                "{} has changed states. Executing webhooks...",
                service.name()
            );
        }

        let notification_errors: Vec<anyhow::Error> = notifications
            .iter_mut()
            // TODO: execute notification in separate threads
            .map(|notification| notification.execute(filtered.clone()))
            .filter_map(|result| {
                if let Err(error) = result {
                    Some(error)
                } else {
                    None
                }
            })
            .collect();
        for error in notification_errors {
            // TODO: send notifications for errors occurring during notification sending
            println!("Error during notification: {:?}", error);
        }
    });
    looping(time::Duration::from_millis(2_000), move || {
        main_loop(&conn, &mut state).context("error during main loop")
    })?;
    Ok(())
}

fn main_loop<T: SystemdConnection>(conn: &T, state: &mut (dyn AppState)) -> Result<()> {
    let unit_status = conn.list_units().context("could not list units")?;
    let unit_status: Vec<UnitStatus> = unit_status.into_iter().map(UnitStatus::from).collect();
    state.apply_new_status(unit_status);
    Ok(())
}

fn looping<T: FnMut() -> Result<()>>(interval: time::Duration, mut function: T) -> Result<()> {
    loop {
        let start = time::Instant::now();
        function()?;
        // measure time and then sleep exact so long that the interval is met
        thread::sleep(interval - start.elapsed());
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use dbus_systemd::{dbus::UnitStatusRaw, tests::MockupSystemdConnection};

    use crate::state::tests::MockupAppState;

    use super::*;

    #[test]
    fn main_loop_return_on_error() {
        let mut conn = MockupSystemdConnection::new();
        conn.error = true;
        let mut state = MockupAppState::new();
        let result = main_loop(&conn, &mut state);
        assert_eq!(state.last_state, None);
        if let Err(err) = result {
            assert_eq!(err.root_cause().to_string(), anyhow!("test").to_string());
        } else {
            panic!("main_loop did not throw expected error");
        }
    }

    #[test]
    fn main_loop_new_empty_status_from_connection() {
        let mut conn = MockupSystemdConnection::new();
        conn.units = vec![];
        let mut state = MockupAppState::new();
        assert_eq!(state.last_state, None);
        main_loop(&conn, &mut state).expect("should not throw error");
        assert_eq!(state.last_state, Some(Vec::new()));
    }

    #[test]
    fn main_loop_new_status_from_connection() {
        const TEST: &str = "test";
        let mut conn = MockupSystemdConnection::new();
        let raw_unit = UnitStatusRaw {
            name: String::from(TEST),
            description: String::from(TEST),
            load_state: String::from(TEST),
            active_state: String::from(TEST),
            sub_state: String::from(TEST),
            following_unit: String::from(TEST),
        };
        conn.units = vec![raw_unit.clone()];
        let mut state = MockupAppState::new();
        assert_eq!(state.last_state, None);
        main_loop(&conn, &mut state).expect("should not throw error");
        assert_eq!(
            state.last_state,
            Some(vec![UnitStatus::from(raw_unit.clone())])
        );
    }
}

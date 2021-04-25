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
use notifications::NotificationProvider;
use state::{SystemdState, SystemdStateImpl};
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

struct AppState<'a, C, S>
where
    C: SystemdConnection,
    S: SystemdState,
{
    filter: FilterState<'a>,
    conn: C,
    notifications: Vec<Box<dyn NotificationProvider>>,
    systemd: S,
}

impl<'a, C, S> AppState<'a, C, S>
where
    C: SystemdConnection,
    S: SystemdState,
{
    fn poll_for_new_systemd_state(&mut self) -> Result<Vec<UnitStatus>> {
        let unit_status = self.conn.list_units().context("could not list units")?;
        let unit_status: Vec<UnitStatus> = unit_status.into_iter().map(UnitStatus::from).collect();
        let changes = self.systemd.apply_new_status(unit_status);
        let filtered: Vec<UnitStatus> = changes
            .iter()
            .cloned()
            .filter(|status| self.filter.filter_function(status))
            .collect();
        Ok(filtered)
    }

    fn notify(&self, status: Vec<UnitStatus>) {
        for service in &status {
            println!(
                "{} has changed states. Executing webhooks...",
                service.name()
            );
        }

        let mut errors = Vec::new();
        for notification in &self.notifications {
            let func = notification.execute(status.clone());
            // TODO: execute notification in separate threads; e.g. via a channel that is read in a thread created by a NotificationProvider
            match func() {
                Err(error) => {
                    eprintln!("Error during notification: {:?}", error);
                    errors.push(error);
                }
                Ok(_) => (),
            }
        }

        for error in errors {
            for notification in &self.notifications {
                let func = notification.execute_error(&error);
                // TODO: execute notification in separate threads
                match func() {
                    Err(error) => {
                        eprintln!(
                            "Error during notification for error during notification: {:?}",
                            error
                        );
                    }
                    Ok(_) => (),
                }
            }
        }
    }
}

fn initialize<'a>() -> Result<AppState<'a, Connection, SystemdStateImpl>> {
    let filter = FilterState::new();
    let conn = Connection::new().context("could not create connection")?;
    let conf = Config::new().context("could not create configuration")?;
    let notifications = notifications::create_notifications(&conf)
        .context("could not create notifications provider")?;
    let systemd = SystemdStateImpl::new();
    Ok(AppState {
        filter,
        conn,
        notifications,
        systemd,
    })
}

fn main() -> Result<()> {
    let opts: Options = Options::parse();

    if opts.about {
        let about = include_str!("../license.html");
        println!("{}", about);
        return Ok(());
    }

    let mut state = initialize()?;

    looping(time::Duration::from_millis(2_000), move || {
        main_loop(&mut state).context("error during main loop")
    })?;
    Ok(())
}

fn main_loop<C, S>(state: &mut AppState<'_, C, S>) -> Result<()>
where
    C: SystemdConnection,
    S: SystemdState,
{
    let status = state
        .poll_for_new_systemd_state()
        .context("could not poll for new systemd state")?;
    state.notify(status);
    /*
    let unit_status = conn.list_units().context("could not list units")?;
    let unit_status: Vec<UnitStatus> = unit_status.into_iter().map(UnitStatus::from).collect();
    let changes = state.apply_new_status(unit_status);
    on_changes(changes);
    */
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

    use crate::state::tests::MockupSystemdState;

    use super::*;

    #[test]
    fn main_loop_return_on_error() {
        let mut state = AppState {
            filter: FilterState::new(),
            conn: MockupSystemdConnection::new(),
            notifications: vec![],
            systemd: MockupSystemdState::new(),
        };
        state.conn.error = true;
        let result = main_loop(&mut state);
        assert_eq!(state.systemd.last_state, None);
        if let Err(err) = result {
            assert_eq!(err.root_cause().to_string(), anyhow!("test").to_string());
        } else {
            panic!("main_loop did not throw expected error");
        }
    }

    #[test]
    fn main_loop_new_empty_status_from_connection() {
        let mut state = AppState {
            filter: FilterState::new(),
            conn: MockupSystemdConnection::new(),
            notifications: vec![],
            systemd: MockupSystemdState::new(),
        };
        state.conn.units = vec![];
        assert_eq!(state.systemd.last_state, None);
        main_loop(&mut state).expect("should not throw error");
        assert_eq!(state.systemd.last_state, Some(Vec::new()));
    }

    #[test]
    fn main_loop_new_status_from_connection() {
        const TEST: &str = "test";
        let raw_unit = UnitStatusRaw {
            name: String::from(TEST),
            description: String::from(TEST),
            load_state: String::from(TEST),
            active_state: String::from(TEST),
            sub_state: String::from(TEST),
            following_unit: String::from(TEST),
        };
        let mut state = AppState {
            filter: FilterState::new(),
            conn: MockupSystemdConnection::new(),
            notifications: vec![],
            systemd: MockupSystemdState::new(),
        };
        state.conn.units = vec![raw_unit.clone()];
        assert_eq!(state.systemd.last_state, None);
        main_loop(&mut state).expect("should not throw error");
        assert_eq!(
            state.systemd.last_state,
            Some(vec![UnitStatus::from(raw_unit.clone())])
        );
    }
}

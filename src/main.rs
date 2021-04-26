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

use std::{sync::Arc, thread, time};

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

/// Holds the 'global' app internal state of the major sub-components.
/// This includes the D-Bus connection to systemd, the notification providers and the app-local
/// mirror of the state of systemd.
struct AppState<'a, C, S>
where
    C: SystemdConnection,
    S: SystemdState,
{
    filter: FilterState<'a>,
    conn: C,
    notifications: Arc<Vec<Box<dyn NotificationProvider>>>,
    systemd: S,
}

impl<'a, C, S> AppState<'a, C, S>
where
    C: SystemdConnection,
    S: SystemdState,
{
    /// Poll the system bus for new changes on the systemd daemon.
    /// The response includes all units of the current state and this function only returns the filtered
    /// unit status and only if they changed from the previous call to this function (hence the `&mut`).
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

    /// Execute notifications for the given status array that holds all relevant changes of units
    /// which the user is notified by all notification providers.
    ///
    /// Any errors during initial notification of the changed services are then also broadcasted by
    /// executing the error-notification of all notification providers.
    /// An error on one notification provider is send to all notification providers' error-notifies.
    fn notify(&self, status: Vec<UnitStatus>) {
        for service in &status {
            println!(
                "{} has changed states. Executing webhooks...",
                service.name()
            );
        }

        // execute for each notification provider the provided function that executes the notification
        // in a separate thread to prevent blocking the process in case of errors
        for notification in &*self.notifications {
            let func = notification.execute(status.clone());

            // clone the atomic reference for each thread that is spawned
            let notifications = self.notifications.clone();
            std::thread::spawn(move || match func() {
                // if an error occurs during the execution of the notification function,
                // execute a notification for the error itself
                Err(error) => {
                    eprintln!("Error during notification: {:?}", error);
                    for notification in &*notifications {
                        let func = notification.execute_error(&error);
                        // execute notification for error in separate thread
                        std::thread::spawn(move || match func() {
                            Err(error) => {
                                eprintln!(
                                    "Error during notification for error during notification: {:?}",
                                    error
                                );
                            }
                            Ok(_) => (),
                        });
                    }
                }
                Ok(_) => (),
            });
        }
    }
}

/// Initialize an AppState for specific implementations that are only suitable when executed on a machine
/// with systemd available.
/// Notable side-effect: Uses environment variables to read the configuration.
///
/// Not usable for unit tests, unless the presence of systemd can be verified.
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
        notifications: Arc::new(notifications),
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

/// Execute the typical workload for this daemon program for one iteration.
/// Designed to be periodically executed.
fn main_loop<C, S>(state: &mut AppState<'_, C, S>) -> Result<()>
where
    C: SystemdConnection,
    S: SystemdState,
{
    let status = state
        .poll_for_new_systemd_state()
        .context("could not poll for new systemd state")?;
    state.notify(status);
    Ok(())
}

/// Provides a timed loop, where each iteration is executed in the specified interval.
/// If the execution of the function in an iteration is taking longer than the specified interval
/// the next iteration follows promptly.
///
/// The endless loop is stopped on receiving an error from the iteration function.
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
            notifications: Arc::new(vec![]),
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
            notifications: Arc::new(vec![]),
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
            notifications: Arc::new(vec![]),
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

mod dbus_systemd;
mod status;

use std::collections::HashSet;

use anyhow::{Context, Result};
use dbus_systemd::dbus::Connection;
use dbus_systemd::SystemdConnection;
use status::UnitStatus;

#[derive(Debug)]
struct AppState {
    systemd_state: HashSet<UnitStatus>,
    // TODO: add filter function as field
}

impl AppState {
    fn new() -> Self {
        Self {
            systemd_state: HashSet::new(),
        }
    }

    fn apply_new_status(&mut self, new_status: &[UnitStatus]) {
        // TODO add filter field
        // TODO check for changes
        new_status
            .into_iter()
            .filter(|&status| status.name().ends_with(".service"))
            .for_each(|status| {
                self.systemd_state.insert(status.clone());
            });
    }
}

fn main() -> Result<()> {
    let conn = Connection::new().context("could not create connection")?;
    main_loop(conn).context("error during main loop")?;
    Ok(())
}

fn main_loop<T: SystemdConnection>(conn: T) -> Result<()> {
    let mut state = AppState::new();
    let unit_status = conn.list_units().context("could not list units")?;
    let unit_status: Vec<UnitStatus> = unit_status.into_iter().map(UnitStatus::new).collect();
    state.apply_new_status(&unit_status);
    dbg!(state);
    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use dbus_systemd::tests::MockupSystemdConnection;

    use super::*;

    #[test]
    fn main_loop_return_on_error() {
        let mut conn = MockupSystemdConnection::new();
        conn.error = true;
        let result = main_loop(conn);
        if let Err(err) = result {
            assert_eq!(err.root_cause().to_string(), anyhow!("test").to_string());
        } else {
            panic!("main_loop did not throw expected error");
        }
    }
}

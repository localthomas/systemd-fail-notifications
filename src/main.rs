mod dbus_systemd;
mod status;

use std::{collections::HashSet, thread};

use anyhow::{Context, Result};
use dbus_systemd::dbus::Connection;
use dbus_systemd::SystemdConnection;
use status::UnitStatus;

struct AppState<'a> {
    systemd_state: HashSet<UnitStatus>,
    // TODO: add filter function as field
    on_state_changed: Box<dyn Fn(&[&UnitStatus]) + 'a>,
}

impl<'a> AppState<'a> {
    fn new(on_state_changed: impl Fn(&[&UnitStatus]) + 'a) -> Self {
        Self {
            systemd_state: HashSet::new(),
            on_state_changed: Box::new(on_state_changed),
        }
    }

    fn apply_new_status(&mut self, new_status: &[UnitStatus]) {
        // TODO add filter field
        let mut new_state = HashSet::with_capacity(new_status.len());
        new_status
            .into_iter()
            .filter(|&status| status.name().ends_with(".service"))
            .for_each(|status| {
                new_state.insert(status.clone());
            });
        let changes: Vec<&UnitStatus> = new_state.difference(&self.systemd_state).collect();
        if changes.len() > 0 {
            (self.on_state_changed)(changes.as_slice());
        }
        self.systemd_state = new_state;
    }
}

fn main() -> Result<()> {
    let conn = Connection::new().context("could not create connection")?;
    main_loop(conn).context("error during main loop")?;
    Ok(())
}

fn main_loop<T: SystemdConnection>(conn: T) -> Result<()> {
    let mut state = AppState::new(|changes| {
        println!("{:?}", changes);
    });
    loop {
        let unit_status = conn.list_units().context("could not list units")?;
        let unit_status: Vec<UnitStatus> = unit_status.into_iter().map(UnitStatus::new).collect();
        state.apply_new_status(&unit_status);
        thread::sleep(std::time::Duration::from_millis(1_000));
    }
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

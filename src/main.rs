mod dbus_systemd;
mod filter;
mod state;
mod status;

use std::thread;

use anyhow::{Context, Result};
use dbus_systemd::dbus::Connection;
use dbus_systemd::SystemdConnection;
use filter::FilterState;
use state::{AppState, SystemdState};
use status::UnitStatus;

fn main() -> Result<()> {
    let mut filter = FilterState::new();
    let conn = Connection::new().context("could not create connection")?;
    let mut state = SystemdState::new(|changes| {
        changes
            .iter()
            .filter(|status| filter.filter_function(status))
            .for_each(|status| {
                println!("{:#?}", status);
            });
    });
    looping(move || main_loop(&conn, &mut state).context("error during main loop"))?;
    Ok(())
}

fn main_loop<T: SystemdConnection>(conn: &T, state: &mut (dyn AppState)) -> Result<()> {
    let unit_status = conn.list_units().context("could not list units")?;
    let unit_status: Vec<UnitStatus> = unit_status.into_iter().map(UnitStatus::from).collect();
    state.apply_new_status(unit_status);
    Ok(())
}

fn looping<T: FnMut() -> Result<()>>(mut function: T) -> Result<()> {
    loop {
        function()?;
        // measure time and then sleep exact
        thread::sleep(std::time::Duration::from_millis(1_000));
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

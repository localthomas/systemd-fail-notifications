mod dbus_systemd;
mod state;
mod status;

use std::thread;

use anyhow::{Context, Result};
use dbus_systemd::dbus::Connection;
use dbus_systemd::SystemdConnection;
use state::AppState;
use status::UnitStatus;

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
        state.apply_new_status(unit_status);
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

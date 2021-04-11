mod dbus_systemd;

use anyhow::{Context, Result};
use dbus_systemd::dbus::Connection;
use dbus_systemd::SystemdConnection;

fn main() -> Result<()> {
    let conn = Connection::new().context("could not create connection")?;
    main_loop(conn).context("error during main loop")?;
    Ok(())
}

fn main_loop<T: SystemdConnection + Sized>(conn: T) -> Result<()> {
    let unit_status = conn.list_units().context("could not list units")?;
    dbg!(unit_status);
    let unit_status = conn.list_units().context("could not list units")?;
    dbg!(unit_status);
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

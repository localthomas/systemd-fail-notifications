mod dbus_systemd;

use anyhow::Result;
use dbus_systemd::connection::Connection;
use dbus_systemd::SystemdConnection;

fn main() {
    let conn = Connection::new().expect("could not create connection");
    main_loop(conn).expect("error during main loop");
}

fn main_loop<T: SystemdConnection + Sized>(conn: T) -> Result<()> {
    let unit_status = conn.list_units().expect("could not list units");
    dbg!(unit_status);
    let unit_status = conn.list_units().expect("could not list units");
    dbg!(unit_status);
    Ok(())
}

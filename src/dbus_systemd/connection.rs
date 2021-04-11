use anyhow::{Context, Result};
use zbus::export::zvariant::derive::Type;
use zbus::export::zvariant::export::serde::Deserialize;
use zbus::export::zvariant::{self, OwnedObjectPath};

use super::SystemdConnection;

pub struct Connection {
    conn: zbus::Connection,
}

impl SystemdConnection for Connection {
    fn new() -> Result<Self> {
        let conn = zbus::Connection::new_system().context("could not connect to system bus")?;
        Ok(Self { conn })
    }

    fn list_units(&self) -> Result<Vec<UnitStatus>> {
        let message = self
            .conn
            .call_method(
                Some("org.freedesktop.systemd1"),
                "/org/freedesktop/systemd1",
                Some("org.freedesktop.systemd1.Manager"),
                "ListUnits",
                &(),
            )
            .context("could not make method call to ListUnits")?;
        let unit_status: Vec<UnitStatus> = message
            .body()
            .context("could not deserialize the message from dbus")?;
        Ok(unit_status)
    }
}

#[derive(Deserialize, Debug, Type)]
pub struct UnitStatus {
    name: String,
    description: String,
    load_state: String,
    active_state: String,
    sub_state: String,
    following_unit: String,
    unit_object_path: OwnedObjectPath,
    job_queued_id: u32,
    job_type: String,
    job_object_path: OwnedObjectPath,
}

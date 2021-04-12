use anyhow::{Context, Result};
use zbus::export::zvariant::derive::Type;
use zbus::export::zvariant::export::serde::Deserialize;
use zbus::export::zvariant::{self, OwnedObjectPath};

use super::SystemdConnection;

pub struct Connection {
    conn: zbus::Connection,
}

impl Connection {
    pub fn new() -> Result<Self> {
        let conn = zbus::Connection::new_system().context("could not connect to system bus")?;
        Ok(Self { conn })
    }
}

impl SystemdConnection for Connection {
    fn list_units(&self) -> Result<Vec<UnitStatusRaw>> {
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
        let unit_status: Vec<UnitStatusRaw> = message
            .body()
            .context("could not deserialize the message from dbus")?;
        Ok(unit_status)
    }
}

#[derive(Deserialize, Debug, Type, Clone)]
pub struct UnitStatusRaw {
    pub name: String,
    pub description: String,
    pub load_state: String,
    pub active_state: String,
    pub sub_state: String,
    pub following_unit: String,
    pub unit_object_path: OwnedObjectPath,
    pub job_queued_id: u32,
    pub job_type: String,
    pub job_object_path: OwnedObjectPath,
}

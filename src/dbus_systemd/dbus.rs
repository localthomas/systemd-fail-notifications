/*
SPDX-FileCopyrightText: 2021 localthomas

SPDX-License-Identifier: MIT OR Apache-2.0
*/

use anyhow::{Context, Result};
use zbus::export::zvariant::derive::Type;
use zbus::export::zvariant::export::serde::Deserialize;
use zbus::export::zvariant::OwnedObjectPath;

use super::SystemdConnection;

pub struct Connection {
    conn: zbus::Connection,
}

impl Connection {
    pub fn new() -> Result<Self> {
        let conn = zbus::Connection::new_system().context(format!(
            "could not connect to system bus at {}",
            dbus_system_address()
        ))?;
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
        let unit_status: Vec<UnitStatusInternal> = message
            .body()
            .context("could not deserialize the message from dbus")?;
        Ok(unit_status.into_iter().map(UnitStatusRaw::from).collect())
    }
}

fn dbus_system_address() -> String {
    match std::env::var("DBUS_SYSTEM_BUS_ADDRESS") {
        Ok(val) => val,
        _ => "/var/run/dbus/system_bus_socket".to_string(),
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
}

impl From<UnitStatusInternal> for UnitStatusRaw {
    fn from(status: UnitStatusInternal) -> Self {
        Self {
            name: status.name,
            description: status.description,
            load_state: status.load_state,
            active_state: status.active_state,
            sub_state: status.sub_state,
            following_unit: status.following_unit,
        }
    }
}

// Note that this struct is used for serialization and therefore each field is important, even if it is not used.
#[allow(dead_code)]
#[derive(Deserialize, Debug, Type, Clone)]
struct UnitStatusInternal {
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

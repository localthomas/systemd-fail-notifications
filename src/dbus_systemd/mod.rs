pub mod connection;

use anyhow::Result;
use connection::UnitStatus;

pub trait SystemdConnection {
    fn new() -> Result<Self>
    where
        Self: Sized;
    fn list_units(&self) -> Result<Vec<UnitStatus>>;
}

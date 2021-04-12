pub mod dbus;

use anyhow::Result;
use dbus::UnitStatusRaw;

pub trait SystemdConnection {
    fn list_units(&self) -> Result<Vec<UnitStatusRaw>>;
}

#[cfg(test)]
pub mod tests {
    use anyhow::anyhow;

    use super::*;

    pub struct MockupSystemdConnection {
        pub units: Vec<UnitStatusRaw>,
        pub error: bool,
    }

    impl MockupSystemdConnection {
        pub fn new() -> Self {
            Self {
                units: Vec::new(),
                error: false,
            }
        }
    }

    impl SystemdConnection for MockupSystemdConnection {
        fn list_units(&self) -> Result<Vec<UnitStatusRaw>> {
            if self.error {
                Err(anyhow!("test"))
            } else {
                Ok(self.units.to_vec())
            }
        }
    }
}

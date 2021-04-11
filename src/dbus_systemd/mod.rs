pub mod dbus;

use anyhow::Result;
use dbus::UnitStatus;

pub trait SystemdConnection {
    fn list_units(&self) -> Result<Vec<UnitStatus>>;
}

#[cfg(test)]
pub mod tests {
    use anyhow::anyhow;

    use super::*;

    pub struct MockupSystemdConnection {
        pub units: Vec<UnitStatus>,
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
        fn list_units(&self) -> Result<Vec<UnitStatus>> {
            if self.error {
                Err(anyhow!("test"))
            } else {
                Ok(self.units.to_vec())
            }
        }
    }
}

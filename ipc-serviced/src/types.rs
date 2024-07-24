use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Command {
    PowerOff,
    Reboot,
    /// pid of the service
    ServiceReady(u32),
}

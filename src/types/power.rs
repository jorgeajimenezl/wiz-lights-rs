//! Power mode for light control.

use serde::{Deserialize, Serialize};

/// Power state for a light.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PowerMode {
    /// Reboot the light
    Reboot,
    /// Turn the light on
    On,
    /// Turn the light off
    Off,
}

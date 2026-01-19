//! Fan control types for fan-equipped Wiz fixtures.

use serde::{Deserialize, Serialize};

/// Fan power state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FanState {
    #[default]
    Off = 0,
    On = 1,
}

impl FanState {
    pub fn value(self) -> u8 {
        self as u8
    }
}

impl From<bool> for FanState {
    fn from(on: bool) -> Self {
        if on { FanState::On } else { FanState::Off }
    }
}

/// Fan operating mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FanMode {
    #[default]
    Normal = 1,
    Breeze = 2,
}

impl FanMode {
    pub fn value(self) -> u8 {
        self as u8
    }
}

/// Fan rotation direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FanDirection {
    #[default]
    Forward = 0,
    Reverse = 1,
}

impl FanDirection {
    pub fn value(self) -> u8 {
        self as u8
    }
}

/// Fan speed (typically 1-6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FanSpeed {
    pub(crate) value: u8,
}

impl FanSpeed {
    /// Default maximum fan speed.
    pub const DEFAULT_MAX: u8 = 6;

    /// Create a fan speed. Returns None if out of range (1 to max_speed).
    pub fn create(value: u8, max_speed: Option<u8>) -> Option<Self> {
        let max = max_speed.unwrap_or(Self::DEFAULT_MAX);
        if (1..=max).contains(&value) {
            Some(FanSpeed { value })
        } else {
            None
        }
    }

    pub fn value(self) -> u8 {
        self.value
    }
}

//! Brightness control for Wiz lights.

use serde::{Deserialize, Serialize};

/// Brightness level from 10 to 100 percent.
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Brightness {
    pub(crate) value: u8,
}

impl Brightness {
    const MIN: u8 = 10;
    const MAX: u8 = 100;

    pub fn new() -> Self {
        Brightness { value: Self::MAX }
    }

    pub fn value(&self) -> u8 {
        self.value
    }

    /// Returns None if value is outside valid range (10-100).
    pub fn create(value: u8) -> Option<Self> {
        if Self::is_valid(value) {
            Some(Brightness { value })
        } else {
            None
        }
    }

    /// Returns default (100%) if value is invalid.
    pub fn create_or(value: u8) -> Self {
        if Self::is_valid(value) {
            Brightness { value }
        } else {
            Self::new()
        }
    }

    fn is_valid(value: u8) -> bool {
        (Self::MIN..=Self::MAX).contains(&value)
    }
}

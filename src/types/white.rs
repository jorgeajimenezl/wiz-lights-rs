//! White LED channel control.

use serde::{Deserialize, Serialize};

/// White LED intensity for cool or warm white channels, from 1 to 100 percent.
///
/// Some Wiz bulbs have separate cool and warm white LED channels that can be
/// controlled independently of the RGB LEDs. This provides more accurate
/// white light reproduction than mixing RGB.
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct White {
    pub(crate) value: u8,
}

impl White {
    const MIN: u8 = 1;
    const MAX: u8 = 100;

    /// Create a new White with the default value (100%).
    pub fn new() -> Self {
        White { value: Self::MAX }
    }

    /// Get the white value.
    pub fn value(&self) -> u8 {
        self.value
    }

    /// Create a new White with the given value.
    ///
    /// Returns `None` if value is outside the valid range (1-100).
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::White;
    ///
    /// assert!(White::create(0).is_none());
    /// assert!(White::create(1).is_some());
    /// assert!(White::create(100).is_some());
    /// assert!(White::create(101).is_none());
    /// ```
    pub fn create(value: u8) -> Option<Self> {
        if (Self::MIN..=Self::MAX).contains(&value) {
            Some(White { value })
        } else {
            None
        }
    }
}

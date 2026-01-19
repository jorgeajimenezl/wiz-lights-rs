//! Ratio control for dual-head fixtures.

use serde::{Deserialize, Serialize};

/// Ratio for dual-head fixtures, controlling the balance between up and down lights.
///
/// Valid values are 0 to 100, where:
/// - 0 = all light directed downward
/// - 50 = balanced between up and down
/// - 100 = all light directed upward
///
/// This only applies to fixtures with dual-head lighting (e.g., floor lamps with
/// both up-lighting and down-lighting capabilities).
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Ratio {
    pub(crate) value: u8,
}

impl Ratio {
    const DEFAULT: u8 = 50;
    const MAX: u8 = 100;

    /// Create a new Ratio with the default value (50 = balanced).
    pub fn new() -> Self {
        Ratio {
            value: Self::DEFAULT,
        }
    }

    /// Get the ratio value.
    pub fn value(&self) -> u8 {
        self.value
    }

    /// Create a new Ratio with the given value.
    ///
    /// Returns `None` if value exceeds 100.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::Ratio;
    ///
    /// assert!(Ratio::create(0).is_some());
    /// assert!(Ratio::create(50).is_some());
    /// assert!(Ratio::create(100).is_some());
    /// assert!(Ratio::create(101).is_none());
    /// ```
    pub fn create(value: u8) -> Option<Self> {
        if value <= Self::MAX {
            Some(Ratio { value })
        } else {
            None
        }
    }
}

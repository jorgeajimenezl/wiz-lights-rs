//! Color temperature control.

use serde::{Deserialize, Serialize};

/// Color temperature in Kelvin, with valid values from 1000K to 8000K.
///
/// Lower values produce warmer (more yellow/orange) light, while higher
/// values produce cooler (more blue) light. Typical values:
/// - 2700K: Warm white (incandescent-like)
/// - 4000K: Neutral white
/// - 6500K: Daylight
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Kelvin {
    pub(crate) kelvin: u16,
}

impl Kelvin {
    const MIN: u16 = 1000;
    const MAX: u16 = 8000;

    /// Create a new Kelvin with the default value (1000K).
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::Kelvin;
    ///
    /// assert_eq!(Kelvin::new().kelvin(), 1000);
    /// ```
    pub fn new() -> Self {
        Kelvin { kelvin: Self::MIN }
    }

    /// Get the kelvin value.
    pub fn kelvin(&self) -> u16 {
        self.kelvin
    }

    /// Create a new Kelvin with the given value.
    ///
    /// Returns `None` if value is outside the valid range (1000-8000).
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::Kelvin;
    ///
    /// assert!(Kelvin::create(999).is_none());
    /// assert!(Kelvin::create(1000).is_some());
    /// assert!(Kelvin::create(8000).is_some());
    /// assert!(Kelvin::create(8001).is_none());
    /// ```
    pub fn create(kelvin: u16) -> Option<Self> {
        if (Self::MIN..=Self::MAX).contains(&kelvin) {
            Some(Kelvin { kelvin })
        } else {
            None
        }
    }
}

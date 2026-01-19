//! Brightness control for Wiz lights.

use serde::{Deserialize, Serialize};

/// Brightness level for a light, with valid values from 10 to 100 percent.
///
/// Brightness can be combined with any other lighting mode (color, scene, etc.).
/// The Wiz bulbs do not support brightness below 10%.
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Brightness {
    pub(crate) value: u8,
}

impl Brightness {
    const MIN: u8 = 10;
    const MAX: u8 = 100;

    /// Create a new Brightness with the default value (100%).
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::Brightness;
    ///
    /// let brightness = Brightness::new();
    /// assert_eq!(brightness.value(), 100);
    /// ```
    pub fn new() -> Self {
        Brightness { value: Self::MAX }
    }

    /// Get the brightness value.
    pub fn value(&self) -> u8 {
        self.value
    }

    /// Create a new Brightness with the given value.
    ///
    /// Returns `None` if value is outside the valid range (10-100).
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::Brightness;
    ///
    /// assert!(Brightness::create(9).is_none());
    /// assert!(Brightness::create(10).is_some());
    /// assert!(Brightness::create(100).is_some());
    /// assert!(Brightness::create(101).is_none());
    /// ```
    pub fn create(value: u8) -> Option<Self> {
        if Self::is_valid(value) {
            Some(Brightness { value })
        } else {
            None
        }
    }

    /// Create a Brightness, using default if value is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::Brightness;
    ///
    /// assert_eq!(Brightness::create_or(9).value(), 100);
    /// assert_eq!(Brightness::create_or(10).value(), 10);
    /// assert_eq!(Brightness::create_or(100).value(), 100);
    /// assert_eq!(Brightness::create_or(101).value(), 100);
    /// ```
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

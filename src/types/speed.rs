//! Animation speed for dynamic scenes.

use serde::{Deserialize, Serialize};

/// Animation speed for dynamic scenes, with valid values from 20 to 200 percent.
///
/// Speed only affects scenes with animation (like Party, Ocean, etc.).
/// A value of 100 is the default speed; lower values slow the animation,
/// higher values speed it up.
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Speed {
    pub(crate) value: u8,
}

impl Speed {
    const MIN: u8 = 20;
    const MAX: u8 = 200;
    const DEFAULT: u8 = 100;

    /// Create a new Speed with the default value (100%).
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::Speed;
    ///
    /// assert_eq!(Speed::new().value(), 100);
    /// ```
    pub fn new() -> Self {
        Speed { value: Self::DEFAULT }
    }

    /// Get the speed value.
    pub fn value(&self) -> u8 {
        self.value
    }

    /// Create a new Speed with the given value.
    ///
    /// Returns `None` if value is outside the valid range (20-200).
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::Speed;
    ///
    /// assert!(Speed::create(19).is_none());
    /// assert!(Speed::create(20).is_some());
    /// assert!(Speed::create(200).is_some());
    /// assert!(Speed::create(201).is_none());
    /// ```
    pub fn create(value: u8) -> Option<Self> {
        if Self::is_valid(value) {
            Some(Speed { value })
        } else {
            None
        }
    }

    /// Create a Speed, using default if value is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::Speed;
    ///
    /// assert_eq!(Speed::create_or(19).value(), 100);
    /// assert_eq!(Speed::create_or(20).value(), 20);
    /// assert_eq!(Speed::create_or(200).value(), 200);
    /// assert_eq!(Speed::create_or(201).value(), 100);
    /// ```
    pub fn create_or(value: u8) -> Self {
        if Self::is_valid(value) {
            Speed { value }
        } else {
            Self::new()
        }
    }

    fn is_valid(value: u8) -> bool {
        (Self::MIN..=Self::MAX).contains(&value)
    }
}

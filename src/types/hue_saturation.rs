//! Hue and Saturation color representation.

use super::Color;

/// Hue and Saturation color representation.
///
/// An alternative way to specify colors using:
/// - Hue: The color angle on the color wheel (0-360 degrees)
/// - Saturation: The intensity of the color (0-100 percent)
///
/// This is commonly used in color pickers and provides a more intuitive
/// way to select colors than RGB values.
#[derive(Debug, Clone, Default)]
pub struct HueSaturation {
    hue: u16,
    saturation: u8,
}

impl HueSaturation {
    /// Create a new HueSaturation with the given values.
    ///
    /// # Arguments
    ///
    /// * `hue` - Hue angle in degrees (0-360)
    /// * `saturation` - Saturation percentage (0-100)
    ///
    /// Returns `None` if values are outside valid ranges.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::HueSaturation;
    ///
    /// assert!(HueSaturation::create(0, 100).is_some());   // Red at full saturation
    /// assert!(HueSaturation::create(120, 50).is_some()); // Green at 50% saturation
    /// assert!(HueSaturation::create(361, 50).is_none()); // Invalid hue
    /// assert!(HueSaturation::create(180, 101).is_none()); // Invalid saturation
    /// ```
    pub fn create(hue: u16, saturation: u8) -> Option<Self> {
        if hue <= 360 && saturation <= 100 {
            Some(HueSaturation { hue, saturation })
        } else {
            None
        }
    }

    /// Get the hue value.
    pub fn hue(&self) -> u16 {
        self.hue
    }

    /// Get the saturation value.
    pub fn saturation(&self) -> u8 {
        self.saturation
    }

    /// Convert to RGB Color.
    ///
    /// Uses HSV to RGB conversion with Value fixed at 255 (max brightness).
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::HueSaturation;
    ///
    /// let hs = HueSaturation::create(0, 100).unwrap();
    /// let color = hs.to_color();
    /// assert_eq!(color.red(), 255);
    /// assert_eq!(color.green(), 0);
    /// assert_eq!(color.blue(), 0);
    /// ```
    pub fn to_color(&self) -> Color {
        let h = self.hue as f32;
        let s = self.saturation as f32 / 100.0;
        let v = 1.0;

        if s == 0.0 {
            let gray = (v * 255.0) as u8;
            return Color::rgb(gray, gray, gray);
        }

        let h = h / 60.0;
        let i = h.floor() as i32;
        let f = h - i as f32;
        let p = v * (1.0 - s);
        let q = v * (1.0 - s * f);
        let t = v * (1.0 - s * (1.0 - f));

        let (r, g, b) = match i % 6 {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };

        Color::rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }
}

impl From<&HueSaturation> for Color {
    fn from(hs: &HueSaturation) -> Self {
        hs.to_color()
    }
}

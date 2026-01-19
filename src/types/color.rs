//! RGB, RGBW, and RGBWW color representations.

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// An RGB color with red, green, and blue components (0-255 each).
#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Color {
    pub(crate) red: u8,
    pub(crate) green: u8,
    pub(crate) blue: u8,
}

impl Color {
    /// Create a color with the given RGB values.
    pub fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    /// Create a default color (black: 0,0,0).
    pub fn new() -> Self {
        Self::default()
    }

    pub fn red(&self) -> u8 {
        self.red
    }

    pub fn green(&self) -> u8 {
        self.green
    }

    pub fn blue(&self) -> u8 {
        self.blue
    }
}

impl FromStr for Color {
    type Err = String;

    /// Parse from comma-separated string (e.g., "255,128,0").
    fn from_str(s: &str) -> Result<Self, String> {
        let parts: Vec<u8> = s.split(',').map(|c| c.parse().unwrap_or(0)).collect();
        if parts.len() == 3 {
            Ok(Self::rgb(parts[0], parts[1], parts[2]))
        } else {
            Err("Expected format: r,g,b".into())
        }
    }
}

/// An RGBW color (RGB + warm white, 0-255 each).
#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ColorRGBW {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub warm: u8,
}

impl ColorRGBW {
    pub fn new(red: u8, green: u8, blue: u8, warm: u8) -> Self {
        Self {
            red,
            green,
            blue,
            warm,
        }
    }

    pub fn to_rgb(&self) -> Color {
        Color::rgb(self.red, self.green, self.blue)
    }
}

impl FromStr for ColorRGBW {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let parts: Vec<u8> = s.split(',').map(|c| c.parse().unwrap_or(0)).collect();
        if parts.len() == 4 {
            Ok(Self::new(parts[0], parts[1], parts[2], parts[3]))
        } else {
            Err("Expected format: r,g,b,w".into())
        }
    }
}

/// An RGBWW color (RGB + cool white + warm white, 0-255 each).
#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ColorRGBWW {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub cool: u8,
    pub warm: u8,
}

impl ColorRGBWW {
    pub fn new(red: u8, green: u8, blue: u8, cool: u8, warm: u8) -> Self {
        Self {
            red,
            green,
            blue,
            cool,
            warm,
        }
    }

    pub fn to_rgb(&self) -> Color {
        Color::rgb(self.red, self.green, self.blue)
    }

    pub fn to_rgbw(&self) -> ColorRGBW {
        ColorRGBW::new(self.red, self.green, self.blue, self.warm)
    }
}

impl FromStr for ColorRGBWW {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let parts: Vec<u8> = s.split(',').map(|c| c.parse().unwrap_or(0)).collect();
        if parts.len() == 5 {
            Ok(Self::new(parts[0], parts[1], parts[2], parts[3], parts[4]))
        } else {
            Err("Expected format: r,g,b,c,w".into())
        }
    }
}

//! # wiz_lights_rs
//!
//! A Rust library for controlling Philips Wiz smart lights over UDP.
//!
//! This crate provides a simple API to communicate with Wiz smart bulbs on your
//! local network. It supports setting colors, brightness, color temperature,
//! scenes, and power states.
//!
//! ## Quick Start
//!
//! ```no_run
//! use std::net::Ipv4Addr;
//! use std::str::FromStr;
//! use wiz_lights_rs::{Light, Payload, Color};
//!
//! // Create a light instance with the bulb's IP address
//! let light = Light::new(Ipv4Addr::from_str("192.168.1.100").unwrap(), Some("Living Room"));
//!
//! // Set the light to blue
//! let mut payload = Payload::new();
//! payload.color(&Color::from_str("0,0,255").unwrap());
//! // light.set(&payload).unwrap();
//! ```
//!
//! ## Features
//!
//! - **RGB Colors**: Set any RGB color using the [`Color`] type
//! - **Brightness**: Control brightness from 10-100% using [`Brightness`]
//! - **Color Temperature**: Set warm to cool white (1000K-8000K) using [`Kelvin`]
//! - **Scenes**: Use preset lighting scenes with [`SceneMode`]
//! - **Power Control**: Turn lights on/off or reboot with [`PowerMode`]
//! - **Room Grouping**: Organize lights into [`Room`]s for batch operations
//! - **Discovery**: Find bulbs on your network with [`discover_bulbs`]
//! - **Hue/Saturation**: Alternative color mode with [`HueSaturation`]
//!
//! ## Communication
//!
//! All communication with Wiz bulbs occurs over UDP on port 38899. The bulbs must
//! be on the same local network and ideally have static IP addresses assigned.

mod config;
mod discovery;
mod errors;
mod history;
mod light;
mod payload;
pub mod push;
mod response;
mod room;
mod status;
mod types;

// Re-export public API
pub use config::{BulbClass, BulbType, ExtendedWhiteRange, Features, KelvinRange, SystemConfig, WhiteRange};
pub use discovery::{discover_bulbs, DiscoveredBulb};
pub use errors::Error;
pub use history::{HistoryEntry, HistorySummary, MessageHistory, MessageType};
pub use light::Light;
pub use payload::Payload;
pub use response::LightingResponse;
pub use room::Room;
pub use status::{LastSet, LightStatus};
pub use types::{
    Brightness, Color, ColorRGBW, ColorRGBWW, FanDirection, FanMode, FanSpeed, FanState,
    HueSaturation, Kelvin, PowerMode, Ratio, SceneMode, Speed, White,
};

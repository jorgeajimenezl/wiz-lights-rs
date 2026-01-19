//! # wiz_lights_rs
//!
//! An async Rust library for controlling Philips Wiz smart lights over UDP.
//!
//! This crate provides a **runtime-agnostic** async API to communicate with Wiz smart bulbs
//! on your local network. It supports setting colors, brightness, color temperature,
//! scenes, and power states.
//!
//! ## Quick Start
//!
//! ```ignore
//! use std::net::Ipv4Addr;
//! use std::str::FromStr;
//! use wiz_lights_rs::{Light, Payload, Color};
//!
//! // Works with any async runtime!
//! async fn control_light() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a light instance with the bulb's IP address
//!     let light = Light::new(Ipv4Addr::from_str("192.168.1.100")?, Some("Living Room"));
//!
//!     // Set the light to blue
//!     let mut payload = Payload::new();
//!     payload.color(&Color::from_str("0,0,255")?);
//!     light.set(&payload).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - **Runtime Agnostic**: Works with tokio, async-std, or smol async runtimes
//! - **RGB Colors**: Set any RGB color using the [`Color`] type
//! - **Brightness**: Control brightness from 10-100% using [`Brightness`]
//! - **Color Temperature**: Set warm to cool white (1000K-8000K) using [`Kelvin`]
//! - **Scenes**: Use preset lighting scenes with [`SceneMode`]
//! - **Power Control**: Turn lights on/off or reboot with [`PowerMode`]
//! - **Room Grouping**: Organize lights into [`Room`]s for batch operations
//! - **Discovery**: Find bulbs on your network with [`discover_bulbs`]
//! - **Hue/Saturation**: Alternative color mode with [`HueSaturation`]
//! - **Push Notifications**: Real-time state updates via [`push::PushManager`]
//!
//! ## Communication
//!
//! All communication with Wiz bulbs occurs over UDP on port 38899. The bulbs must
//! be on the same local network and ideally have static IP addresses assigned.
//!
//! ## Runtime Selection
//!
//! This library is runtime-agnostic. Select your preferred runtime using feature flags:
//!
//! ### Using tokio (default)
//!
//! ```toml
//! [dependencies]
//! wiz-lights-rs = "0.1"
//! tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
//! ```
//!
//! ### Using async-std
//!
//! ```toml
//! [dependencies]
//! wiz-lights-rs = { version = "0.1", default-features = false, features = ["runtime-async-std"] }
//! async-std = { version = "1.12", features = ["attributes"] }
//! ```
//!
//! ### Using smol
//!
//! ```toml
//! [dependencies]
//! wiz-lights-rs = { version = "0.1", default-features = false, features = ["runtime-smol"] }
//! smol = "2"
//! ```
//!
//! ## Feature Flags
//!
//! - `runtime-tokio` (default): Use the tokio async runtime
//! - `runtime-async-std`: Use the async-std runtime
//! - `runtime-smol`: Use the smol runtime

mod config;
mod discovery;
mod errors;
mod history;
mod light;
mod payload;
pub mod push;
mod response;
mod room;
pub mod runtime;
mod status;
mod types;

// Re-export public API
pub use config::{
    BulbClass, BulbType, ExtendedWhiteRange, Features, KelvinRange, SystemConfig, WhiteRange,
};
pub use discovery::{DiscoveredBulb, discover_bulbs};
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

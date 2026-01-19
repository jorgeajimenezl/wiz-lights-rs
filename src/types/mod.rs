//! Value types for light control parameters.

mod brightness;
mod color;
mod fan;
mod hue_saturation;
mod kelvin;
mod power;
mod ratio;
mod scene;
mod speed;
mod white;

pub use brightness::Brightness;
pub use color::{Color, ColorRGBW, ColorRGBWW};
pub use fan::{FanDirection, FanMode, FanSpeed, FanState};
pub use hue_saturation::HueSaturation;
pub use kelvin::Kelvin;
pub use power::PowerMode;
pub use ratio::Ratio;
pub use scene::SceneMode;
pub use speed::Speed;
pub use white::White;

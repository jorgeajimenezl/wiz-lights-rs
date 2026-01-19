//! Configuration payload for Wiz lights.

use serde::{Deserialize, Serialize};

use crate::types::{
    Brightness, Color, ColorRGBW, ColorRGBWW, FanDirection, FanMode, FanSpeed, FanState,
    HueSaturation, Kelvin, Ratio, SceneMode, Speed, White,
};

/// A configuration payload to send to Wiz lights.
///
/// Payloads can contain multiple lighting attributes (color, brightness, scene, etc.)
/// that will be applied to the bulb in a single command.
///
/// # Creating Payloads
///
/// You can create a payload in two ways:
///
/// 1. **From a single attribute** using the [`From`] trait:
///    ```
///    use wiz_lights_rs::{Payload, SceneMode};
///    let payload = Payload::from(&SceneMode::Sunset);
///    ```
///
/// 2. **Builder pattern** for combining multiple attributes:
///    ```
///    use std::str::FromStr;
///    use wiz_lights_rs::{Payload, Brightness, Color};
///    let mut payload = Payload::new();
///    payload.brightness(&Brightness::create(80).unwrap());
///    payload.color(&Color::from_str("255,128,0").unwrap());
///    ```
#[serde_with::skip_serializing_none]
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Payload {
    #[serde(rename = "sceneId")]
    pub(crate) scene: Option<u16>,
    pub(crate) dimming: Option<u8>,
    pub(crate) speed: Option<u8>,
    pub(crate) temp: Option<u16>,
    pub(crate) ratio: Option<u8>,
    #[serde(rename = "r")]
    pub(crate) red: Option<u8>,
    #[serde(rename = "g")]
    pub(crate) green: Option<u8>,
    #[serde(rename = "b")]
    pub(crate) blue: Option<u8>,
    #[serde(rename = "c")]
    pub(crate) cool: Option<u8>,
    #[serde(rename = "w")]
    pub(crate) warm: Option<u8>,
    // Fan control parameters
    #[serde(rename = "fanState")]
    pub(crate) fan_state: Option<u8>,
    #[serde(rename = "fanMode")]
    pub(crate) fan_mode: Option<u8>,
    #[serde(rename = "fanSpeed")]
    pub(crate) fan_speed: Option<u8>,
    #[serde(rename = "fanRevrs")]
    pub(crate) fan_reverse: Option<u8>,
}

impl Payload {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if at least one lighting attribute is set.
    pub fn is_valid(&self) -> bool {
        self.scene.is_some()
            || self.dimming.is_some()
            || self.temp.is_some()
            || (self.red.is_some() && self.green.is_some() && self.blue.is_some())
            || self.cool.is_some()
            || self.warm.is_some()
    }

    pub fn scene(&mut self, scene: &SceneMode) {
        self.scene = Some(scene.id());
    }

    pub fn brightness(&mut self, brightness: &Brightness) {
        self.dimming = Some(brightness.value);
    }

    pub fn speed(&mut self, speed: &Speed) {
        self.speed = Some(speed.value);
    }

    pub fn temp(&mut self, temp: &Kelvin) {
        self.temp = Some(temp.kelvin);
    }

    pub fn color(&mut self, color: &Color) {
        self.red = Some(color.red);
        self.green = Some(color.green);
        self.blue = Some(color.blue);
    }

    pub fn color_rgbw(&mut self, color: &ColorRGBW) {
        self.red = Some(color.red);
        self.green = Some(color.green);
        self.blue = Some(color.blue);
        self.warm = Some(color.warm);
    }

    pub fn color_rgbww(&mut self, color: &ColorRGBWW) {
        self.red = Some(color.red);
        self.green = Some(color.green);
        self.blue = Some(color.blue);
        self.cool = Some(color.cool);
        self.warm = Some(color.warm);
    }

    pub fn hue_saturation(&mut self, hs: &HueSaturation) {
        self.color(&hs.to_color());
    }

    pub fn cool(&mut self, cool: &White) {
        self.cool = Some(cool.value);
    }

    pub fn warm(&mut self, warm: &White) {
        self.warm = Some(warm.value);
    }

    pub fn ratio(&mut self, ratio: &Ratio) {
        self.ratio = Some(ratio.value);
    }

    pub fn fan_state(&mut self, state: &FanState) {
        self.fan_state = Some(state.value());
    }

    pub fn fan_mode(&mut self, mode: &FanMode) {
        self.fan_mode = Some(mode.value());
    }

    pub fn fan_speed(&mut self, speed: &FanSpeed) {
        self.fan_speed = Some(speed.value());
    }

    pub fn fan_direction(&mut self, direction: &FanDirection) {
        self.fan_reverse = Some(direction.value());
    }

    pub(crate) fn get_color(&self) -> Option<Color> {
        match (self.red, self.green, self.blue) {
            (Some(r), Some(g), Some(b)) => Some(Color::rgb(r, g, b)),
            _ => None,
        }
    }
}

impl From<&SceneMode> for Payload {
    fn from(scene: &SceneMode) -> Self {
        let mut p = Payload::new();
        p.scene(scene);
        p
    }
}

impl From<&Kelvin> for Payload {
    fn from(kelvin: &Kelvin) -> Self {
        let mut p = Payload::new();
        p.temp(kelvin);
        p
    }
}

impl From<&Color> for Payload {
    fn from(color: &Color) -> Self {
        let mut p = Payload::new();
        p.color(color);
        p
    }
}

impl From<&Speed> for Payload {
    fn from(speed: &Speed) -> Self {
        let mut p = Payload::new();
        p.speed(speed);
        p
    }
}

impl From<&Brightness> for Payload {
    fn from(brightness: &Brightness) -> Self {
        let mut p = Payload::new();
        p.brightness(brightness);
        p
    }
}

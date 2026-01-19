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
    /// Create a new empty payload.
    ///
    /// At least one attribute must be set for the payload to be valid.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::Payload;
    ///
    /// let mut payload = Payload::new();
    /// assert_eq!(payload.is_valid(), false);
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if this payload contains at least one valid attribute.
    ///
    /// Note: Speed alone is not valid; it must be combined with a scene.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::{Payload, SceneMode, Speed};
    ///
    /// let mut payload = Payload::new();
    ///
    /// payload.speed(&Speed::create(100).unwrap());
    /// assert_eq!(payload.is_valid(), false);
    ///
    /// payload.scene(&SceneMode::Focus);
    /// assert_eq!(payload.is_valid(), true);
    /// ```
    pub fn is_valid(&self) -> bool {
        self.scene.is_some()
            || self.dimming.is_some()
            || self.temp.is_some()
            || (self.red.is_some() && self.green.is_some() && self.blue.is_some())
            || self.cool.is_some()
            || self.warm.is_some()
    }

    /// Set the scene mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::{Payload, SceneMode};
    ///
    /// let mut payload = Payload::new();
    /// payload.scene(&SceneMode::Focus);
    /// assert_eq!(payload.is_valid(), true);
    /// ```
    pub fn scene(&mut self, scene: &SceneMode) {
        self.scene = Some(scene.id());
    }

    /// Set the brightness level.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::{Payload, Brightness};
    ///
    /// let mut payload = Payload::new();
    /// payload.brightness(&Brightness::create(100).unwrap());
    /// assert_eq!(payload.is_valid(), true);
    /// ```
    pub fn brightness(&mut self, brightness: &Brightness) {
        self.dimming = Some(brightness.value);
    }

    /// Set the animation speed.
    ///
    /// Speed is only effective when combined with a scene mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::Ipv4Addr;
    /// use std::str::FromStr;
    /// use wiz_lights_rs::{Light, Payload, LastSet, Color, Speed, LightingResponse};
    ///
    /// let ip = Ipv4Addr::from_str("10.1.2.3").unwrap();
    /// let mut light = Light::new(ip, None);
    ///
    /// let mut payload = Payload::new();
    /// payload.speed(&Speed::create(100).unwrap());
    /// payload.color(&Color::from_str("0,0,255").unwrap());
    ///
    /// let resp = LightingResponse::payload(ip, payload);
    /// assert!(light.process_reply(&resp));
    ///
    /// let status = light.status().unwrap();
    /// assert_eq!(status.last().unwrap(), &LastSet::Color);
    /// assert_eq!(status.speed().unwrap().value(), 100);
    /// ```
    pub fn speed(&mut self, speed: &Speed) {
        self.speed = Some(speed.value);
    }

    /// Set the color temperature.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::{Payload, Kelvin};
    ///
    /// let mut payload = Payload::new();
    /// payload.temp(&Kelvin::create(4000).unwrap());
    /// assert_eq!(payload.is_valid(), true);
    /// ```
    pub fn temp(&mut self, temp: &Kelvin) {
        self.temp = Some(temp.kelvin);
    }

    /// Set the RGB color.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use wiz_lights_rs::{Payload, Color};
    ///
    /// let mut payload = Payload::new();
    /// payload.color(&Color::from_str("255,255,255").unwrap());
    /// assert_eq!(payload.is_valid(), true);
    /// ```
    pub fn color(&mut self, color: &Color) {
        self.red = Some(color.red);
        self.green = Some(color.green);
        self.blue = Some(color.blue);
    }

    /// Set the RGBW color (RGB + warm white).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use wiz_lights_rs::{Payload, ColorRGBW};
    ///
    /// let mut payload = Payload::new();
    /// payload.color_rgbw(&ColorRGBW::from_str("255,128,0,50").unwrap());
    /// assert_eq!(payload.is_valid(), true);
    /// ```
    pub fn color_rgbw(&mut self, color: &ColorRGBW) {
        self.red = Some(color.red);
        self.green = Some(color.green);
        self.blue = Some(color.blue);
        self.warm = Some(color.warm);
    }

    /// Set the RGBWW color (RGB + cool white + warm white).
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use wiz_lights_rs::{Payload, ColorRGBWW};
    ///
    /// let mut payload = Payload::new();
    /// payload.color_rgbww(&ColorRGBWW::from_str("255,128,0,30,50").unwrap());
    /// assert_eq!(payload.is_valid(), true);
    /// ```
    pub fn color_rgbww(&mut self, color: &ColorRGBWW) {
        self.red = Some(color.red);
        self.green = Some(color.green);
        self.blue = Some(color.blue);
        self.cool = Some(color.cool);
        self.warm = Some(color.warm);
    }

    /// Set the color using hue and saturation.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::{Payload, HueSaturation};
    ///
    /// let mut payload = Payload::new();
    /// payload.hue_saturation(&HueSaturation::create(0, 100).unwrap());
    /// assert_eq!(payload.is_valid(), true);
    /// ```
    pub fn hue_saturation(&mut self, hs: &HueSaturation) {
        self.color(&hs.to_color());
    }

    /// Set the cool white intensity.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::{Payload, White};
    ///
    /// let mut payload = Payload::new();
    /// payload.cool(&White::create(50).unwrap());
    /// assert_eq!(payload.is_valid(), true);
    /// ```
    pub fn cool(&mut self, cool: &White) {
        self.cool = Some(cool.value);
    }

    /// Set the warm white intensity.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::{Payload, White};
    ///
    /// let mut payload = Payload::new();
    /// payload.warm(&White::create(50).unwrap());
    /// assert_eq!(payload.is_valid(), true);
    /// ```
    pub fn warm(&mut self, warm: &White) {
        self.warm = Some(warm.value);
    }

    /// Set the ratio for dual-head fixtures.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::{Payload, Ratio, Brightness};
    ///
    /// let mut payload = Payload::new();
    /// payload.brightness(&Brightness::new());
    /// payload.ratio(&Ratio::create(75).unwrap());
    /// assert_eq!(payload.is_valid(), true);
    /// ```
    pub fn ratio(&mut self, ratio: &Ratio) {
        self.ratio = Some(ratio.value);
    }

    /// Set the fan power state.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::{Payload, FanState};
    ///
    /// let mut payload = Payload::new();
    /// payload.fan_state(&FanState::On);
    /// ```
    pub fn fan_state(&mut self, state: &FanState) {
        self.fan_state = Some(state.value());
    }

    /// Set the fan operating mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::{Payload, FanMode};
    ///
    /// let mut payload = Payload::new();
    /// payload.fan_mode(&FanMode::Breeze);
    /// ```
    pub fn fan_mode(&mut self, mode: &FanMode) {
        self.fan_mode = Some(mode.value());
    }

    /// Set the fan speed.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::{Payload, FanSpeed};
    ///
    /// let mut payload = Payload::new();
    /// let speed = FanSpeed::create(3, None).unwrap();
    /// payload.fan_speed(&speed);
    /// ```
    pub fn fan_speed(&mut self, speed: &FanSpeed) {
        self.fan_speed = Some(speed.value());
    }

    /// Set the fan rotation direction.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::{Payload, FanDirection};
    ///
    /// let mut payload = Payload::new();
    /// payload.fan_direction(&FanDirection::Reverse);
    /// ```
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

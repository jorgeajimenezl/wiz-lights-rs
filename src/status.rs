//! Light status tracking.

use serde::{Deserialize, Serialize};

use crate::payload::Payload;
use crate::types::{Brightness, Color, Kelvin, PowerMode, SceneMode, Speed, White};

/// The last context set on the light that the API is aware of.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LastSet {
    /// The last set context was an RGB color
    Color,
    /// The last set context was a SceneMode
    Scene,
    /// The last set context was a Kelvin temperature
    Temp,
    /// The last set context was a cool white value
    Cool,
    /// The last set context was a warm white value
    Warm,
}

impl LastSet {
    pub(crate) fn from_payload(payload: &Payload) -> Option<Self> {
        if payload.scene.is_some() {
            return Some(LastSet::Scene);
        }
        if payload.get_color().is_some() {
            return Some(LastSet::Color);
        }
        if payload.temp.is_some() {
            return Some(LastSet::Temp);
        }
        if payload.cool.is_some() {
            return Some(LastSet::Cool);
        }
        if payload.warm.is_some() {
            return Some(LastSet::Warm);
        }
        None
    }
}

/// Tracks the last known settings for a light bulb.
#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LightStatus {
    color: Option<Color>,
    brightness: Option<Brightness>,
    emitting: bool,
    scene: Option<SceneMode>,
    speed: Option<Speed>,
    temp: Option<Kelvin>,
    cool: Option<White>,
    warm: Option<White>,
    last: Option<LastSet>,
}

impl LightStatus {
    /// Get the last set context.
    pub fn last(&self) -> Option<&LastSet> {
        self.last.as_ref()
    }

    /// Get the last set color.
    pub fn color(&self) -> Option<&Color> {
        self.color.as_ref()
    }

    /// Get the last set brightness.
    pub fn brightness(&self) -> Option<&Brightness> {
        self.brightness.as_ref()
    }

    /// Check if the light is emitting.
    pub fn emitting(&self) -> bool {
        self.emitting
    }

    /// Get the last set scene.
    pub fn scene(&self) -> Option<&SceneMode> {
        self.scene.as_ref()
    }

    /// Get the last set speed.
    pub fn speed(&self) -> Option<&Speed> {
        self.speed.as_ref()
    }

    /// Get the last set temperature.
    pub fn temp(&self) -> Option<&Kelvin> {
        self.temp.as_ref()
    }

    /// Get the last set cool white value.
    pub fn cool(&self) -> Option<&White> {
        self.cool.as_ref()
    }

    /// Get the last set warm white value.
    pub fn warm(&self) -> Option<&White> {
        self.warm.as_ref()
    }

    /// Update this status with values from another status.
    ///
    /// Values set in `other` overwrite values in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::{LightStatus, Payload, Speed, Kelvin};
    ///
    /// let mut status = LightStatus::from(&Payload::from(&Kelvin::new()));
    /// assert_eq!(status.temp().unwrap().kelvin(), 1000);
    /// assert!(status.speed().is_none());
    ///
    /// status.update(&LightStatus::from(&Payload::from(&Speed::new())));
    /// assert_eq!(status.temp().unwrap().kelvin(), 1000);
    /// assert_eq!(status.speed().unwrap().value(), 100);
    /// ```
    pub fn update(&mut self, other: &Self) {
        if let Some(color) = &other.color {
            self.color = Some(color.clone());
        }
        if let Some(brightness) = &other.brightness {
            self.brightness = Some(brightness.clone());
        }
        self.emitting = other.emitting;
        self.scene.clone_from(&other.scene);
        if let Some(speed) = &other.speed {
            self.speed = Some(speed.clone());
        }
        if let Some(temp) = &other.temp {
            self.temp = Some(temp.clone());
        }
        if let Some(cool) = &other.cool {
            self.cool = Some(cool.clone());
        }
        if let Some(warm) = &other.warm {
            self.warm = Some(warm.clone());
        }
        if let Some(last) = &other.last {
            self.last = Some(last.clone());
        }
    }

    pub(crate) fn update_from_payload(&mut self, payload: &Payload) {
        if let Some(color) = payload.get_color() {
            self.color = Some(color);
            self.last = Some(LastSet::Color);
        }
        if let Some(dimming) = payload.dimming {
            self.brightness = Brightness::create(dimming);
        }
        if let Some(speed) = payload.speed {
            self.speed = Speed::create(speed);
        }
        if let Some(temp) = payload.temp {
            self.temp = Kelvin::create(temp);
            self.last = Some(LastSet::Temp);
        }
        if let Some(scene) = payload.scene {
            self.scene = SceneMode::create(scene);
            self.last = Some(LastSet::Scene);
        }
        if let Some(cool) = payload.cool {
            self.cool = White::create(cool);
            self.last = Some(LastSet::Cool);
        }
        if let Some(warm) = payload.warm {
            self.warm = White::create(warm);
            self.last = Some(LastSet::Warm);
        }
    }

    pub(crate) fn update_from_power(&mut self, power: &PowerMode) {
        self.emitting = !matches!(power, PowerMode::Off);
    }
}

impl From<&Payload> for LightStatus {
    fn from(payload: &Payload) -> Self {
        LightStatus {
            color: payload.get_color(),
            brightness: payload.dimming.and_then(Brightness::create),
            emitting: true,
            scene: payload.scene.and_then(SceneMode::create),
            speed: payload.speed.and_then(Speed::create),
            temp: payload.temp.and_then(Kelvin::create),
            cool: payload.cool.and_then(White::create),
            warm: payload.warm.and_then(White::create),
            last: LastSet::from_payload(payload),
        }
    }
}

impl From<&PowerMode> for LightStatus {
    fn from(power: &PowerMode) -> Self {
        LightStatus {
            color: None,
            brightness: None,
            emitting: !matches!(power, PowerMode::Off),
            scene: None,
            speed: None,
            temp: None,
            cool: None,
            warm: None,
            last: None,
        }
    }
}

impl From<&BulbStatus> for LightStatus {
    fn from(bulb: &BulbStatus) -> Self {
        let res = &bulb.result;

        LightStatus {
            color: res.get_color(),
            brightness: res.dimming.and_then(Brightness::create),
            cool: res.cool.and_then(White::create),
            warm: res.warm.and_then(White::create),
            emitting: res.emitting,
            scene: SceneMode::create(res.scene),
            speed: None,
            temp: None,
            last: None,
        }
    }
}

/// Bulb status as reported by the bulb via getPilot.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct BulbStatus {
    pub env: String,
    pub method: String,
    pub result: BulbStatusResult,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct BulbStatusResult {
    #[serde(rename = "r")]
    pub red: Option<u8>,
    #[serde(rename = "g")]
    pub green: Option<u8>,
    #[serde(rename = "b")]
    pub blue: Option<u8>,
    pub dimming: Option<u8>,
    pub mac: String,
    #[serde(rename = "state")]
    pub emitting: bool,
    #[serde(rename = "sceneId")]
    pub scene: u16,
    pub rssi: i32,
    #[serde(rename = "c")]
    pub cool: Option<u8>,
    #[serde(rename = "w")]
    pub warm: Option<u8>,
}

impl BulbStatusResult {
    pub fn get_color(&self) -> Option<Color> {
        match (self.red, self.green, self.blue) {
            (Some(r), Some(g), Some(b)) => Some(Color::rgb(r, g, b)),
            _ => None,
        }
    }
}

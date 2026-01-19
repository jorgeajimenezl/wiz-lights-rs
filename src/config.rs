//! Bulb configuration and type detection.

use serde::{Deserialize, Serialize};

/// System configuration of a Wiz bulb.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SystemConfig {
    pub mac: String,
    #[serde(default)]
    pub home_id: Option<u64>,
    #[serde(default)]
    pub room_id: Option<u64>,
    #[serde(default)]
    pub module_name: Option<String>,
    #[serde(default)]
    pub fw_version: Option<String>,
    #[serde(default)]
    pub group_id: Option<u64>,
    #[serde(default)]
    pub type_id: Option<u32>,
    #[serde(default)]
    pub ping: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct SystemConfigResponse {
    pub method: String,
    pub result: SystemConfig,
}

/// Classification of Wiz bulb types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BulbClass {
    TW,      // Tunable White
    DW,      // Dimmable White
    RGB,     // Full color
    Socket,  // Smart socket
    FanDim,  // Fan with dimmable light
}

/// Feature flags for a Wiz bulb.
#[derive(Debug, Clone, Default)]
pub struct Features {
    pub color: bool,
    pub color_tmp: bool,
    pub effect: bool,
    pub brightness: bool,
    pub dual_head: bool,
    pub fan: bool,
    pub fan_breeze_mode: bool,
    pub fan_reverse: bool,
}

/// Color temperature range (Kelvin).
#[derive(Debug, Clone, Copy, Default)]
pub struct KelvinRange {
    pub min: u16,
    pub max: u16,
}

/// White range values from user config.
#[derive(Debug, Clone)]
pub struct WhiteRange {
    pub values: Vec<f32>,
}

impl WhiteRange {
    pub fn new(values: Vec<f32>) -> Self {
        Self { values }
    }
}

/// Extended white range (CCT range) - typically [warm_min, warm_max, cool_min, cool_max].
#[derive(Debug, Clone)]
pub struct ExtendedWhiteRange {
    pub values: Vec<f32>,
}

impl ExtendedWhiteRange {
    pub fn new(values: Vec<f32>) -> Self {
        Self { values }
    }
}

/// Complete type information for a Wiz bulb.
#[derive(Debug, Clone)]
pub struct BulbType {
    pub features: Features,
    pub name: String,
    pub kelvin_range: KelvinRange,
    pub bulb_class: BulbClass,
    pub fw_version: Option<String>,
    pub white_channels: u8,
}

impl BulbType {
    /// Parse bulb type from module name (e.g., "ESP01_SHRGB1C_31").
    pub fn from_module_name(module_name: &str, fw_version: Option<&str>) -> Self {
        let parts: Vec<&str> = module_name.split('_').collect();
        let mut features = Features::default();
        let mut bulb_class = BulbClass::DW;
        let mut kelvin_range = KelvinRange { min: 2700, max: 6500 };
        let mut white_channels = 0u8;

        if let Some(type_part) = parts.get(1) {
            features.dual_head = type_part.starts_with("DH");

            if type_part.contains("RGB") {
                bulb_class = BulbClass::RGB;
                features.color = true;
                features.color_tmp = true;
                features.effect = true;
                features.brightness = true;
                white_channels = 2;
                kelvin_range = KelvinRange { min: 2200, max: 6500 };
            } else if type_part.contains("TW") {
                bulb_class = BulbClass::TW;
                features.color_tmp = true;
                features.brightness = true;
                features.effect = true;
                white_channels = 2;
            } else if type_part.contains("DW") || type_part.contains("SHDW") {
                bulb_class = BulbClass::DW;
                features.brightness = true;
                white_channels = 1;
            } else if type_part.contains("SOCKET") {
                bulb_class = BulbClass::Socket;
            } else if type_part.contains("FANDIM") {
                bulb_class = BulbClass::FanDim;
                features.brightness = true;
                features.fan = true;
                features.fan_breeze_mode = true;
                features.fan_reverse = true;
                white_channels = 1;
            }
        }

        BulbType {
            features,
            name: module_name.to_string(),
            kelvin_range,
            bulb_class,
            fw_version: fw_version.map(String::from),
            white_channels,
        }
    }
}

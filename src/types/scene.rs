//! Preset lighting scenes.

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// Preset lighting scenes with static colors or dynamic animations.
#[derive(Debug, Serialize, Deserialize, Clone, EnumIter, PartialEq)]
pub enum SceneMode {
    Ocean = 1,
    Romance = 2,
    Sunset = 3,
    Party = 4,
    Fireplace = 5,
    Cozy = 6,
    Forest = 7,
    PastelColors = 8,
    WakeUp = 9,
    Bedtime = 10,
    WarmWhite = 11,
    Daylight = 12,
    CoolWhite = 13,
    NightLight = 14,
    Focus = 15,
    Relax = 16,
    TrueColors = 17,
    TvTime = 18,
    Plantgrowth = 19,
    Spring = 20,
    Summer = 21,
    Fall = 22,
    Deepdive = 23,
    Jungle = 24,
    Mojito = 25,
    Club = 26,
    Christmas = 27,
    Halloween = 28,
    Candlelight = 29,
    GoldenWhite = 30,
    Pulse = 31,
    Steampunk = 32,
    Diwali = 33,
    Alarm = 35,
    WarmFeeling = 36,
    Rhythm = 1000,
}

impl SceneMode {
    pub fn create(value: u16) -> Option<Self> {
        SceneMode::iter().find(|scene| scene.clone() as u16 == value)
    }

    pub fn id(&self) -> u16 {
        self.clone() as u16
    }
}

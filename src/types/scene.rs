//! Preset lighting scenes.

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// Preset lighting scenes built into Wiz bulbs.
///
/// Each scene produces a specific lighting effect, ranging from static colors
/// to dynamic animations. Scene IDs correspond to the official Wiz app scenes.
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
    /// Rhythm mode - syncs with music (requires Wiz app rhythm feature)
    Rhythm = 1000,
}

impl SceneMode {
    /// Create a SceneMode from its numeric ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use wiz_lights_rs::SceneMode;
    ///
    /// assert!(SceneMode::create(1).is_some());  // Ocean
    /// assert!(SceneMode::create(35).is_some()); // Alarm
    /// assert!(SceneMode::create(1000).is_some()); // Rhythm
    /// assert!(SceneMode::create(999).is_none());
    /// ```
    pub fn create(value: u16) -> Option<Self> {
        SceneMode::iter().find(|scene| scene.clone() as u16 == value)
    }

    /// Get the scene ID as u16.
    pub fn id(&self) -> u16 {
        self.clone() as u16
    }
}

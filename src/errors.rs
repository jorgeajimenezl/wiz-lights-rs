use std::{net::Ipv4Addr, string::FromUtf8Error};

use uuid::Uuid;

/// All error types that can occur when interacting with Wiz lights.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to serialize data to JSON.
    #[error("failed to dump json: {0:?}")]
    JsonDump(serde_json::Error),

    /// Failed to deserialize JSON data.
    #[error("failed to load json: {0:?}")]
    JsonLoad(serde_json::Error),

    /// A network socket operation failed while communicating with a bulb.
    #[error("socket {action} error: {err:?}")]
    Socket { action: String, err: std::io::Error },

    /// The UDP response from a bulb contained invalid UTF-8.
    #[error("utf8 decoding error: {0:?}")]
    Utf8Decode(FromUtf8Error),

    /// Attempted to send a [`crate::Payload`] with no attributes set.
    #[error("invalid payload; no attributes set")]
    NoAttribute,

    /// The specified room does not exist.
    #[error("room not found {0}")]
    RoomNotFound(Uuid),

    /// The specified light does not exist in the given room.
    #[error("light {light_id:?} not found in room {room_id:?}")]
    LightNotFound { room_id: Uuid, light_id: Uuid },

    /// The provided IP address is invalid (e.g., already in use).
    #[error("light with ip {ip} is invalid because the IP is {reason}")]
    InvalidIP { ip: Ipv4Addr, reason: String },

    /// The room update would result in no changes.
    #[error("no change for room {0}")]
    NoChangeRoom(Uuid),

    /// The light update would result in no changes.
    #[error("no change for light {light_id:?} in room {room_id:?}")]
    NoChangeLight { room_id: Uuid, light_id: Uuid },

    /// Attempted to modify a light in a room that has no lights.
    #[error("no lights in room {0}")]
    NoLights(Uuid),

    /// Failed to parse a [`crate::Color`] from a string.
    #[error("invalid color string: {0}")]
    InvalidColorString(String),
}

impl Error {
    /// Create a new socket error
    pub fn socket(action: &str, err: std::io::Error) -> Self {
        Error::Socket {
            action: action.to_string(),
            err,
        }
    }

    /// Create a new light not found error
    pub fn light_not_found(room_id: &Uuid, light_id: &Uuid) -> Self {
        Error::LightNotFound {
            room_id: *room_id,
            light_id: *light_id,
        }
    }

    /// Create a new invalid IP error
    pub fn invalid_ip(ip: &Ipv4Addr, reason: &str) -> Self {
        Error::InvalidIP {
            ip: *ip,
            reason: reason.to_string(),
        }
    }

    /// Create a new no change light error
    pub fn no_change_light(room_id: &Uuid, light_id: &Uuid) -> Self {
        Error::NoChangeLight {
            room_id: *room_id,
            light_id: *light_id,
        }
    }
}

/// Hacky implementation of PartialEq for testing
#[cfg(test)]
impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

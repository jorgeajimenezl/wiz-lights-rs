//! Lighting response types.

use std::net::Ipv4Addr;

use crate::payload::Payload;
use crate::status::LightStatus;
use crate::types::PowerMode;

/// A response from a lighting command.
///
/// Pass this to [`Light::process_reply`](crate::Light::process_reply) to update
/// the internal status cache after sending commands to bulbs.
#[derive(Debug)]
pub struct LightingResponse {
    pub(crate) ip: Ipv4Addr,
    pub(crate) response: LightingResponseType,
}

impl LightingResponse {
    /// Create a response from a payload.
    pub fn payload(ip: Ipv4Addr, payload: Payload) -> Self {
        LightingResponse {
            ip,
            response: LightingResponseType::Payload(payload),
        }
    }

    /// Create a response from a power mode change.
    pub fn power(ip: Ipv4Addr, power: PowerMode) -> Self {
        LightingResponse {
            ip,
            response: LightingResponseType::Power(power),
        }
    }

    /// Create a response from a status query.
    pub fn status(ip: Ipv4Addr, status: LightStatus) -> Self {
        LightingResponse {
            ip,
            response: LightingResponseType::Status(status),
        }
    }
}

/// The type of lighting response.
#[derive(Debug)]
pub(crate) enum LightingResponseType {
    /// Response from a lighting setting change
    Payload(Payload),
    /// Response from a power state change
    Power(PowerMode),
    /// Response from a status query
    Status(LightStatus),
}

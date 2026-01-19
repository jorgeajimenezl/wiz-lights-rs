//! Device discovery via UDP broadcast.

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use serde_json::{Value, json};

use crate::errors::Error;
use crate::light::Light;
use crate::runtime::{self, AsyncUdpSocket, Instant, UdpSocket};

type Result<T> = std::result::Result<T, Error>;

/// A discovered Wiz bulb.
#[derive(Debug, Clone)]
pub struct DiscoveredBulb {
    pub ip: Ipv4Addr,
    pub mac: String,
}

impl DiscoveredBulb {
    pub fn into_light(self, name: Option<&str>) -> Light {
        Light::new(self.ip, name)
    }
}

/// Discovers Wiz bulbs using UDP broadcast.
pub async fn discover_bulbs(discovery_timeout: Duration) -> Result<Vec<DiscoveredBulb>> {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .await
        .map_err(|e| Error::socket("bind", e))?;

    socket
        .set_broadcast(true)
        .map_err(|e| Error::socket("set_broadcast", e))?;

    let msg = json!({
        "method": "registration",
        "params": {
            "phoneMac": "AAAAAAAAAAAA",
            "register": false,
            "phoneIp": "1.2.3.4",
            "id": "1"
        }
    });
    let msg_bytes = serde_json::to_vec(&msg).map_err(Error::JsonDump)?;

    socket
        .send_to(&msg_bytes, "255.255.255.255:38899")
        .await
        .map_err(|e| Error::socket("send_to", e))?;

    let mut discovered: HashMap<String, DiscoveredBulb> = HashMap::new();
    let start = Instant::now();
    let mut buffer = [0u8; 4096];
    let recv_timeout = Duration::from_millis(500);

    while start.elapsed() < discovery_timeout {
        // Use runtime-agnostic timeout for each recv_from operation
        match runtime::timeout(recv_timeout, socket.recv_from(&mut buffer)).await {
            Ok(Ok((size, addr))) => {
                if let Ok(response) = String::from_utf8(buffer[..size].to_vec())
                    && let Ok(json) = serde_json::from_str::<Value>(&response)
                    && let Some(mac) = extract_mac(&json)
                {
                    let ip = match addr {
                        SocketAddr::V4(v4) => *v4.ip(),
                        SocketAddr::V6(_) => continue,
                    };
                    discovered.insert(mac.clone(), DiscoveredBulb { ip, mac });
                }
            }
            // Timeout elapsed - continue loop to check overall timeout
            Ok(Err(_)) | Err(_) => continue,
        }
    }

    Ok(discovered.into_values().collect())
}

fn extract_mac(json: &Value) -> Option<String> {
    json.get("result")
        .and_then(|r| r.get("mac"))
        .and_then(|m| m.as_str())
        .map(String::from)
}

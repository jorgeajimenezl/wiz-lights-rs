//! Push notification support for real-time state updates via syncPilot.

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::discovery::DiscoveredBulb;
use crate::errors::Error;
use crate::runtime::{self, AsyncUdpSocket, Instant, JoinHandle, Mutex, UdpSocket};

type Result<T> = std::result::Result<T, Error>;

pub const LISTEN_PORT: u16 = 38900;
pub const RESPOND_PORT: u16 = 38899;

/// Callback type for state updates (syncPilot messages).
/// Takes the MAC address and the params value from the message.
pub type StateCallback = Arc<dyn Fn(&str, &Value) + Send + Sync + 'static>;

/// Callback type for discovery events (firstBeat messages).
/// Takes the discovered bulb information.
pub type DiscoveryCallback = Arc<dyn Fn(DiscoveredBulb) + Send + Sync + 'static>;

/// Diagnostics for the push manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushDiagnostics {
    pub running: bool,
    pub subscription_count: usize,
    pub time_since_last_push: Option<f64>,
    pub last_error: Option<String>,
}

/// Manages push notification subscriptions for multiple bulbs.
///
/// The `PushManager` listens for push notifications from Wiz bulbs on port 38900.
/// It supports subscribing to state updates for specific bulbs (by MAC address)
/// and can also notify when new bulbs are discovered.
///
/// # Example
///
/// ```ignore
/// use std::net::Ipv4Addr;
/// use wiz_lights_rs::push::PushManager;
///
/// let manager = PushManager::new();
///
/// // Subscribe to updates for a specific bulb
/// manager.subscribe("AABBCCDDEEFF", |mac, params| {
///     println!("Bulb {} updated: {:?}", mac, params);
/// }).await;
///
/// // Start listening (provide your local IP for registration)
/// manager.start(Ipv4Addr::new(192, 168, 1, 100)).await?;
///
/// // ... later ...
/// manager.stop().await;
/// ```
pub struct PushManager {
    running: Arc<AtomicBool>,
    subscriptions: Arc<Mutex<HashMap<String, StateCallback>>>,
    discovery_callback: Arc<Mutex<Option<DiscoveryCallback>>>,
    listener_task: Mutex<Option<JoinHandle<()>>>,
    last_push: Arc<Mutex<Option<Instant>>>,
    last_error: Arc<Mutex<Option<String>>>,
    register_msg: Arc<Mutex<Option<Value>>>,
}

impl Default for PushManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PushManager {
    /// Create a new push manager.
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            discovery_callback: Arc::new(Mutex::new(None)),
            listener_task: Mutex::new(None),
            last_push: Arc::new(Mutex::new(None)),
            last_error: Arc::new(Mutex::new(None)),
            register_msg: Arc::new(Mutex::new(None)),
        }
    }

    /// Check if the push manager is currently running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Get diagnostics information about the push manager.
    pub async fn diagnostics(&self) -> PushDiagnostics {
        PushDiagnostics {
            running: self.is_running(),
            subscription_count: self.subscriptions.lock().await.len(),
            time_since_last_push: self
                .last_push
                .lock()
                .await
                .map(|t| t.elapsed().as_secs_f64()),
            last_error: self.last_error.lock().await.clone(),
        }
    }

    /// Subscribe to state updates for a specific bulb.
    ///
    /// The callback will be invoked whenever a `syncPilot` message is received
    /// from the bulb with the specified MAC address.
    pub async fn subscribe<F: Fn(&str, &Value) + Send + Sync + 'static>(
        &self,
        mac: &str,
        callback: F,
    ) {
        self.subscriptions
            .lock()
            .await
            .insert(mac.to_uppercase(), Arc::new(callback));
    }

    /// Unsubscribe from state updates for a specific bulb.
    pub async fn unsubscribe(&self, mac: &str) {
        self.subscriptions.lock().await.remove(&mac.to_uppercase());
    }

    /// Set a callback for discovery events.
    ///
    /// The callback will be invoked whenever a `firstBeat` message is received,
    /// indicating a new bulb has appeared on the network.
    pub async fn set_discovery_callback<F: Fn(DiscoveredBulb) + Send + Sync + 'static>(
        &self,
        callback: F,
    ) {
        *self.discovery_callback.lock().await = Some(Arc::new(callback));
    }

    /// Start the push listener on port 38900.
    ///
    /// # Arguments
    ///
    /// * `local_ip` - The local IP address to use for registration messages.
    ///   This should be the IP of the interface on the same network as the bulbs.
    pub async fn start(&self, local_ip: Ipv4Addr) -> Result<()> {
        if self.is_running() {
            return Ok(());
        }

        let socket = UdpSocket::bind(&format!("0.0.0.0:{LISTEN_PORT}"))
            .await
            .map_err(|e| Error::socket("bind push socket", e))?;

        *self.register_msg.lock().await = Some(json!({
            "method": "registration",
            "params": {
                "phoneIp": local_ip.to_string(),
                "register": true,
                "phoneMac": generate_mac(),
            }
        }));

        self.running.store(true, Ordering::SeqCst);

        let running = Arc::clone(&self.running);
        let subscriptions = Arc::clone(&self.subscriptions);
        let discovery_callback = Arc::clone(&self.discovery_callback);
        let last_push = Arc::clone(&self.last_push);
        let last_error = Arc::clone(&self.last_error);

        let handle = runtime::spawn(async move {
            let mut buffer = [0u8; 4096];
            let recv_timeout = Duration::from_millis(500);

            while running.load(Ordering::SeqCst) {
                match runtime::timeout(recv_timeout, socket.recv_from(&mut buffer)).await {
                    Ok(Ok((size, addr))) => {
                        *last_push.lock().await = Some(Instant::now());

                        let Ok(msg_str) = String::from_utf8(buffer[..size].to_vec()) else {
                            continue;
                        };
                        if msg_str == "test" {
                            continue;
                        }

                        let Ok(msg) = serde_json::from_str::<Value>(&msg_str) else {
                            continue;
                        };
                        let method = msg.get("method").and_then(|m| m.as_str());
                        let mac = msg
                            .get("params")
                            .and_then(|p| p.get("mac"))
                            .and_then(|m| m.as_str())
                            .map(|s| s.to_uppercase());

                        let SocketAddr::V4(v4) = addr else {
                            continue;
                        };
                        let source_ip = *v4.ip();

                        match (method, &mac) {
                            (Some("syncPilot"), Some(mac_addr)) => {
                                let subs = subscriptions.lock().await;
                                if let Some(cb) = subs.get(mac_addr) {
                                    let cb = Arc::clone(cb);
                                    let mac_addr = mac_addr.clone();
                                    let params = msg.get("params").cloned().unwrap_or(Value::Null);
                                    // Execute callback - we don't spawn here to keep it simple
                                    // and maintain ordering of callbacks
                                    drop(subs); // Release lock before callback
                                    cb(&mac_addr, &params);
                                }
                            }
                            (Some("firstBeat"), Some(mac_addr)) => {
                                let disc_cb = discovery_callback.lock().await;
                                if let Some(ref cb) = *disc_cb {
                                    let cb = Arc::clone(cb);
                                    let bulb = DiscoveredBulb {
                                        ip: source_ip,
                                        mac: mac_addr.clone(),
                                    };
                                    drop(disc_cb); // Release lock before callback
                                    cb(bulb);
                                }
                            }
                            _ => debug!("Unknown push method: {:?}", method),
                        }
                    }
                    // Timeout or error - continue loop
                    Ok(Err(e)) => {
                        *last_error.lock().await = Some(e.to_string());
                        error!("Push socket error: {}", e);
                    }
                    Err(_) => {
                        // Timeout - just continue
                    }
                }
            }
        });

        *self.listener_task.lock().await = Some(handle);
        Ok(())
    }

    /// Stop the push listener.
    pub async fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(h) = self.listener_task.lock().await.take() {
            // Wait for the task to complete (it will exit due to running flag)
            let _ = h.await;
        }
    }

    /// Get the registration message for registering with bulbs.
    pub async fn registration_message(&self) -> Option<Value> {
        self.register_msg.lock().await.clone()
    }

    /// Register with a bulb to receive push notifications.
    ///
    /// This sends a registration message to the bulb at the specified IP address.
    pub async fn register_bulb(&self, bulb_ip: Ipv4Addr) -> Result<()> {
        let reg_msg = self
            .registration_message()
            .await
            .ok_or(Error::NoAttribute)?;

        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| Error::socket("bind", e))?;

        let msg_bytes = serde_json::to_vec(&reg_msg).map_err(Error::JsonDump)?;

        // Use runtime-agnostic timeout for the send operation
        runtime::timeout(
            Duration::from_secs(2),
            socket.send_to(&msg_bytes, &format!("{bulb_ip}:{RESPOND_PORT}")),
        )
        .await
        .map_err(|_| {
            Error::socket(
                "send_to",
                std::io::Error::new(std::io::ErrorKind::TimedOut, "send timeout"),
            )
        })?
        .map_err(|e| Error::socket("send_to", e))?;

        Ok(())
    }
}

impl Drop for PushManager {
    fn drop(&mut self) {
        // Signal the task to stop
        self.running.store(false, Ordering::SeqCst);
        // Note: We can't await the task in drop, so the task will be aborted
        // when the JoinHandle is dropped. This is acceptable because the task
        // checks the running flag frequently and will exit cleanly.
    }
}

fn generate_mac() -> String {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!(
        "{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
        ((seed >> 40) & 0xFF) as u8,
        ((seed >> 32) & 0xFF) as u8,
        ((seed >> 24) & 0xFF) as u8,
        ((seed >> 16) & 0xFF) as u8,
        ((seed >> 8) & 0xFF) as u8,
        (seed & 0xFF) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_manager_new() {
        let manager = PushManager::new();
        assert!(!manager.is_running());
    }

    #[tokio::test]
    async fn test_subscribe_unsubscribe() {
        let manager = PushManager::new();
        manager.subscribe("AABBCCDDEEFF", |_, _| {}).await;
        assert_eq!(manager.subscriptions.lock().await.len(), 1);
        manager.unsubscribe("AABBCCDDEEFF").await;
        assert_eq!(manager.subscriptions.lock().await.len(), 0);
    }

    #[test]
    fn test_generate_mac() {
        let mac = generate_mac();
        assert_eq!(mac.len(), 12);
        assert!(mac.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

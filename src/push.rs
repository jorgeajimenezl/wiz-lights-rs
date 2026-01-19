//! Push notification support for real-time state updates via syncPilot.

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::discovery::DiscoveredBulb;
use crate::errors::Error;

type Result<T> = std::result::Result<T, Error>;

pub const LISTEN_PORT: u16 = 38900;
pub const RESPOND_PORT: u16 = 38899;

pub type StateCallback = Box<dyn Fn(&str, &Value) + Send + 'static>;
pub type DiscoveryCallback = Box<dyn Fn(DiscoveredBulb) + Send + 'static>;

/// Diagnostics for the push manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushDiagnostics {
    pub running: bool,
    pub subscription_count: usize,
    pub time_since_last_push: Option<f64>,
    pub last_error: Option<String>,
}

/// Manages push notification subscriptions for multiple bulbs.
pub struct PushManager {
    running: Arc<AtomicBool>,
    subscriptions: Arc<Mutex<HashMap<String, StateCallback>>>,
    discovery_callback: Arc<Mutex<Option<DiscoveryCallback>>>,
    listener_thread: Mutex<Option<JoinHandle<()>>>,
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
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            discovery_callback: Arc::new(Mutex::new(None)),
            listener_thread: Mutex::new(None),
            last_push: Arc::new(Mutex::new(None)),
            last_error: Arc::new(Mutex::new(None)),
            register_msg: Arc::new(Mutex::new(None)),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn diagnostics(&self) -> PushDiagnostics {
        PushDiagnostics {
            running: self.is_running(),
            subscription_count: self.subscriptions.lock().unwrap().len(),
            time_since_last_push: self.last_push.lock().unwrap().map(|t| t.elapsed().as_secs_f64()),
            last_error: self.last_error.lock().unwrap().clone(),
        }
    }

    pub fn subscribe<F: Fn(&str, &Value) + Send + 'static>(&self, mac: &str, callback: F) {
        self.subscriptions.lock().unwrap().insert(mac.to_uppercase(), Box::new(callback));
    }

    pub fn unsubscribe(&self, mac: &str) {
        self.subscriptions.lock().unwrap().remove(&mac.to_uppercase());
    }

    pub fn set_discovery_callback<F: Fn(DiscoveredBulb) + Send + 'static>(&self, callback: F) {
        *self.discovery_callback.lock().unwrap() = Some(Box::new(callback));
    }

    /// Start the push listener on port 38900.
    pub fn start(&self, local_ip: Ipv4Addr) -> Result<()> {
        if self.is_running() {
            return Ok(());
        }

        let socket = UdpSocket::bind(format!("0.0.0.0:{LISTEN_PORT}"))
            .map_err(|e| Error::socket("bind push socket", e))?;
        socket.set_read_timeout(Some(Duration::from_millis(500)))
            .map_err(|e| Error::socket("set_read_timeout", e))?;

        *self.register_msg.lock().unwrap() = Some(json!({
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

        let handle = thread::spawn(move || {
            let mut buffer = [0u8; 4096];

            while running.load(Ordering::SeqCst) {
                match socket.recv_from(&mut buffer) {
                    Ok((size, addr)) => {
                        *last_push.lock().unwrap() = Some(Instant::now());

                        let Ok(msg_str) = String::from_utf8(buffer[..size].to_vec()) else { continue };
                        if msg_str == "test" { continue; }

                        let Ok(msg) = serde_json::from_str::<Value>(&msg_str) else { continue };
                        let method = msg.get("method").and_then(|m| m.as_str());
                        let mac = msg.get("params")
                            .and_then(|p| p.get("mac"))
                            .and_then(|m| m.as_str())
                            .map(|s| s.to_uppercase());

                        let SocketAddr::V4(v4) = addr else { continue };
                        let source_ip = *v4.ip();

                        match (method, &mac) {
                            (Some("syncPilot"), Some(mac_addr)) => {
                                if let Some(cb) = subscriptions.lock().unwrap().get(mac_addr) {
                                    cb(mac_addr, msg.get("params").unwrap_or(&Value::Null));
                                }
                            }
                            (Some("firstBeat"), Some(mac_addr)) => {
                                if let Some(ref cb) = *discovery_callback.lock().unwrap() {
                                    cb(DiscoveredBulb { ip: source_ip, mac: mac_addr.clone() });
                                }
                            }
                            _ => debug!("Unknown push method: {:?}", method),
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                    Err(e) => {
                        *last_error.lock().unwrap() = Some(e.to_string());
                        error!("Push socket error: {}", e);
                    }
                }
            }
        });

        *self.listener_thread.lock().unwrap() = Some(handle);
        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(h) = self.listener_thread.lock().unwrap().take() {
            let _ = h.join();
        }
    }

    pub fn registration_message(&self) -> Option<Value> {
        self.register_msg.lock().unwrap().clone()
    }

    pub fn register_bulb(&self, bulb_ip: Ipv4Addr) -> Result<()> {
        let reg_msg = self.registration_message().ok_or(Error::NoAttribute)?;
        let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| Error::socket("bind", e))?;
        socket.set_write_timeout(Some(Duration::from_secs(2)))
            .map_err(|e| Error::socket("set_write_timeout", e))?;
        let msg_bytes = serde_json::to_vec(&reg_msg).map_err(Error::JsonDump)?;
        socket.send_to(&msg_bytes, format!("{bulb_ip}:{RESPOND_PORT}"))
            .map_err(|e| Error::socket("send_to", e))?;
        Ok(())
    }
}

impl Drop for PushManager {
    fn drop(&mut self) {
        self.stop();
    }
}

fn generate_mac() -> String {
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
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

    #[test]
    fn test_subscribe_unsubscribe() {
        let manager = PushManager::new();
        manager.subscribe("AABBCCDDEEFF", |_, _| {});
        assert_eq!(manager.subscriptions.lock().unwrap().len(), 1);
        manager.unsubscribe("AABBCCDDEEFF");
        assert_eq!(manager.subscriptions.lock().unwrap().len(), 0);
    }

    #[test]
    fn test_generate_mac() {
        let mac = generate_mac();
        assert_eq!(mac.len(), 12);
        assert!(mac.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

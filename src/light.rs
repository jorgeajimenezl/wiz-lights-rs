//! Individual light control.

use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;

use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::runtime::{self, AsyncUdpSocket, Mutex, UdpSocket};

use crate::config::{BulbType, ExtendedWhiteRange, SystemConfig, SystemConfigResponse, WhiteRange};
use crate::errors::Error;
use crate::history::{MessageHistory, MessageType};
use crate::payload::Payload;
use crate::response::{LightingResponse, LightingResponseType};
use crate::status::{BulbStatus, LightStatus};
use crate::types::{FanDirection, FanMode, FanSpeed, FanState, PowerMode};

type Result<T> = std::result::Result<T, Error>;

/// Represents a single Wiz smart light bulb.
///
/// A `Light` communicates with a physical Wiz bulb over UDP. Each light is
/// identified by its IPv4 address and can optionally have a user-friendly name.
///
/// # Example
///
/// ```
/// use std::net::Ipv4Addr;
/// use std::str::FromStr;
/// use wiz_lights_rs::Light;
///
/// let light = Light::new(Ipv4Addr::from_str("192.168.1.100").unwrap(), Some("Bedroom"));
/// assert!(light.status().is_none());
/// ```
#[serde_with::skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct Light {
    ip: Ipv4Addr,
    name: Option<String>,
    status: Option<LightStatus>,
    #[serde(skip)]
    history: Arc<Mutex<MessageHistory>>,
}

impl Clone for Light {
    fn clone(&self) -> Self {
        // For cloning, we create a new history mutex with a clone of the history data.
        // This requires blocking to read the current history, which is acceptable for clone.
        // Note: try_lock API differs between runtimes:
        // - tokio returns Result<Guard, TryLockError>
        // - async-std and async-lock (smol) return Option<Guard>
        #[cfg(feature = "runtime-tokio")]
        let history_clone = match self.history.try_lock() {
            Ok(guard) => guard.clone(),
            Err(_) => MessageHistory::new(), // If locked, start fresh
        };
        #[cfg(any(feature = "runtime-async-std", feature = "runtime-smol"))]
        let history_clone = match self.history.try_lock() {
            Some(guard) => guard.clone(),
            None => MessageHistory::new(), // If locked, start fresh
        };
        Light {
            ip: self.ip,
            name: self.name.clone(),
            status: self.status.clone(),
            history: Arc::new(Mutex::new(history_clone)),
        }
    }
}

impl Light {
    const PORT: u16 = 38899;
    const TIMEOUT_MS: u64 = 1000;
    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAYS_MS: [u64; 3] = [750, 1500, 3000];

    pub fn new(ip: Ipv4Addr, name: Option<&str>) -> Self {
        Light {
            ip,
            name: name.map(String::from),
            status: None,
            history: Arc::new(Mutex::new(MessageHistory::new())),
        }
    }

    pub fn ip(&self) -> Ipv4Addr {
        self.ip
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn status(&self) -> Option<&LightStatus> {
        self.status.as_ref()
    }

    pub async fn history(&self) -> MessageHistory {
        self.history.lock().await.clone()
    }

    pub async fn clear_history(&self) {
        self.history.lock().await.clear();
    }

    /// Returns diagnostics including state, configuration, and history.
    pub async fn diagnostics(&self) -> Value {
        let mut diag = json!({
            "ip": self.ip.to_string(),
            "name": self.name,
            "status": self.status.as_ref().map(|s| json!({
                "emitting": s.emitting(),
                "color": s.color().map(|c| format!("{},{},{}", c.red(), c.green(), c.blue())),
                "brightness": s.brightness().map(|b| b.value()),
                "temp": s.temp().map(|t| t.kelvin()),
                "scene": s.scene().map(|sc| format!("{:?}", sc)),
            })),
        });

        // Add history summary
        let history = self.history.lock().await;
        diag["history"] = serde_json::to_value(history.summary()).unwrap_or(Value::Null);
        drop(history); // Release lock before network operations

        // Try to add configuration info (may fail if device is unreachable)
        if let Ok(config) = self.get_system_config().await {
            diag["system_config"] = json!({
                "mac": config.mac,
                "module_name": config.module_name,
                "fw_version": config.fw_version,
                "home_id": config.home_id,
                "room_id": config.room_id,
            });
        }

        if let Ok(Some(white_range)) = self.get_white_range().await {
            diag["white_range"] = json!(white_range.values);
        }

        if let Ok(Some(ext_range)) = self.get_extended_white_range().await {
            diag["extended_white_range"] = json!(ext_range.values);
        }

        if let Ok(Some(fan_range)) = self.get_fan_speed_range().await {
            diag["fan_speed_range"] = json!(fan_range);
        }

        if let Ok(bulb_type) = self.get_bulb_type().await {
            diag["bulb_type"] = json!({
                "name": bulb_type.name,
                "class": format!("{:?}", bulb_type.bulb_class),
                "kelvin_range": {
                    "min": bulb_type.kelvin_range.min,
                    "max": bulb_type.kelvin_range.max,
                },
                "features": {
                    "color": bulb_type.features.color,
                    "color_tmp": bulb_type.features.color_tmp,
                    "effect": bulb_type.features.effect,
                    "brightness": bulb_type.features.brightness,
                    "fan": bulb_type.features.fan,
                },
                "fw_version": bulb_type.fw_version,
            });
        }

        diag
    }

    /// Queries the bulb for current status (live network call).
    pub async fn get_status(&self) -> Result<LightStatus> {
        let resp = self.send_command(&json!({"method": "getPilot"})).await?;
        let status: BulbStatus = serde_json::from_value(resp).map_err(Error::JsonLoad)?;
        Ok(LightStatus::from(&status))
    }

    /// Applies lighting settings from a payload.
    pub async fn set(&self, payload: &Payload) -> Result<LightingResponse> {
        if !payload.is_valid() {
            return Err(Error::NoAttribute);
        }

        let msg = serde_json::to_value(payload).map_err(Error::JsonDump)?;
        let response = self
            .send_command(&json!({
                "method": "setPilot",
                "params": msg,
            }))
            .await?;

        debug!("UDP response: {:?}", response);
        Ok(LightingResponse::payload(self.ip, payload.clone()))
    }

    pub async fn set_power(&self, power: &PowerMode) -> Result<LightingResponse> {
        match power {
            PowerMode::On => self.set_power_state(true).await,
            PowerMode::Off => self.set_power_state(false).await,
            PowerMode::Reboot => self.reboot_bulb().await,
        }
    }

    pub async fn toggle(&self) -> Result<LightingResponse> {
        let status = self.get_status().await?;
        if status.emitting() {
            self.set_power(&PowerMode::Off).await
        } else {
            self.set_power(&PowerMode::On).await
        }
    }

    /// Factory resets the bulb (including WiFi configuration).
    pub async fn reset(&self) -> Result<()> {
        self.send_command(&json!({"method": "reset"})).await?;
        Ok(())
    }

    /// Returns power consumption in watts (if supported).
    pub async fn get_power(&self) -> Result<Option<f32>> {
        let resp = self.send_command(&json!({"method": "getPower"})).await?;
        Ok(resp
            .get("result")
            .and_then(|r| r.get("power"))
            .and_then(|p| p.as_f64())
            .map(|p| p as f32))
    }

    pub async fn get_system_config(&self) -> Result<SystemConfig> {
        let resp = self
            .send_command(&json!({"method": "getSystemConfig"}))
            .await?;
        let config: SystemConfigResponse = serde_json::from_value(resp).map_err(Error::JsonLoad)?;
        Ok(config.result)
    }

    pub async fn get_user_config(&self) -> Result<Value> {
        let resp = self
            .send_command(&json!({"method": "getUserConfig"}))
            .await?;
        Ok(resp.get("result").cloned().unwrap_or(Value::Null))
    }

    /// Returns model configuration (firmware >= 1.22).
    pub async fn get_model_config(&self) -> Result<Value> {
        let resp = self
            .send_command(&json!({"method": "getModelConfig"}))
            .await?;
        Ok(resp.get("result").cloned().unwrap_or(Value::Null))
    }

    pub async fn get_bulb_type(&self) -> Result<BulbType> {
        let config = self.get_system_config().await?;
        let module_name = config.module_name.as_deref().unwrap_or("Unknown");
        let fw_version = config.fw_version.as_deref();
        Ok(BulbType::from_module_name(module_name, fw_version))
    }

    pub async fn get_white_range(&self) -> Result<Option<WhiteRange>> {
        let config = self.get_user_config().await?;
        Ok(parse_f32_array(&config, "whiteRange").map(WhiteRange::new))
    }

    pub async fn get_extended_white_range(&self) -> Result<Option<ExtendedWhiteRange>> {
        // Try model config first (FW >= 1.22), then user config
        let model = self.get_model_config().await?;
        let user = self.get_user_config().await?;

        for (config, key) in [
            (&model, "cctRange"),
            (&user, "extRange"),
            (&user, "cctRange"),
        ] {
            if let Some(values) = parse_f32_array(config, key) {
                return Ok(Some(ExtendedWhiteRange::new(values)));
            }
        }
        Ok(None)
    }

    pub async fn get_fan_speed_range(&self) -> Result<Option<u8>> {
        let model = self.get_model_config().await?;
        if let Some(v) = model.get("fanSpeed").and_then(|v| v.as_u64()) {
            return Ok(Some(v as u8));
        }
        let user = self.get_user_config().await?;
        Ok(user
            .get("fanSpeed")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8))
    }

    pub async fn fan_set_state(
        &self,
        state: Option<FanState>,
        mode: Option<FanMode>,
        speed: Option<FanSpeed>,
        direction: Option<FanDirection>,
    ) -> Result<LightingResponse> {
        let mut payload = Payload::new();

        if let Some(s) = state {
            payload.fan_state(&s);
        }
        if let Some(m) = mode {
            payload.fan_mode(&m);
        }
        if let Some(sp) = speed {
            payload.fan_speed(&sp);
        }
        if let Some(d) = direction {
            payload.fan_direction(&d);
        }

        let msg = serde_json::to_value(&payload).map_err(Error::JsonDump)?;
        self.send_command(&json!({
            "method": "setPilot",
            "params": msg,
        }))
        .await?;

        Ok(LightingResponse::payload(self.ip, payload))
    }

    pub async fn fan_turn_on(
        &self,
        mode: Option<FanMode>,
        speed: Option<FanSpeed>,
    ) -> Result<LightingResponse> {
        self.fan_set_state(Some(FanState::On), mode, speed, None)
            .await
    }

    pub async fn fan_turn_off(&self) -> Result<LightingResponse> {
        self.fan_set_state(Some(FanState::Off), None, None, None)
            .await
    }

    pub async fn fan_toggle(&self) -> Result<LightingResponse> {
        // Check fan state from the raw response
        let resp = self.send_command(&json!({"method": "getPilot"})).await?;
        let fan_on = resp
            .get("result")
            .and_then(|r| r.get("fanState"))
            .and_then(|s| s.as_u64())
            .map(|s| s == 1)
            .unwrap_or(false);

        if fan_on {
            self.fan_turn_off().await
        } else {
            self.fan_turn_on(None, None).await
        }
    }

    pub async fn set_fan_speed(&self, speed: FanSpeed) -> Result<LightingResponse> {
        self.fan_set_state(None, None, Some(speed), None).await
    }

    pub async fn set_fan_mode(&self, mode: FanMode) -> Result<LightingResponse> {
        self.fan_set_state(None, Some(mode), None, None).await
    }

    pub async fn set_fan_direction(&self, direction: FanDirection) -> Result<LightingResponse> {
        self.fan_set_state(None, None, None, Some(direction)).await
    }

    pub fn process_reply(&mut self, resp: &LightingResponse) -> bool {
        if resp.ip != self.ip {
            return false;
        }

        match &resp.response {
            LightingResponseType::Payload(payload) => self.update_status_from_payload(payload),
            LightingResponseType::Power(power) => self.update_status_from_power(power),
            LightingResponseType::Status(status) => self.update_status(status),
        }
        true
    }

    pub(crate) fn update(&mut self, other: &Self) -> bool {
        let mut changed = false;
        if self.name != other.name {
            self.name.clone_from(&other.name);
            changed = true;
        }
        if self.ip != other.ip {
            self.ip = other.ip;
            changed = true;
        }
        changed
    }

    async fn set_power_state(&self, on: bool) -> Result<LightingResponse> {
        self.send_command(&json!({"method": "setState", "params": {"state": on}}))
            .await?;
        let power = if on { PowerMode::On } else { PowerMode::Off };
        Ok(LightingResponse::power(self.ip, power))
    }

    async fn reboot_bulb(&self) -> Result<LightingResponse> {
        self.send_command(&json!({"method": "reboot"})).await?;
        Ok(LightingResponse::power(self.ip, PowerMode::Reboot))
    }

    fn update_status(&mut self, status: &LightStatus) {
        if let Some(current) = &mut self.status {
            current.update(status);
        } else {
            self.status = Some(status.clone());
        }
    }

    fn update_status_from_payload(&mut self, payload: &Payload) {
        if let Some(status) = &mut self.status {
            status.update_from_payload(payload);
        } else {
            self.status = Some(LightStatus::from(payload));
        }
    }

    fn update_status_from_power(&mut self, power: &PowerMode) {
        if let Some(status) = &mut self.status {
            status.update_from_power(power);
        } else {
            self.status = Some(LightStatus::from(power));
        }
    }

    async fn send_command(&self, msg: &Value) -> Result<Value> {
        // Record the sent message
        self.history.lock().await.record(MessageType::Send, msg);

        let msg_str = serde_json::to_string(msg).map_err(Error::JsonDump)?;
        let mut last_error = None;

        for attempt in 0..=Self::MAX_RETRIES {
            match self.send_udp(&msg_str).await {
                Ok(response) => {
                    // Record the received response
                    self.history
                        .lock()
                        .await
                        .record(MessageType::Receive, &response);
                    return Ok(response);
                }
                Err(e) => {
                    // Record the error
                    self.history.lock().await.record_error(&e.to_string());
                    last_error = Some(e);
                    if attempt < Self::MAX_RETRIES {
                        let delay_idx = (attempt as usize).min(Self::RETRY_DELAYS_MS.len() - 1);
                        runtime::sleep(Duration::from_millis(Self::RETRY_DELAYS_MS[delay_idx]))
                            .await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or(Error::NoAttribute))
    }

    async fn send_udp(&self, msg: &str) -> Result<Value> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .await
            .map_err(|e| Error::socket("bind", e))?;

        socket
            .connect(&format!("{}:{}", self.ip, Self::PORT))
            .await
            .map_err(|e| Error::socket("connect", e))?;

        socket
            .send(msg.as_bytes())
            .await
            .map_err(|e| Error::socket("send", e))?;

        let mut buffer = [0u8; 4096];

        // Use runtime-agnostic timeout for the receive operation
        let bytes = runtime::timeout(
            Duration::from_millis(Self::TIMEOUT_MS),
            socket.recv(&mut buffer),
        )
        .await
        .map_err(|_| {
            Error::socket(
                "receive",
                std::io::Error::new(std::io::ErrorKind::TimedOut, "receive timeout"),
            )
        })?
        .map_err(|e| Error::socket("receive", e))?;

        let response = String::from_utf8(buffer[..bytes].to_vec()).map_err(Error::Utf8Decode)?;
        serde_json::from_str(&response).map_err(Error::JsonLoad)
    }
}

fn parse_f32_array(config: &Value, key: &str) -> Option<Vec<f32>> {
    config.get(key).and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_f64().map(|f| f as f32))
            .collect()
    })
}

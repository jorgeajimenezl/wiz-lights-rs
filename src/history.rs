//! Message history tracking for debugging and diagnostics.

use std::collections::HashMap;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Type of message in the history.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    Send,
    Receive,
    Push,
}

/// A recorded message in the history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub msg_type: MessageType,
    pub method: String,
    pub message: Value,
    /// Seconds since history creation
    pub timestamp: f64,
}

/// Tracks message history for debugging.
#[derive(Debug, Clone)]
pub struct MessageHistory {
    history: HashMap<MessageType, HashMap<String, Value>>,
    last_error: Option<String>,
    start_time: Instant,
    entries: Vec<HistoryEntry>,
    max_entries: usize,
}

impl Default for MessageHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageHistory {
    pub const DEFAULT_MAX_ENTRIES: usize = 100;

    pub fn new() -> Self {
        Self {
            history: HashMap::from([
                (MessageType::Send, HashMap::new()),
                (MessageType::Receive, HashMap::new()),
                (MessageType::Push, HashMap::new()),
            ]),
            last_error: None,
            start_time: Instant::now(),
            entries: Vec::new(),
            max_entries: Self::DEFAULT_MAX_ENTRIES,
        }
    }

    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            max_entries,
            ..Self::new()
        }
    }

    pub fn record(&mut self, msg_type: MessageType, message: &Value) {
        let Some(method) = message.get("method").and_then(|m| m.as_str()) else {
            return;
        };

        if let Some(type_map) = self.history.get_mut(&msg_type) {
            type_map.insert(method.to_string(), message.clone());
        }

        self.entries.push(HistoryEntry {
            msg_type,
            method: method.to_string(),
            message: message.clone(),
            timestamp: self.start_time.elapsed().as_secs_f64(),
        });

        if self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
    }

    pub fn record_error(&mut self, error: &str) {
        self.last_error = Some(error.to_string());
    }

    pub fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.history.values_mut().for_each(|m| m.clear());
        self.entries.clear();
        self.last_error = None;
    }

    pub fn summary(&self) -> HistorySummary {
        let count = |t: MessageType| self.history.get(&t).map_or(0, |m| m.len());
        HistorySummary {
            send_count: count(MessageType::Send),
            receive_count: count(MessageType::Receive),
            push_count: count(MessageType::Push),
            total_entries: self.entries.len(),
            last_error: self.last_error.clone(),
        }
    }
}

/// Summary of message history for diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySummary {
    pub send_count: usize,
    pub receive_count: usize,
    pub push_count: usize,
    pub total_entries: usize,
    pub last_error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_record_message() {
        let mut history = MessageHistory::new();
        history.record(
            MessageType::Send,
            &json!({"method": "setPilot", "params": {"state": true}}),
        );

        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_record_error() {
        let mut history = MessageHistory::new();
        history.record_error("Connection timeout");
        assert_eq!(history.last_error(), Some("Connection timeout"));
    }

    #[test]
    fn test_max_entries() {
        let mut history = MessageHistory::with_max_entries(2);
        for i in 0..5 {
            history.record(
                MessageType::Send,
                &json!({"method": format!("method{}", i)}),
            );
        }
        assert_eq!(history.len(), 2);
    }
}

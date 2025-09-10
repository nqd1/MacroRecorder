use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    KeyDown,
    KeyUp,
    MouseMove,
    MouseDown,
    MouseUp,
    MouseScroll,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EventType::KeyDown => write!(f, "KDOWN"),
            EventType::KeyUp => write!(f, "KUP"),
            EventType::MouseMove => write!(f, "MMOVE"),
            EventType::MouseDown => write!(f, "MDOWN"),
            EventType::MouseUp => write!(f, "MUP"),
            EventType::MouseScroll => write!(f, "MSCROLL"),
        }
    }
}

impl EventType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "KDOWN" => Some(EventType::KeyDown),
            "KUP" => Some(EventType::KeyUp),
            "MMOVE" => Some(EventType::MouseMove),
            "MDOWN" => Some(EventType::MouseDown),
            "MUP" => Some(EventType::MouseUp),
            "MSCROLL" => Some(EventType::MouseScroll),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroEvent {
    pub timestamp: f64,
    pub event_type: EventType,
    pub data: serde_json::Value,
}

impl MacroEvent {
    pub fn new(timestamp: f64, event_type: EventType) -> Self {
        Self {
            timestamp,
            event_type,
            data: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
    
    pub fn with_data(timestamp: f64, event_type: EventType, data: serde_json::Value) -> Self {
        Self {
            timestamp,
            event_type,
            data,
        }
    }
    
    // Convert to .mcr file format (compatible with Python version)
    pub fn to_mcr_line(&self) -> String {
        let mut parts = vec![
            format!("{:.6}", self.timestamp),
            self.event_type.to_string(),
        ];
        
        // Add data fields based on event type
        match self.event_type {
            EventType::KeyDown | EventType::KeyUp => {
                if let Some(key_name) = self.data.get("key_name") {
                    if let Some(key_str) = key_name.as_str() {
                        parts.push(format!("char={}", key_str));
                    }
                }
            }
            EventType::MouseMove => {
                if let (Some(x), Some(y)) = (self.data.get("x"), self.data.get("y")) {
                    if let (Some(x_val), Some(y_val)) = (x.as_i64(), y.as_i64()) {
                        parts.push(format!("x={}", x_val));
                        parts.push(format!("y={}", y_val));
                    }
                }
            }
            EventType::MouseDown | EventType::MouseUp => {
                if let (Some(x), Some(y), Some(button)) = (
                    self.data.get("x"),
                    self.data.get("y"),
                    self.data.get("button")
                ) {
                    if let (Some(x_val), Some(y_val), Some(btn_val)) = (
                        x.as_i64(),
                        y.as_i64(),
                        button.as_u64()
                    ) {
                        let button_name = match btn_val {
                            1 => "left",
                            2 => "right",
                            3 => "middle",
                            _ => "unknown",
                        };
                        parts.push(format!("button={}", button_name));
                        parts.push(format!("x={}", x_val));
                        parts.push(format!("y={}", y_val));
                    }
                }
            }
            EventType::MouseScroll => {
                if let (Some(x), Some(y), Some(delta)) = (
                    self.data.get("x"),
                    self.data.get("y"),
                    self.data.get("delta")
                ) {
                    if let (Some(x_val), Some(y_val), Some(delta_val)) = (
                        x.as_i64(),
                        y.as_i64(),
                        delta.as_i64()
                    ) {
                        let dy = if delta_val > 0 { 1 } else { -1 };
                        parts.push(format!("dx=0"));
                        parts.push(format!("dy={}", dy));
                        parts.push(format!("x={}", x_val));
                        parts.push(format!("y={}", y_val));
                    }
                }
            }
        }
        
        parts.join(";")
    }
    
    // Parse from .mcr file format
    pub fn from_mcr_line(line: &str) -> Option<Self> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }
        
        let parts: Vec<&str> = line.split(';').collect();
        if parts.len() < 2 {
            return None;
        }
        
        let timestamp = parts[0].parse::<f64>().ok()?;
        let event_type = EventType::from_str(parts[1])?;
        
        let mut data = serde_json::Map::new();
        
        // Parse remaining parts as key=value pairs
        for part in parts.iter().skip(2) {
            if let Some((key, value)) = part.split_once('=') {
                match key {
                    "char" => {
                        data.insert("key_name".to_string(), serde_json::Value::String(value.to_string()));
                    }
                    "x" | "y" | "dx" | "dy" => {
                        if let Ok(num) = value.parse::<i64>() {
                            data.insert(key.to_string(), serde_json::Value::Number(serde_json::Number::from(num)));
                        }
                    }
                    "button" => {
                        let button_num = match value {
                            "left" => 1,
                            "right" => 2,
                            "middle" => 3,
                            _ => 0,
                        };
                        data.insert(key.to_string(), serde_json::Value::Number(serde_json::Number::from(button_num)));
                    }
                    _ => {
                        data.insert(key.to_string(), serde_json::Value::String(value.to_string()));
                    }
                }
            }
        }
        
        Some(MacroEvent {
            timestamp,
            event_type,
            data: serde_json::Value::Object(data),
        })
    }
    
    pub fn is_mouse_move(&self) -> bool {
        matches!(self.event_type, EventType::MouseMove)
    }
    
    pub fn get_description(&self) -> String {
        match self.event_type {
            EventType::KeyDown => {
                if let Some(key) = self.data.get("key_name") {
                    format!("Key Down: {}", key.as_str().unwrap_or("?"))
                } else {
                    "Key Down".to_string()
                }
            }
            EventType::KeyUp => {
                if let Some(key) = self.data.get("key_name") {
                    format!("Key Up: {}", key.as_str().unwrap_or("?"))
                } else {
                    "Key Up".to_string()
                }
            }
            EventType::MouseMove => {
                if let (Some(x), Some(y)) = (self.data.get("x"), self.data.get("y")) {
                    format!("Mouse Move: ({}, {})", 
                        x.as_i64().unwrap_or(0), 
                        y.as_i64().unwrap_or(0))
                } else {
                    "Mouse Move".to_string()
                }
            }
            EventType::MouseDown => {
                let button = self.data.get("button")
                    .and_then(|b| b.as_u64())
                    .map(|b| match b {
                        1 => "Left",
                        2 => "Right", 
                        3 => "Middle",
                        _ => "Unknown",
                    })
                    .unwrap_or("Unknown");
                    
                if let (Some(x), Some(y)) = (self.data.get("x"), self.data.get("y")) {
                    format!("{} Click Down: ({}, {})", 
                        button,
                        x.as_i64().unwrap_or(0), 
                        y.as_i64().unwrap_or(0))
                } else {
                    format!("{} Click Down", button)
                }
            }
            EventType::MouseUp => {
                let button = self.data.get("button")
                    .and_then(|b| b.as_u64())
                    .map(|b| match b {
                        1 => "Left",
                        2 => "Right",
                        3 => "Middle", 
                        _ => "Unknown",
                    })
                    .unwrap_or("Unknown");
                    
                if let (Some(x), Some(y)) = (self.data.get("x"), self.data.get("y")) {
                    format!("{} Click Up: ({}, {})", 
                        button,
                        x.as_i64().unwrap_or(0), 
                        y.as_i64().unwrap_or(0))
                } else {
                    format!("{} Click Up", button)
                }
            }
            EventType::MouseScroll => {
                let delta = self.data.get("delta")
                    .and_then(|d| d.as_i64())
                    .unwrap_or(0);
                let direction = if delta > 0 { "Up" } else { "Down" };
                
                if let (Some(x), Some(y)) = (self.data.get("x"), self.data.get("y")) {
                    format!("Scroll {}: ({}, {})", 
                        direction,
                        x.as_i64().unwrap_or(0), 
                        y.as_i64().unwrap_or(0))
                } else {
                    format!("Scroll {}", direction)
                }
            }
        }
    }
}

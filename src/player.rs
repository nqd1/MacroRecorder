use crate::events::{MacroEvent, EventType};
use std::time::{Duration, Instant};
use std::thread;
use windows::{
    core::*,
    Win32::{
        // Foundation::*,
        UI::Input::KeyboardAndMouse::*,
        UI::WindowsAndMessaging::*,
    },
};

#[derive(Debug, Clone)]
pub enum PlayerState {
    Idle,
    Playing,
    Paused,
    Stopped,
}

pub struct MacroPlayer {
    events: Vec<MacroEvent>,
    state: PlayerState,
    current_position: usize,
    start_time: Option<Instant>,
    pause_start: Option<Instant>,
    total_pause_time: Duration,
    playback_speed: f32,
}

impl MacroPlayer {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            state: PlayerState::Idle,
            current_position: 0,
            start_time: None,
            pause_start: None,
            total_pause_time: Duration::ZERO,
            playback_speed: 1.0,
        }
    }
    
    pub fn load_from_file(&mut self, path: &str) -> std::result::Result<usize, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        self.events.clear();
        
        for line in content.lines() {
            if let Some(event) = MacroEvent::from_mcr_line(line) {
                self.events.push(event);
            }
        }
        
        // Sort events by timestamp
        self.events.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
        
        self.current_position = 0;
        self.state = PlayerState::Idle;
        
        log::info!("Loaded {} events from {}", self.events.len(), path);
        Ok(self.events.len())
    }
    
    pub fn set_speed(&mut self, speed: f32) {
        self.playback_speed = speed.max(0.1).min(10.0);
    }
    
    pub fn start(&mut self) {
        if !self.events.is_empty() {
            self.state = PlayerState::Playing;
            self.current_position = 0;
            self.start_time = Some(Instant::now());
            self.pause_start = None;
            self.total_pause_time = Duration::ZERO;
            
            // Start playback in a separate thread
            let events = self.events.clone();
            let speed = self.playback_speed;
            
            thread::spawn(move || {
                Self::play_events(events, speed);
            });
            
            log::info!("Playback started with {} events at {}x speed", self.events.len(), self.playback_speed);
        }
    }
    
    pub fn pause(&mut self) {
        if matches!(self.state, PlayerState::Playing) {
            self.state = PlayerState::Paused;
            self.pause_start = Some(Instant::now());
            log::info!("Playback paused");
        }
    }
    
    pub fn resume(&mut self) {
        if matches!(self.state, PlayerState::Paused) {
            self.state = PlayerState::Playing;
            if let Some(pause_start) = self.pause_start.take() {
                self.total_pause_time += pause_start.elapsed();
            }
            log::info!("Playback resumed");
        }
    }
    
    pub fn stop(&mut self) {
        self.state = PlayerState::Stopped;
        if let Some(pause_start) = self.pause_start.take() {
            self.total_pause_time += pause_start.elapsed();
        }
        log::info!("Playback stopped");
    }
    
    pub fn get_state(&self) -> PlayerState {
        self.state.clone()
    }
    
    pub fn get_current_position(&self) -> usize {
        self.current_position
    }
    
    pub fn get_total_events(&self) -> usize {
        self.events.len()
    }
    
    fn play_events(events: Vec<MacroEvent>, speed: f32) {
        if events.is_empty() {
            return;
        }
        
        let start_time = Instant::now();
        let mut last_event_time = 0.0;
        
        for (index, event) in events.iter().enumerate() {
            // Calculate when this event should be played
            let target_time = Duration::from_secs_f64(event.timestamp / speed as f64);
            let current_time = start_time.elapsed();
            
            // Wait until it's time to play this event
            if target_time > current_time {
                let wait_time = target_time - current_time;
                thread::sleep(wait_time);
            }
            
            // Execute the event
            Self::execute_event(event);
            
            last_event_time = event.timestamp;
        }
        
        log::info!("Playback completed");
    }
    
    fn execute_event(event: &MacroEvent) {
        unsafe {
            match event.event_type {
                EventType::KeyDown => {
                    if let Some(key_name) = event.data.get("key_name") {
                        if let Some(key_str) = key_name.as_str() {
                            if let Some(vk_code) = Self::key_name_to_vk_code(key_str) {
                                Self::send_key_input(vk_code, true);
                            }
                        }
                    }
                }
                EventType::KeyUp => {
                    if let Some(key_name) = event.data.get("key_name") {
                        if let Some(key_str) = key_name.as_str() {
                            if let Some(vk_code) = Self::key_name_to_vk_code(key_str) {
                                Self::send_key_input(vk_code, false);
                            }
                        }
                    }
                }
                EventType::MouseMove => {
                    if let (Some(x), Some(y)) = (event.data.get("x"), event.data.get("y")) {
                        if let (Some(x_val), Some(y_val)) = (x.as_i64(), y.as_i64()) {
                            Self::send_mouse_move(x_val as i32, y_val as i32);
                        }
                    }
                }
                EventType::MouseDown => {
                    if let (Some(x), Some(y), Some(button)) = (
                        event.data.get("x"),
                        event.data.get("y"),
                        event.data.get("button")
                    ) {
                        if let (Some(x_val), Some(y_val), Some(btn_val)) = (
                            x.as_i64(),
                            y.as_i64(),
                            button.as_u64()
                        ) {
                            Self::send_mouse_click(x_val as i32, y_val as i32, btn_val as u32, true);
                        }
                    }
                }
                EventType::MouseUp => {
                    if let (Some(x), Some(y), Some(button)) = (
                        event.data.get("x"),
                        event.data.get("y"),
                        event.data.get("button")
                    ) {
                        if let (Some(x_val), Some(y_val), Some(btn_val)) = (
                            x.as_i64(),
                            y.as_i64(),
                            button.as_u64()
                        ) {
                            Self::send_mouse_click(x_val as i32, y_val as i32, btn_val as u32, false);
                        }
                    }
                }
                EventType::MouseScroll => {
                    if let (Some(x), Some(y), Some(delta)) = (
                        event.data.get("x"),
                        event.data.get("y"),
                        event.data.get("delta")
                    ) {
                        if let (Some(x_val), Some(y_val), Some(delta_val)) = (
                            x.as_i64(),
                            y.as_i64(),
                            delta.as_i64()
                        ) {
                            Self::send_mouse_scroll(x_val as i32, y_val as i32, delta_val as i32);
                        }
                    }
                }
            }
        }
    }
    
    unsafe fn send_key_input(vk_code: u16, is_down: bool) {
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: VIRTUAL_KEY(vk_code),
                    wScan: 0,
                    dwFlags: if is_down { KEYBD_EVENT_FLAGS(0) } else { KEYEVENTF_KEYUP },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
    
    unsafe fn send_mouse_move(x: i32, y: i32) {
        SetCursorPos(x, y);
    }
    
    unsafe fn send_mouse_click(x: i32, y: i32, button: u32, is_down: bool) {
        SetCursorPos(x, y);
        
        let flags = match (button, is_down) {
            (1, true) => MOUSEEVENTF_LEFTDOWN,
            (1, false) => MOUSEEVENTF_LEFTUP,
            (2, true) => MOUSEEVENTF_RIGHTDOWN,
            (2, false) => MOUSEEVENTF_RIGHTUP,
            (3, true) => MOUSEEVENTF_MIDDLEDOWN,
            (3, false) => MOUSEEVENTF_MIDDLEUP,
            _ => return,
        };
        
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
    
    unsafe fn send_mouse_scroll(x: i32, y: i32, delta: i32) {
        SetCursorPos(x, y);
        
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: (delta * 120) as i32 as u32, // WHEEL_DELTA is typically 120
                    dwFlags: MOUSEEVENTF_WHEEL,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
    
    fn key_name_to_vk_code(key_name: &str) -> Option<u16> {
        match key_name {
            // Letters
            key if key.len() == 1 && key.chars().next().unwrap().is_ascii_lowercase() => {
                Some(key.to_uppercase().chars().next().unwrap() as u16)
            }
            // Numbers
            key if key.len() == 1 && key.chars().next().unwrap().is_ascii_digit() => {
                Some(key.chars().next().unwrap() as u16)
            }
            // Function keys
            key if key.starts_with('f') && key.len() <= 3 => {
                if let Ok(num) = key[1..].parse::<u16>() {
                    if num >= 1 && num <= 12 {
                        Some(0x70 + num - 1) // F1 is 0x70
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            // Special keys
            "space" => Some(VK_SPACE.0),
            "enter" => Some(VK_RETURN.0),
            "backspace" => Some(VK_BACK.0),
            "tab" => Some(VK_TAB.0),
            "shift" => Some(VK_SHIFT.0),
            "ctrl" => Some(VK_CONTROL.0),
            "alt" => Some(VK_MENU.0),
            "esc" => Some(VK_ESCAPE.0),
            "left" => Some(VK_LEFT.0),
            "up" => Some(VK_UP.0),
            "right" => Some(VK_RIGHT.0),
            "down" => Some(VK_DOWN.0),
            "delete" => Some(VK_DELETE.0),
            "insert" => Some(VK_INSERT.0),
            "home" => Some(VK_HOME.0),
            "end" => Some(VK_END.0),
            "page_up" => Some(VK_PRIOR.0),
            "page_down" => Some(VK_NEXT.0),
            // Symbols
            ";" => Some(0xBA),
            "=" => Some(0xBB),
            "," => Some(0xBC),
            "-" => Some(0xBD),
            "." => Some(0xBE),
            "/" => Some(0xBF),
            "`" => Some(0xC0),
            "[" => Some(0xDB),
            "\\" => Some(0xDC),
            "]" => Some(0xDD),
            "'" => Some(0xDE),
            _ => {
                // Try to parse vk_XXX format
                if key_name.starts_with("vk_") {
                    key_name[3..].parse::<u16>().ok()
                } else {
                    None
                }
            }
        }
    }
}

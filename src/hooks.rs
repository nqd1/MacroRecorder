use std::sync::{Arc, Mutex, OnceLock};
use windows::{
    Win32::{
        Foundation::*,
        UI::WindowsAndMessaging::*,
        UI::Input::KeyboardAndMouse::*,
        System::LibraryLoader::GetModuleHandleW,
    },
};
use crate::events::{MacroEvent, EventType};

type HookCallback = Box<dyn Fn(MacroEvent) + Send + Sync>;

static GLOBAL_HOOKS: OnceLock<Arc<Mutex<Option<GlobalHooks>>>> = OnceLock::new();

pub struct GlobalHooks {
    keyboard_hook: Option<HHOOK>,
    mouse_hook: Option<HHOOK>,
    callback: Option<HookCallback>,
    start_time: std::time::Instant,
}

impl GlobalHooks {
    pub fn new() -> Self {
        Self {
            keyboard_hook: None,
            mouse_hook: None,
            callback: None,
            start_time: std::time::Instant::now(),
        }
    }
    
    pub fn set_callback(&mut self, callback: HookCallback) {
        self.callback = Some(callback);
    }
    
    pub fn install(&mut self) -> std::result::Result<(), String> {
        unsafe {
            let hooks_ref = GLOBAL_HOOKS.get_or_init(|| Arc::new(Mutex::new(None)));
            *hooks_ref.lock().unwrap() = Some(std::ptr::read(self as *const _));
            
            let hinstance = match GetModuleHandleW(None) {
                Ok(h) => h,
                Err(e) => return Err(format!("Failed to get module handle: {}", e)),
            };
            
            self.keyboard_hook = Some(match SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_hook_proc),
                hinstance,
                0,
            ) {
                Ok(hook) => hook,
                Err(e) => return Err(format!("Failed to install keyboard hook: {}", e)),
            });
            
            self.mouse_hook = Some(match SetWindowsHookExW(
                WH_MOUSE_LL,
                Some(mouse_hook_proc),
                hinstance,
                0,
            ) {
                Ok(hook) => hook,
                Err(e) => return Err(format!("Failed to install mouse hook: {}", e)),
            });
            
            self.start_time = std::time::Instant::now();
            log::info!("Global hooks installed successfully");
            Ok(())
        }
    }
    
    pub fn uninstall(&mut self) {
        unsafe {
            if let Some(hooks_ref) = GLOBAL_HOOKS.get() {
                if let Ok(mut guard) = hooks_ref.try_lock() {
                    *guard = None;
                }
            }
            
            std::thread::sleep(std::time::Duration::from_millis(10));
            
            if let Some(hook) = self.keyboard_hook.take() {
                let result = UnhookWindowsHookEx(hook);
                if result.is_err() {
                    log::warn!("Failed to unhook keyboard hook: {:?}", result);
                }
            }
            
            if let Some(hook) = self.mouse_hook.take() {
                let result = UnhookWindowsHookEx(hook);
                if result.is_err() {
                    log::warn!("Failed to unhook mouse hook: {:?}", result);
                }
            }
            
            log::info!("Global hooks uninstalled");
        }
    }
    
    fn handle_keyboard_event(&self, vk_code: u32, scan_code: u32, is_key_down: bool) {
        if let Some(callback) = &self.callback {
            let timestamp = self.start_time.elapsed().as_secs_f64();
            let key_name = vk_code_to_string(vk_code);
            
            let event = MacroEvent {
                timestamp,
                event_type: if is_key_down {
                    EventType::KeyDown
                } else {
                    EventType::KeyUp
                },
                data: serde_json::json!({
                    "vk_code": vk_code,
                    "scan_code": scan_code,
                    "key_name": key_name,
                }),
            };
            
            callback(event);
        }
    }
    
    fn handle_mouse_event(&self, event_type: EventType, x: i32, y: i32, button: Option<u32>, delta: Option<i32>) {
        if let Some(callback) = &self.callback {
            let timestamp = self.start_time.elapsed().as_secs_f64();
            
            let mut data = serde_json::json!({
                "x": x,
                "y": y,
            });
            
            if let Some(btn) = button {
                data["button"] = serde_json::Value::Number(serde_json::Number::from(btn));
            }
            
            if let Some(d) = delta {
                data["delta"] = serde_json::Value::Number(serde_json::Number::from(d));
            }
            
            let event = MacroEvent {
                timestamp,
                event_type,
                data,
            };
            
            callback(event);
        }
    }
}

impl Drop for GlobalHooks {
    fn drop(&mut self) {
        self.uninstall();
    }
}

unsafe extern "system" fn keyboard_hook_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code >= 0 {
        if let Some(hooks_ref) = GLOBAL_HOOKS.get() {
            if let Ok(hooks_guard) = hooks_ref.lock() {
                if let Some(hooks) = hooks_guard.as_ref() {
                    let kbd_struct = *(l_param.0 as *const KBDLLHOOKSTRUCT);
                    let is_key_down = w_param.0 == WM_KEYDOWN as usize || w_param.0 == WM_SYSKEYDOWN as usize;
                    
                    let vk_code = kbd_struct.vkCode;
                    if !(vk_code == VK_CONTROL.0 as u32 || 
                         (vk_code >= 0x52 && vk_code <= 0x52) ||
                         (vk_code >= 0x50 && vk_code <= 0x50) ||
                         (vk_code >= 0x51 && vk_code <= 0x51)) {
                        hooks.handle_keyboard_event(
                            kbd_struct.vkCode,
                            kbd_struct.scanCode,
                            is_key_down,
                        );
                    }
                }
            }
        }
    }
    
    CallNextHookEx(None, n_code, w_param, l_param)
}

unsafe extern "system" fn mouse_hook_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code >= 0 {
        if let Some(hooks_ref) = GLOBAL_HOOKS.get() {
            if let Ok(hooks_guard) = hooks_ref.lock() {
                if let Some(hooks) = hooks_guard.as_ref() {
                    let mouse_struct = *(l_param.0 as *const MSLLHOOKSTRUCT);
                    let x = mouse_struct.pt.x;
                    let y = mouse_struct.pt.y;
                    
                    match w_param.0 as u32 {
                        WM_MOUSEMOVE => {
                            hooks.handle_mouse_event(EventType::MouseMove, x, y, None, None);
                        }
                        WM_LBUTTONDOWN => {
                            hooks.handle_mouse_event(EventType::MouseDown, x, y, Some(1), None);
                        }
                        WM_LBUTTONUP => {
                            hooks.handle_mouse_event(EventType::MouseUp, x, y, Some(1), None);
                        }
                        WM_RBUTTONDOWN => {
                            hooks.handle_mouse_event(EventType::MouseDown, x, y, Some(2), None);
                        }
                        WM_RBUTTONUP => {
                            hooks.handle_mouse_event(EventType::MouseUp, x, y, Some(2), None);
                        }
                        WM_MBUTTONDOWN => {
                            hooks.handle_mouse_event(EventType::MouseDown, x, y, Some(3), None);
                        }
                        WM_MBUTTONUP => {
                            hooks.handle_mouse_event(EventType::MouseUp, x, y, Some(3), None);
                        }
                        WM_MOUSEWHEEL => {
                            let delta = ((mouse_struct.mouseData >> 16) & 0xFFFF) as i16 as i32;
                            hooks.handle_mouse_event(EventType::MouseScroll, x, y, None, Some(delta));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    CallNextHookEx(None, n_code, w_param, l_param)
}

fn vk_code_to_string(vk_code: u32) -> String {
    match vk_code {
        0x41..=0x5A => char::from(vk_code as u8).to_string().to_lowercase(),
        0x30..=0x39 => char::from(vk_code as u8).to_string(),
        0x70..=0x7B => format!("f{}", vk_code - 0x6F),
        val if val == VK_SPACE.0 as u32 => "space".to_string(),
        val if val == VK_RETURN.0 as u32 => "enter".to_string(),
        val if val == VK_BACK.0 as u32 => "backspace".to_string(),
        val if val == VK_TAB.0 as u32 => "tab".to_string(),
        val if val == VK_SHIFT.0 as u32 => "shift".to_string(),
        val if val == VK_CONTROL.0 as u32 => "ctrl".to_string(),
        val if val == VK_MENU.0 as u32 => "alt".to_string(),
        val if val == VK_ESCAPE.0 as u32 => "esc".to_string(),
        val if val == VK_LEFT.0 as u32 => "left".to_string(),
        val if val == VK_UP.0 as u32 => "up".to_string(),
        val if val == VK_RIGHT.0 as u32 => "right".to_string(),
        val if val == VK_DOWN.0 as u32 => "down".to_string(),
        val if val == VK_DELETE.0 as u32 => "delete".to_string(),
        val if val == VK_INSERT.0 as u32 => "insert".to_string(),
        val if val == VK_HOME.0 as u32 => "home".to_string(),
        val if val == VK_END.0 as u32 => "end".to_string(),
        val if val == VK_PRIOR.0 as u32 => "page_up".to_string(),
        val if val == VK_NEXT.0 as u32 => "page_down".to_string(),
        0xBA => ";".to_string(),
        0xBB => "=".to_string(),
        0xBC => ",".to_string(),
        0xBD => "-".to_string(),
        0xBE => ".".to_string(),
        0xBF => "/".to_string(),
        0xC0 => "`".to_string(),
        0xDB => "[".to_string(),
        0xDC => "\\".to_string(),
        0xDD => "]".to_string(),
        0xDE => "'".to_string(),
        _ => format!("vk_{}", vk_code),
    }
}

use super::{load_json, save_json};
use serde::{Deserialize, Serialize};

/// 窗口状态，用于下次启动时恢复
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowState {
    pub width: u32,
    pub height: u32,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub is_maximized: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
            x: None,
            y: None,
            is_maximized: false,
        }
    }
}

pub fn load() -> WindowState {
    load_json("window_state.json")
}

pub fn save(state: &WindowState) -> Result<(), String> {
    save_json("window_state.json", state)
}

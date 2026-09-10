use crate::storage::window::WindowState;

/// 读取窗口状态
#[tauri::command]
pub fn load_window_state() -> WindowState {
    crate::storage::window::load()
}

/// 保存窗口状态
#[tauri::command]
pub fn save_window_state(state: WindowState) -> Result<(), String> {
    crate::storage::window::save(&state)
}

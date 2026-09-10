use crate::storage::preferences::Preferences;

/// 读取用户设置
#[tauri::command]
pub fn load_preferences() -> Preferences {
    crate::storage::preferences::load()
}

/// 保存用户设置
#[tauri::command]
pub fn save_preferences(prefs: Preferences) -> Result<(), String> {
    crate::storage::preferences::save(&prefs)
}

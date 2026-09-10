use super::{load_json, save_json};
use serde::{Deserialize, Serialize};

/// 用户设置（默认中文、跟随系统主题）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    #[serde(default = "default_language")]
    pub language: String,
    /// auto | light | dark
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_font_size")]
    pub font_size: f64,
    #[serde(default = "default_font_family")]
    pub font_family: String,
    #[serde(default = "default_line_height")]
    pub line_height: f64,
    #[serde(default = "default_max_width")]
    pub max_content_width: u32,
    #[serde(default = "default_sidebar_visible")]
    pub sidebar_visible: bool,
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: u32,
    /// outline | recent
    #[serde(default = "default_sidebar_tab")]
    pub sidebar_active_tab: String,
    #[serde(default = "default_code_highlight")]
    pub code_highlight: bool,
    /// 上次打开文件的目录，供文件对话框默认定位
    #[serde(default)]
    pub last_dir: Option<String>,
    /// 上次打开的文件，供启动时自动恢复
    #[serde(default)]
    pub last_opened_path: Option<String>,
}

fn default_language() -> String {
    "zh-CN".into()
}
fn default_theme() -> String {
    "auto".into()
}
fn default_font_size() -> f64 {
    16.0
}
fn default_font_family() -> String {
    "'Microsoft YaHei', 'Segoe UI', system-ui, sans-serif".into()
}
fn default_line_height() -> f64 {
    1.8
}
fn default_max_width() -> u32 {
    900
}
fn default_sidebar_visible() -> bool {
    true
}
fn default_sidebar_width() -> u32 {
    260
}
fn default_sidebar_tab() -> String {
    "outline".into()
}
fn default_code_highlight() -> bool {
    true
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            language: default_language(),
            theme: default_theme(),
            font_size: default_font_size(),
            font_family: default_font_family(),
            line_height: default_line_height(),
            max_content_width: default_max_width(),
            sidebar_visible: default_sidebar_visible(),
            sidebar_width: default_sidebar_width(),
            sidebar_active_tab: default_sidebar_tab(),
            code_highlight: default_code_highlight(),
            last_dir: None,
            last_opened_path: None,
        }
    }
}

pub fn load() -> Preferences {
    load_json("preferences.json")
}

pub fn save(prefs: &Preferences) -> Result<(), String> {
    save_json("preferences.json", prefs)
}

mod commands;
mod file_opener;
mod storage;
mod utils;
// Wi-Fi 传书服务仅在安卓提供
#[cfg(target_os = "android")]
mod wifi_server;

// 桌面专属能力（窗口状态恢复、单实例回调）使用的导入；安卓目标下不存在
#[cfg(not(target_os = "android"))]
use tauri::{Emitter, LogicalPosition, LogicalSize, Manager};

/// 应用入口：注册插件、命令、启动时恢复窗口状态、关闭时保存窗口状态。
/// 移动端需用 mobile_entry_point 宏生成 JNI 入口（桌面不受影响）
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 1. 基础 Builder（dialog 插件桌面/安卓均支持；file_opener 仅安卓注册 Kotlin 插件）
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(file_opener::init());

    // 2. 单实例：仅桌面（Android 无此模型），cfg 块避免属性插在方法链中间
    #[cfg(not(target_os = "android"))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if let Some(path) = args.get(1) {
                let _ = app.emit("open-file", path);
            }
        }));
    }

    // 3. 命令注册：invoke_handler 是「替换」语义，不能分段多次调用；
    //    每平台在唯一一个 cfg 块内一次性注册全部命令
    #[cfg(target_os = "android")]
    {
        builder = builder.invoke_handler(tauri::generate_handler![
            commands::file::open_file_dialog,
            commands::file::read_markdown_file,
            commands::file::get_startup_file,
            commands::file::pull_incoming_file,
            commands::file::choose_browse_folder,
            commands::file::list_browse_files,
            commands::file::open_browse_file,
            commands::file::wifi_lan_address,
            commands::file::wifi_server_start,
            commands::file::wifi_server_stop,
            commands::progress::save_progress,
            commands::recent::get_recent_files,
            commands::recent::add_recent_file,
            commands::recent::remove_recent_file,
            commands::recent::set_recent_starred,
            commands::bookmarks::get_bookmarks,
            commands::bookmarks::add_bookmark,
            commands::bookmarks::remove_bookmark,
            commands::window::load_window_state,
            commands::window::save_window_state,
            commands::preferences::load_preferences,
            commands::preferences::save_preferences,
        ]);
    }

    // 桌面：基础命令 + Windows 专属的 explorer 定位
    #[cfg(not(target_os = "android"))]
    {
        builder = builder.invoke_handler(tauri::generate_handler![
            commands::file::open_file_dialog,
            commands::file::read_markdown_file,
            commands::file::get_startup_file,
            commands::file::reveal_in_explorer,
            commands::progress::save_progress,
            commands::recent::get_recent_files,
            commands::recent::add_recent_file,
            commands::recent::remove_recent_file,
            commands::recent::set_recent_starred,
            commands::bookmarks::get_bookmarks,
            commands::bookmarks::add_bookmark,
            commands::bookmarks::remove_bookmark,
            commands::window::load_window_state,
            commands::window::save_window_state,
            commands::preferences::load_preferences,
            commands::preferences::save_preferences,
        ]);
    }

    builder
        .setup(|app| {
            // 0. 安卓：数据目录指向 App 私有目录（桌面无 APPDATA，落到只读位置会写失败）
            #[cfg(target_os = "android")]
            {
                use tauri::Manager;
                if let Ok(dir) = app.path().app_data_dir() {
                    crate::storage::init_data_dir(dir);
                }
            }
            // 1. 恢复窗口状态（仅桌面；Android 无窗口概念）
            #[cfg(not(target_os = "android"))]
            restore_window_state(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running LiteMD");
}

/// 启动时恢复上次窗口大小 / 位置 / 最大化状态，并在关闭时保存（仅桌面）
#[cfg(not(target_os = "android"))]
fn restore_window_state(app: &tauri::App) {
    // 1. 启动时恢复上次窗口大小 / 位置 / 最大化状态
    let state = storage::window::load();
    if let Some(win) = app.get_webview_window("main") {
        win.set_size(LogicalSize::new(state.width as f64, state.height as f64))
            .ok();
        if state.is_maximized {
            win.maximize().ok();
        } else if let (Some(x), Some(y)) = (state.x, state.y) {
            win.set_position(LogicalPosition::new(x as f64, y as f64)).ok();
        }

        // 2. 窗口关闭时保存窗口状态
        let win_guard = win.clone();
        win.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let mut current = storage::window::load();
                current.is_maximized = win_guard.is_maximized().unwrap_or(false);
                if let Ok(size) = win_guard.inner_size() {
                    current.width = size.width;
                    current.height = size.height;
                }
                if !current.is_maximized {
                    if let Ok(pos) = win_guard.outer_position() {
                        current.x = Some(pos.x);
                        current.y = Some(pos.y);
                    }
                }
                storage::window::save(&current).ok();
            }
        });
    }
}

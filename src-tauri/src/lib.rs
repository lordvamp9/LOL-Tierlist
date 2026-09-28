pub mod models;
pub mod meta_service;

use models::LoLMetaData;
use tauri::{AppHandle, Window};

#[tauri::command]
async fn get_meta_data(
    app: AppHandle,
    server: Option<String>,
    tier: Option<String>,
    force_refresh: Option<bool>,
) -> Result<LoLMetaData, String> {
    let s = server.unwrap_or_else(|| "LAS".to_string());
    let t = tier.unwrap_or_else(|| "DIAMOND".to_string());
    let refresh = force_refresh.unwrap_or(false);
    meta_service::load_meta_data(&app, &s, &t, refresh).await
}

#[tauri::command]
fn minimize_window(window: Window) {
    let _ = window.minimize();
}

#[tauri::command]
fn toggle_maximize_window(window: Window) {
    if window.is_maximized().unwrap_or(false) {
        let _ = window.unmaximize();
    } else {
        let _ = window.maximize();
    }
}

#[tauri::command]
fn close_window(window: Window) {
    let _ = window.close();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_meta_data,
            minimize_window,
            toggle_maximize_window,
            close_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

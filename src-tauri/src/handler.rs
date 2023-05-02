
use os;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
pub fn message_box(msg: &str) {
    os::message_box(msg);
    //format!("Hello, {}! You've been greeted from Rust!", name)
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
pub fn get_logon_logoff_log(size: i32) -> Vec<String> {
    os::get_logon_logoff_log(size)
    //format!("Hello, {}! You've been greeted from Rust!", name)
}


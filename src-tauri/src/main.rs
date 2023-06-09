// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod handler;

fn main() {
    tauri::Builder::default()
        .setup(|app|{
            //let _window = tauri::WindowBuilder::new(app, "label", tauri::WindowUrl::App("index.html".into())).build()?;
            //let main_window = tauri::Manager::get_window(app, "main").unwrap();
            //main_window.eval(&format!("window.location.assign('{}')", tauri::WindowUrl::App("test.html".into()))).unwrap();
            // for (key, value) in tauri::Manager::windows(app) {
            //     println!("{}", key);
            //     value.eval(&format!("window.location.replace('{}')", tauri::WindowUrl::App("test.html".into()))).unwrap();
            // }
            Ok(())
        })
        .on_page_load(|window, payload| {
            if payload.url().chars().last().unwrap() == '/' {
                window.eval(&format!("window.location.assign('{}')", tauri::WindowUrl::App("test.html".into()))).unwrap();
            }
            // println!("{}", payload.url());
            // match payload.url() {
            //     "http://127.0.0.1:1430/" => {
            //         window.eval(&format!("window.location.assign('{}')", tauri::WindowUrl::App("test.html".into()))).unwrap();
            //     },
            //     _ => {},
            // }
        })
        .invoke_handler(tauri::generate_handler![
            handler::message_box,
            handler::get_logon_logoff_log,
            greet,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

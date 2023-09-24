// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

pub mod command;
mod loader;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let loaders = loader::Loaders::init();
            let handle = app.handle();

            loaders.subscribe(move |statuses| {
                handle.emit_all("loader_status_update", statuses).unwrap();
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

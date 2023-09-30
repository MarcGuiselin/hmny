// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod invoke;
mod state;
mod task;

fn main() {
    let state = state::State::new();

    tauri::Builder::default()
        .manage(state.clone())
        .invoke_handler(tauri::generate_handler![
            invoke::clean,
            invoke::build_wraps,
            invoke::compile_wrap
        ])
        .setup(move |app| {
            let state = state.clone();

            // Always start by building wraps
            state.enqueue_task(task::Task::Dev(task::Dev::Cargo(task::Cargo::BuildWraps)));

            // Start listening for task updates
            state.clone().initiate(app.handle());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

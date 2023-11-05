// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use notify::{RecommendedWatcher, Watcher};

mod invoke;
mod state;
mod task;

const CARGO_BUILD_WRAPS_DIR: &str = "../../target/wasm32-unknown-unknown/release";

fn main() {
    let state = state::State::new();

    // Clean wraps on startup
    state.enqueue_task(task::Task::Dev(task::Dev::Cargo(task::Cargo::CleanWraps)));

    // Always start by building wraps
    state.enqueue_task(task::Task::Dev(task::Dev::Cargo(task::Cargo::BuildWraps)));

    // Watch for changes to the wraps directory
    let state_clone = state.clone();
    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| match res {
            Ok(event) => {
                if event.kind.is_create() || event.kind.is_modify() {
                    let path = event.paths.first().unwrap();
                    if path.extension() == Some("wasm".as_ref()) {
                        state_clone.enqueue_task(task::Task::Dev(task::Dev::Polywrap(
                            task::Polywrap::CompileWrap {
                                name: path.file_stem().unwrap().to_str().unwrap().to_string(),
                            },
                        )));
                    }
                }
            }
            Err(e) => println!("watch error: {:?}", e),
        },
        notify::Config::default(),
    )
    .expect("failed to create watcher");
    let _ = std::fs::create_dir_all(CARGO_BUILD_WRAPS_DIR);
    watcher
        .watch(
            CARGO_BUILD_WRAPS_DIR.as_ref(),
            notify::RecursiveMode::NonRecursive,
        )
        .expect("failed to watch path");

    tauri::Builder::default()
        .manage(state.clone())
        .invoke_handler(tauri::generate_handler![
            invoke::clean,
            invoke::build_wraps,
            invoke::compile_wrap
        ])
        .setup(move |app| {
            // Start listening for task updates
            state.initiate(app.handle());

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

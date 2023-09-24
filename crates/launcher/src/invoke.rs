// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::{state, task::*};

type State<'r> = tauri::State<'r, state::State>;

#[tauri::command]
pub async fn clean(state: State<'_>) -> Result<(), String> {
    state
        .enqueue_task(Task::Dev(Dev::Cargo(Cargo::CleanWraps)))
        .await;

    Ok(())
}

#[tauri::command]
pub async fn build_wraps(state: State<'_>) -> Result<(), String> {
    state
        .enqueue_task(Task::Dev(Dev::Cargo(Cargo::BuildWraps)))
        .await;

    Ok(())
}

#[tauri::command]
pub async fn compile_wrap(name: String, state: State<'_>) -> Result<(), String> {
    state
        .enqueue_task(Task::Dev(Dev::CompileWrap { name }))
        .await;

    Ok(())
}

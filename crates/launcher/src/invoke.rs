use crate::{state, task::*};

type State<'r> = tauri::State<'r, state::State>;
type Result<T> = std::result::Result<T, String>;

#[tauri::command]
pub async fn clean(state: State<'_>) -> Result<()> {
    state.enqueue_task(Task::Dev(Dev::Cargo(Cargo::CleanWraps)));

    Ok(())
}

#[tauri::command]
pub async fn build_wraps(state: State<'_>) -> Result<()> {
    state.enqueue_task(Task::Dev(Dev::Cargo(Cargo::BuildWraps)));

    Ok(())
}

#[tauri::command]
pub async fn compile_wrap(name: String, state: State<'_>) -> Result<()> {
    state.enqueue_task(Task::Dev(Dev::CompileWrap { name }));

    Ok(())
}

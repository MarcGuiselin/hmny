use super::{CargoCommand, Handle, Status, StatusSender};
use std::{
    io::{Error, ErrorKind, Result},
    path::Path,
    sync::Arc,
};
use tokio::{fs, sync::Mutex};

enum Step {
    CargoClean,
    DeleteArtifacts,
    Done,
}

struct Inner {
    handle: Handle,
    error: Option<String>,
    step: Step,
}

pub fn start_task(update_sender: StatusSender) -> Handle {
    let handle = Handle::new();
    let inner = Arc::new(Mutex::new(Inner {
        handle: handle.clone(),
        error: None,
        step: Step::CargoClean,
    }));

    let inner_handle = Arc::clone(&inner);
    tauri::async_runtime::spawn(async move {
        match initiate(update_sender.clone(), &inner_handle).await {
            Ok(_) => {}
            Err(error) => {
                let mut inner = inner_handle.lock().await;
                inner.error = Some(error.to_string());
                update_sender.send(get_status(&inner)).await.unwrap();
            }
        }
    });

    handle
}

async fn initiate(update_sender: StatusSender, inner_handle: &Arc<Mutex<Inner>>) -> Result<()> {
    // Step 1: cargo clean
    send_status(&update_sender, &inner_handle, Step::CargoClean).await;

    let output = CargoCommand::new("clean")
        .args(&["--target", "wasm32-unknown-unknown", "--release"])
        .command
        .output()
        .await?;

    if !output.status.success() {
        return Err(Error::new(
            ErrorKind::Other,
            format!("cargo clean failed exit code: {}", output.status),
        ));
    }

    // Step 2: Delete all remaining artifacts
    send_status(&update_sender, &inner_handle, Step::DeleteArtifacts).await;
    delete_directory_contents("../../target/wasm32-unknown-unknown/release").await?;

    // Done
    send_status(&update_sender, &inner_handle, Step::Done).await;

    Ok(())
}

async fn send_status(update_sender: &StatusSender, inner_handle: &Arc<Mutex<Inner>>, step: Step) {
    let mut inner = inner_handle.lock().await;
    inner.step = step;
    update_sender.send(get_status(&inner)).await.unwrap();
}

async fn delete_directory_contents(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if let Ok(mut entries) = fs::read_dir(&path).await {
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                fs::remove_dir_all(path).await?;
            } else {
                fs::remove_file(path).await?;
            }
        }
    }

    Ok(())
}

fn get_status(inner: &Inner) -> Status {
    let title = "Cleaning Wraps".into();
    let status = None;
    let ratio = match inner.step {
        Step::CargoClean => 0.0,
        Step::DeleteArtifacts => 0.5,
        Step::Done => 1.0,
    };

    Status {
        handle: inner.handle.clone(),
        title,
        status,
        done_ratio: ratio,
        doing_ratio: ratio,
        error: inner.error.clone(),
    }
}

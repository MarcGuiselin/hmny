use super::{package_dependency_count, CargoCommand, Ready, Status, StatusSender};
use std::{io::Result, process::Stdio, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::Mutex,
};
use uuid::Uuid;

const PACKAGES: &'static [&'static str] = &["homescreen_default", "mimetype_markdown", "test_wrap"];

#[derive(Default)]
struct Inner {
    id: Uuid,
    error: Option<String>,
    log: String,
    max: usize,
    completed: usize,
    is_compiling: bool,
}

pub fn create_ready(update_sender: StatusSender) -> Ready {
    let id = Uuid::new_v4();
    let inner = Arc::new(Mutex::new(Inner {
        id,
        ..Inner::default()
    }));

    let inner_handle = Arc::clone(&inner);
    tauri::async_runtime::spawn(async move {
        match initiate(update_sender, &inner_handle).await {
            Ok(_) => {}
            Err(error) => {
                let mut inner = inner_handle.lock().await;
                inner.error = Some(error.to_string());
            }
        }
    });

    Ready::new(id, Box::new(inner))
}

async fn initiate(update_sender: StatusSender, inner_handle: &Arc<Mutex<Inner>>) -> Result<()> {
    {
        let max = package_dependency_count(PACKAGES).await?;
        let mut inner = inner_handle.lock().await;
        inner.max = max;

        // Send the first update
        update_sender.send(get_status(&inner)).await.unwrap();
    }

    let mut child: tokio::process::Child = CargoCommand::new("build")
        .args(&["--target", "wasm32-unknown-unknown", "-r", "--verbose"])
        .packages(PACKAGES)
        .command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = BufReader::new(child.stdout.take().unwrap());
    handle_stream(stdout, &inner_handle, &update_sender);

    let stderr = BufReader::new(child.stderr.take().unwrap());
    handle_stream(stderr, &inner_handle, &update_sender);

    Ok(())
}

fn handle_stream<R: tokio::io::AsyncRead + Unpin + Send + 'static>(
    reader: R,
    inner_handle: &Arc<Mutex<Inner>>,
    update_sender: &StatusSender,
) {
    let inner = Arc::clone(inner_handle);
    let update_sender = update_sender.clone();
    tauri::async_runtime::spawn(async move {
        let mut buf_reader = BufReader::new(reader).lines();
        while let Some(line) = buf_reader.next_line().await.expect("Failed to read line") {
            let line = line.trim();

            let mut inner = inner.lock().await;
            inner.log += line;
            inner.log += "\n";

            if inner.completed < inner.max {
                let mut changed = false;

                // If the previous line was is_compiling, we consider it done when we move to the next line
                // (Yes I know that the build process is multi-threaded so this might not be true)
                if inner.is_compiling {
                    inner.completed += 1;
                    inner.is_compiling = false;
                    changed = true;
                }

                if line.starts_with("Compiling") {
                    inner.is_compiling = true;
                } else if line.starts_with("Fresh") {
                    inner.completed += 1;
                    changed = true;
                }
                if line.starts_with("Finished") {
                    inner.completed = inner.max;
                    changed = true;
                }

                if changed {
                    update_sender.send(get_status(&inner)).await.unwrap();
                }
            }
        }
    });
}

fn get_status(inner: &Inner) -> Status {
    let title = "Building Wraps".into();
    let status = Some(format!("Completed {}/{}", inner.completed, inner.max));
    let max = inner.max as f32;
    let done_ratio = inner.completed as f32 / max;
    let doing_ratio = (inner.completed as f32 + if inner.is_compiling { 1. } else { 0. }) / max;

    Status {
        id: inner.id,
        title,
        status,
        done_ratio,
        doing_ratio,
        error: inner.error.clone(),
    }
}

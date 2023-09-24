use super::{Loader, TaskStatus, UpdateSender};
use crate::command::{package_dependency_count, CargoCommand};
use std::{
    io::{BufRead, BufReader, Result},
    process::Stdio,
    str,
    sync::{Arc, Mutex},
    thread,
};

#[derive(Clone, Default)]
pub struct WrapLoader {
    error: Option<String>,
    inner: Arc<Mutex<WrapLoaderInner>>,
    max: usize,
}

#[derive(Clone, Default)]
struct WrapLoaderInner {
    log: String,
    completed: usize,
    is_compiling: bool,
}

impl WrapLoader {
    pub const PACKAGES: &'static [&'static str] =
        &["homescreen_default", "mimetype_markdown", "test_wrap"];

    pub fn new_boxed(update_sender: &UpdateSender) -> Box<Self> {
        let loader = match Self::new(update_sender) {
            Ok(loader) => loader,
            Err(error) => Self {
                error: Some(error.to_string()),
                ..Self::default()
            },
        };
        Box::new(loader)
    }

    fn start_reader_thread<R: BufRead + Send + 'static>(
        &self,
        reader: R,
        update_sender: &UpdateSender,
    ) -> thread::JoinHandle<()> {
        let update_sender = update_sender.clone();
        let inner = Arc::clone(&self.inner);
        let max = self.max;
        thread::spawn(move || {
            for line in reader.lines() {
                let line = &line.expect("Failed to read line");
                let line = line.trim();
                let mut inner = inner.lock().expect("Failed to acquire lock");

                inner.log += line;
                inner.log += "\n";

                if inner.completed < max {
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
                        inner.completed = max;
                        changed = true;
                    }

                    if changed {
                        update_sender.send(()).expect("Failed to send update");
                    }
                }
            }
        })
    }

    fn new(update_sender: &UpdateSender) -> Result<Self> {
        let max = package_dependency_count(Self::PACKAGES)?;
        let item = Self {
            max,
            ..Default::default()
        };

        let mut command = CargoCommand::new("build")
            .args(&["--target", "wasm32-unknown-unknown", "-r", "--verbose"])
            .packages(Self::PACKAGES)
            .command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        item.start_reader_thread(
            BufReader::new(command.stdout.take().unwrap()),
            update_sender,
        );
        item.start_reader_thread(
            BufReader::new(command.stderr.take().unwrap()),
            update_sender,
        );

        Ok(item)
    }
}

impl Loader for WrapLoader {
    fn get_status(&self) -> TaskStatus {
        let inner = self.inner.lock().unwrap();

        let title = "Building Wraps".into();
        let status = Some(format!("Completed {}/{}", inner.completed, self.max));
        let max = self.max as f32;
        let done_ratio = inner.completed as f32 / max;
        let doing_ratio = (inner.completed as f32 + if inner.is_compiling { 1. } else { 0. }) / max;

        TaskStatus {
            title,
            status,
            done_ratio,
            doing_ratio,
            error: self.error.clone(),
        }
    }
}

use bevy::prelude::*;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::{
    path::Path,
    sync::{
        mpsc::{Receiver, TryRecvError},
        Arc, Mutex,
    },
};

const ELEMENTS_LOAD_DIR: &str = "./target/wasm32-unknown-unknown/release";

pub struct ElementFileWatcherPlugin;

struct ElementFileWatcherInner {
    watcher: RecommendedWatcher,
    receiver: Mutex<Receiver<Result<Event>>>,
}

impl ElementFileWatcherInner {
    pub fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        let watcher = RecommendedWatcher::new(
            move |res| {
                sender.send(res).expect("Watch event send failure.");
            },
            default(),
        )
        .expect("Failed to create filesystem watcher.");

        let receiver = Mutex::from(receiver);
        Self { watcher, receiver }
    }

    /// Watch for changes recursively at the provided path.
    pub fn watch<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        self.watcher
            .watch(path.as_ref(), RecursiveMode::NonRecursive)
    }
}

#[derive(Clone, Resource)]
pub struct ElementFileWatcher {
    inner: Arc<ElementFileWatcherInner>,
}

impl Default for ElementFileWatcher {
    fn default() -> Self {
        let mut inner = ElementFileWatcherInner::new();

        inner
            .watch(ELEMENTS_LOAD_DIR)
            .expect("Failed to watch path.");

        let inner = Arc::new(inner);
        Self { inner }
    }
}

fn elements_file_watcher_system(file_watcher: Res<ElementFileWatcher>) {
    if let Ok(receiver) = file_watcher.inner.receiver.lock() {
        loop {
            let Event { kind, paths, .. } = match receiver.try_recv() {
                Ok(result) => result.unwrap(),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => panic!("FilesystemWatcher disconnected."),
            };

            paths.iter().for_each(|path| {
                // We only load wasm files for now
                match path.extension() {
                    Some(ext) if ext == "wasm" => match kind {
                        EventKind::Create(_) => {
                            println!("Load: {:?}", path.file_stem());
                        }
                        EventKind::Remove(_) => {
                            println!("Unload: {:?}", path.file_stem());
                        }
                        _ => {
                            println!("Unknown file watcher event: {:?} {:?}", path, kind);
                        }
                    },
                    _ => {}
                }
            });
        }
    }
}

impl Plugin for ElementFileWatcherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ElementFileWatcher>()
            .add_systems(PostUpdate, elements_file_watcher_system);
    }
}

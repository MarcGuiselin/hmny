use super::Wraps;
use bevy::prelude::*;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher};
use std::{
    fs,
    path::Path,
    sync::{
        mpsc::{Receiver, TryRecvError},
        Arc, Mutex,
    },
};

const WRAPS_LOAD_DIR: &str = "./target/polywrap";

pub struct WrapFileWatcherPlugin;

struct WrapFileWatcherInner {
    watcher: RecommendedWatcher,
    receiver: Mutex<Receiver<Result<Event>>>,
}

impl WrapFileWatcherInner {
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
pub struct WrapFileWatcher {
    inner: Arc<WrapFileWatcherInner>,
}

impl Default for WrapFileWatcher {
    fn default() -> Self {
        let mut inner = WrapFileWatcherInner::new();

        let path = Path::new(WRAPS_LOAD_DIR);
        let _ = fs::create_dir_all(path);
        inner.watch(path).expect("Failed to watch path.");

        let inner = Arc::new(inner);
        Self { inner }
    }
}

#[derive(serde::Serialize)]
pub struct PingArgs {
    pub message: String,
}

#[derive(serde::Deserialize)]
pub struct PongResult {
    pub response: String,
}

fn ping_test_wrap() {
    use polywrap::*;

    let start = std::time::Instant::now();
    let mut config = ClientConfig::new();
    config.add(SystemClientConfig::default().into());
    let client = Client::new(config.build());
    println!("Polywrap client created in {:?}", start.elapsed());

    let start = std::time::Instant::now();
    let add_file_resp = client
        .invoke::<PongResult>(
            &Uri::try_from("wrap://fs/C:/Users/Marc/Documents/Projects/Harmony/hmny/target/polywrap/release/test_wrap").unwrap(),
            "ping",
            Some(
                &to_vec(&PingArgs {
                    message: "Hello from Rust!".to_string(),
                })
                .unwrap(),
            ),
            None,
            None,
        )
        .unwrap();
    println!("Resolving wrap and invoking took {:?}", start.elapsed());
    println!("Response to ping: '{}'", add_file_resp.response);

    let start = std::time::Instant::now();
    let add_file_resp = client
        .invoke::<PongResult>(
            &Uri::try_from("wrap://fs/C:/Users/Marc/Documents/Projects/Harmony/hmny/target/polywrap/release/test_wrap").unwrap(),
            "ping",
            Some(
                &to_vec(&PingArgs {
                    message: "Hello from Rust 2!".to_string(),
                })
                .unwrap(),
            ),
            None,
            None,
        )
        .unwrap();
    println!("Resolving wrap and invoking 2 took {:?}", start.elapsed());
    println!("Response to ping 2: '{}'", add_file_resp.response);
}

fn wraps_load_from_dir_system(mut wraps: ResMut<Wraps>) {
    ping_test_wrap();

    let paths = fs::read_dir(WRAPS_LOAD_DIR).expect("Failed to read wraps load directory");

    paths.into_iter().for_each(|path| {
        let path = path.unwrap().path();

        match path.extension() {
            Some(ext) if ext == "wasm" => {
                // if let Err(res) = wraps.load_from_path(path.clone()) {
                //     error!("Error while attempting to load plugin {:?}", path);
                //     error!("    {:?}", res);
                // }
            }
            _ => {}
        }
    });
}

fn wraps_file_watcher_system(mut wraps: ResMut<Wraps>, file_watcher: Res<WrapFileWatcher>) {
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
                            wraps.load_from_path(path).unwrap();
                        }
                        EventKind::Remove(_) => {
                            wraps.unload_from_path(path).unwrap();
                        }
                        _ => {
                            warn!("Unknown file watcher event: {:?} {:?}", path, kind);
                        }
                    },
                    _ => {}
                }
            });
        }
    }
}

impl Plugin for WrapFileWatcherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WrapFileWatcher>()
            .add_systems(PreStartup, wraps_load_from_dir_system);
        //.add_systems(PostUpdate, wraps_file_watcher_system);
    }
}

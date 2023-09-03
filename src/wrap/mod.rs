use bevy::prelude::*;

mod file_watcher;
pub use file_watcher::*;
mod loader;
pub use loader::*;

pub struct WrapPlugin;

impl Plugin for WrapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((WrapFileWatcherPlugin, WrapLoaderPlugin));
    }
}

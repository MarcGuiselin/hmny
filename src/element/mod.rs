use bevy::prelude::*;

mod file_watcher;
pub use file_watcher::*;
mod loader;
pub use loader::*;

pub struct ElementPlugin;

impl Plugin for ElementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ElementFileWatcherPlugin, ElementLoaderPlugin));
    }
}

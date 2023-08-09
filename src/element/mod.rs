use bevy::prelude::*;

mod loader;
pub use loader::*;

pub struct ElementPlugin;

impl Plugin for ElementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ElementLoaderPlugin);
    }
}

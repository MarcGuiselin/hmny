use bevy::{prelude::*, window::PresentMode};

mod dimension;
use dimension::DimensionPlugin;

pub struct HarmonyPlugin;

impl Plugin for HarmonyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Harmony Browser".into(),
                    present_mode: PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }),
            DimensionPlugin,
        ));
    }
}

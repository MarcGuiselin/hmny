use bevy::{prelude::*, window::PresentMode};
use bevy_framepace::FramepacePlugin;

mod dimension;
mod history;
mod text;
mod wrap;

pub struct HarmonyPlugin;

impl Plugin for HarmonyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Harmony Browser".into(),
                    // Disable vsync to lower input latency
                    // Done by examples in https://github.com/aevyrie/bevy_mod_picking
                    // See this issue: https://github.com/aevyrie/bevy_mod_raycast/issues/14
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
            FramepacePlugin,
            // Harmony Core Plugins
            dimension::DimensionPlugin,
            wrap::WrapPlugin,
            text::TextPlugin,
            history::HistoryPlugin,
        ));
    }
}

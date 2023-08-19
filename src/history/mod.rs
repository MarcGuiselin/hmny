use crate::element::{ElementKey, Elements};
use bevy::prelude::*;
use hmny_common::prelude::*;

pub struct HistoryPlugin;

impl Plugin for HistoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut elements: ResMut<Elements>) {
    match elements.signal(ElementKey::HomeScreen, Signal::AskHomeScreen) {
        Ok(Signal::HomeScreen { mime_type, data }) => {
            println!(
                r#"Load home screen with mimetype: "{}" data: "{:?}""#,
                mime_type, data
            );
        }
        other => {
            println!("Could not load home screen data: {:?}", other);
        }
    }
}
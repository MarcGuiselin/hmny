use crate::wrap::{WrapKey, Wraps};
use bevy::prelude::*;
use hmny_common::prelude::*;

pub struct HistoryPlugin;

impl Plugin for HistoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut wraps: ResMut<Wraps>) {
    match wraps.signal(WrapKey::HomeScreen, HomescreenQuery::AskHomeScreen) {
        Ok(HomescreenResponse::HomeScreen { mime_type, data }) => {
            println!(
                r#"Load home screen with mimetype: "{}" data: "{:?}""#,
                mime_type, data
            );

            match wraps.signal(
                WrapKey::Mimetype(mime_type),
                MimetypeQuery::AskParse { data },
            ) {
                Ok(MimetypeResponse::Dimension(dimension)) => {
                    println!(r#"Load dimension: "{:?}""#, dimension);
                }
                other => {
                    println!("Could not load dimension: {:?}", other);
                }
            }
        }
        other => {
            println!("Could not load home screen data: {:?}", other);
        }
    }
}

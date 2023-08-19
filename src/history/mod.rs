use crate::element::{ElementKey, Elements};
use bevy::prelude::*;
use hmny_common::prelude::*;

pub struct HistoryPlugin;

impl Plugin for HistoryPlugin {
    fn build(&self, app: &mut App) {
        // Enable this to test the scene serialization
        //app.add_systems(PreStartup, test_save_scene_system);
        app.add_systems(Startup, setup);
    }
}

fn test_save_scene_system() {
    // Initialize bevy world
    let mut world = World::new();
    let registry = AppTypeRegistry::default();
    world.insert_resource(registry);

    // Text
    world.spawn(
        TextBundle::from_section(
            "hello\nbevy!",
            TextStyle {
                font_size: 100.0,
                color: Color::SEA_GREEN,
                ..default()
            },
        )
        .with_text_alignment(TextAlignment::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(15.0),
            ..default()
        }),
    );

    // Serialize scene
    let type_registry = world.resource::<AppTypeRegistry>();
    let scene = DynamicScene::from_world(&world);
    let serialized_scene = scene.serialize_ron(type_registry).unwrap();

    // Showing the scene in the console
    info!("{}", serialized_scene);
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

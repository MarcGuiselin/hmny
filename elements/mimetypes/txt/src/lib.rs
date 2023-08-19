use bevy::prelude::*;
use hmny_common::prelude::*;

// Macro that defines metadata and signal matcher
define_element! {
    publisher: Publisher::new("Harmony", vec![]),
    element_type: ElementType::MimeType("txt".into()),
    signals: match signal {
        Signal::AskDimension(data) => dimension(data),
    }
}

// Receive data from application
fn dimension(data: &DataType) -> SignalResult {
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

    // Return to application
    Ok(Signal::Dimension { serialized_scene })
}

// fn dimension(data: &DataType) -> SignalResult {
//     let serialized_scene = format!(
//         r###"
// #version 330
//
// in vec4 v_color;
// out vec4 color;
//
// void main() {{
//     color = v_color;
// }};
// "###
//     );
//
//     Ok(Signal::Dimension { serialized_scene })
// }

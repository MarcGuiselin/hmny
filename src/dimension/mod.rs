use bevy::{prelude::*, window::ReceivedCharacter};

#[derive(Resource)]
struct Cursor {
    visible: bool,
    x: f32,
    y: f32,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            visible: false,
            x: 0.,
            y: 0.,
        }
    }
}

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct FollowMouse;

pub struct DimensionPlugin;

impl Plugin for DimensionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Cursor>()
            .insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)))
            .add_systems(Startup, setup)
            .add_systems(PreUpdate, cursor_system)
            .add_systems(Update, (follow_mouse_update, print_char_event_system));
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), MainCamera));

    // Rectangle
    commands.spawn((
        FollowMouse,
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(50.0, 100.0)),
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        },
    ));
}

fn follow_mouse_update(
    cursor: Res<Cursor>,
    mut sprites: Query<(&mut Transform, &mut Visibility), With<FollowMouse>>,
) {
    let Cursor { visible, x, y } = *cursor;
    for (mut transform, mut visibility) in sprites.iter_mut() {
        transform.translation.x = x;
        transform.translation.y = y;
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn cursor_system(
    mut cursor: ResMut<Cursor>,
    windows: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera.single();
    let world_position = windows
        .get_single()
        .ok()
        .and_then(|window| window.cursor_position())
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate());

    cursor.visible = world_position.is_some();
    if let Some(Vec2 { x, y }) = world_position {
        cursor.x = x;
        cursor.y = y;
        gizmos.circle_2d(Vec2 { x, y }, 10.0, Color::GREEN);
    }
}

/// This system prints out all char events as they come in
fn print_char_event_system(mut char_input_events: EventReader<ReceivedCharacter>) {
    for event in char_input_events.read() {
        info!("{:?}: '{}'", event, event.char);
    }
}

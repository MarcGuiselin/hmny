use crate::text::{
    bevy_color_to_cosmic, ActiveEditor, Attrs, AttrsList, AttrsOwned, Buffer, BufferLine,
    CosmicAttrs, CosmicEditSpriteBundle, CosmicEditor, CosmicMetrics, CosmicTextPosition, Editor,
    Family, Metrics, ReadOnly, Shaping,
};
use crate::wrap::{WrapKey, Wraps};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use hmny_common::prelude::*;

pub struct HistoryPlugin;

impl Plugin for HistoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup, setup_test_ui));
    }
}

fn setup(
    mut wraps: ResMut<Wraps>,
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
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
                    println!(r#"Loading dimension: "{:?}""#, dimension);
                    let mut count = 0;
                    dimension.children.into_iter().for_each(|entity| {
                        let primary_window = windows.single();
                        let scale_factor = primary_window.scale_factor() as f32;
                        summon_entity(entity, &mut commands, scale_factor, count);
                        count += 1;
                    });
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

fn summon_entity(
    entity: hmny_common::prelude::Entity,
    commands: &mut Commands,
    scale_factor: f32,
    count: u16,
) {
    use hmny_common::prelude::{Component as Comp, Style as Sty};

    let root_entity = commands.spawn_empty().id();
    let mut is_visible = false;
    let mut has_transform = false;

    for component in entity.components.into_iter() {
        match component {
            Comp::Location2D(_) | Comp::Location3D(_) => {
                if has_transform {
                    return;
                }
                has_transform = true;

                let (rotation, translation) = match component {
                    Comp::Location2D(location) => {
                        let position: Vec2 = location.position.into();
                        (
                            location.rotation.into(),
                            Vec3::new(position.x, position.y, 0.),
                        )
                    }
                    Comp::Location3D(location) => {
                        (location.rotation.into(), location.position.into())
                    }
                    _ => unreachable!(),
                };

                commands
                    .get_entity(root_entity)
                    .unwrap()
                    .insert(TransformBundle {
                        local: Transform {
                            translation,
                            rotation,
                            scale: Vec3::ONE,
                        },
                        ..Default::default()
                    });
            }
            Comp::Text(hmny_common::prelude::Text {
                color: default_color,
                font_size,
                line_height,
                spans,
            }) => {
                is_visible = true;
                let line_height = font_size * line_height;

                let mut buffer =
                    Buffer::new_empty(Metrics::new(font_size, line_height).scale(scale_factor));

                let mut line_text = String::new();
                let attrs = Attrs::new().family(Family::Name(FONT_NAME));
                let mut attrs_list = AttrsList::new(attrs);
                for TextSpan {
                    text,
                    color: override_color,
                    style,
                    weight,
                } in spans.into_iter()
                {
                    let color = override_color.as_ref().unwrap_or(&default_color);
                    let attrs = Attrs::new()
                        .style(match style {
                            Sty::Normal => cosmic_text::Style::Normal,
                            Sty::Italic => cosmic_text::Style::Italic,
                            Sty::Oblique => cosmic_text::Style::Oblique,
                        })
                        .weight(cosmic_text::Weight(weight))
                        .color(cosmic_text::Color::rgba(color.r, color.g, color.b, 255));

                    let start = line_text.len();
                    line_text.push_str(&text);
                    let end = line_text.len();
                    attrs_list.add_span(start..end, attrs);
                }
                buffer
                    .lines
                    .push(BufferLine::new(line_text, attrs_list, Shaping::Advanced));

                let editor = CosmicEditor(Editor::new(buffer));
                let inner_entity = commands
                    .spawn((
                        ReadOnly,
                        CosmicEditSpriteBundle {
                            sprite: Sprite {
                                custom_size: Some(Vec2::new(400., 100.)),
                                ..Default::default()
                            },
                            cosmic_metrics: CosmicMetrics {
                                font_size,
                                line_height,
                                scale_factor,
                            },
                            text_position: CosmicTextPosition::TopLeft,
                            cosmic_attrs: CosmicAttrs(AttrsOwned::new(attrs)),
                            editor,
                            transform: Transform::from_translation(Vec3::new(
                                0.,
                                -110.0 * count as f32,
                                10.,
                            )),
                            background_color: BackgroundColor(Color::rgba(1., 0., 0., 0.2)),
                            ..Default::default()
                        },
                    ))
                    .id();

                commands
                    .get_entity(root_entity)
                    .unwrap()
                    .add_child(inner_entity);
            }
        }
    }

    if !has_transform {
        commands
            .get_entity(root_entity)
            .unwrap()
            .insert(TransformBundle::default());
    }
    if is_visible {
        commands
            .get_entity(root_entity)
            .unwrap()
            .insert(VisibilityBundle::default());
    }
}

const FONT_NAME: &str = "Atkinson Hyperlegible";
const FONT_SIZE: f32 = 16.;
const LINE_HEIGHT: f32 = 24.;

fn setup_test_ui(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut active_editor: ResMut<ActiveEditor>,
    // mut font_system: ResMut<CosmicFontSystem>,
) {
    let primary_window = windows.single();
    let scale_factor = primary_window.scale_factor() as f32;

    let attrs = Attrs::new()
        .family(Family::Name(FONT_NAME))
        .color(bevy_color_to_cosmic(Color::BLACK));

    let buffer = {
        let mut buffer =
            Buffer::new_empty(Metrics::new(FONT_SIZE, LINE_HEIGHT).scale(scale_factor));

        let lines: Vec<Vec<(String, AttrsOwned)>> = vec![
            vec![
                (
                    String::from("B"),
                    AttrsOwned::new(attrs.weight(cosmic_text::Weight::BOLD)),
                ),
                (String::from("old "), AttrsOwned::new(attrs)),
                (
                    String::from("I"),
                    AttrsOwned::new(attrs.style(cosmic_text::Style::Italic)),
                ),
                (String::from("talic "), AttrsOwned::new(attrs)),
                (String::from("f"), AttrsOwned::new(attrs)),
                (String::from("i "), AttrsOwned::new(attrs)),
                (
                    String::from("f"),
                    AttrsOwned::new(attrs.weight(cosmic_text::Weight::BOLD)),
                ),
                (String::from("i "), AttrsOwned::new(attrs)),
                (
                    String::from("f"),
                    AttrsOwned::new(attrs.style(cosmic_text::Style::Italic)),
                ),
                (String::from("i "), AttrsOwned::new(attrs)),
            ],
            vec![
                (String::from("Sans-Serif Normal "), AttrsOwned::new(attrs)),
                (
                    String::from("Sans-Serif Bold "),
                    AttrsOwned::new(attrs.weight(cosmic_text::Weight::BOLD)),
                ),
                (
                    String::from("Sans-Serif Italic "),
                    AttrsOwned::new(attrs.style(cosmic_text::Style::Italic)),
                ),
                (
                    String::from("Sans-Serif Bold Italic"),
                    AttrsOwned::new(
                        attrs
                            .weight(cosmic_text::Weight::BOLD)
                            .style(cosmic_text::Style::Italic),
                    ),
                ),
            ],
            vec![
                (
                    String::from("R"),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0x00, 0x00))),
                ),
                (
                    String::from("A"),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0x7F, 0x00))),
                ),
                (
                    String::from("I"),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0xFF, 0x00))),
                ),
                (
                    String::from("N"),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x00, 0xFF, 0x00))),
                ),
                (
                    String::from("B"),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x00, 0x00, 0xFF))),
                ),
                (
                    String::from("O"),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x4B, 0x00, 0x82))),
                ),
                (
                    String::from("W "),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x94, 0x00, 0xD3))),
                ),
                (
                    String::from("Red "),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0x00, 0x00))),
                ),
                (
                    String::from("Orange "),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0x7F, 0x00))),
                ),
                (
                    String::from("Yellow "),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0xFF, 0x00))),
                ),
                (
                    String::from("Green "),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x00, 0xFF, 0x00))),
                ),
                (
                    String::from("Blue "),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x00, 0x00, 0xFF))),
                ),
                (
                    String::from("Indigo "),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x4B, 0x00, 0x82))),
                ),
                (
                    String::from("Violet "),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x94, 0x00, 0xD3))),
                ),
                (
                    String::from("U"),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x94, 0x00, 0xD3))),
                ),
                (
                    String::from("N"),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x4B, 0x00, 0x82))),
                ),
                (
                    String::from("I"),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x00, 0x00, 0xFF))),
                ),
                (
                    String::from("C"),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0x00, 0xFF, 0x00))),
                ),
                (
                    String::from("O"),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0xFF, 0x00))),
                ),
                (
                    String::from("R"),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0x7F, 0x00))),
                ),
                (
                    String::from("N"),
                    AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0x00, 0x00))),
                ),
            ],
            vec![(
                String::from("生活,삶,जिंदगी 😀 FPS"),
                AttrsOwned::new(attrs.color(cosmic_text::Color::rgb(0xFF, 0x00, 0x00))),
            )],
        ];

        buffer.lines.clear();
        for line in lines {
            let mut line_text = String::new();
            let mut attrs_list = AttrsList::new(attrs);
            for (text, attrs) in line.iter() {
                let start = line_text.len();
                line_text.push_str(text);
                let end = line_text.len();
                attrs_list.add_span(start..end, attrs.as_attrs());
            }
            buffer
                .lines
                .push(BufferLine::new(line_text, attrs_list, Shaping::Advanced));
        }
        buffer
    };

    let cosmic_edit = commands
        .spawn(CosmicEditSpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(400., 200.)),
                ..Default::default()
            },
            cosmic_metrics: CosmicMetrics {
                font_size: FONT_SIZE,
                line_height: LINE_HEIGHT,
                scale_factor,
            },
            text_position: CosmicTextPosition::TopLeft,
            cosmic_attrs: CosmicAttrs(AttrsOwned::new(attrs)),
            editor: CosmicEditor(Editor::new(buffer)),
            transform: Transform::from_translation(Vec3::new(-400., 200., 10.)),
            ..Default::default()
        })
        .id();

    active_editor.replace(cosmic_edit);
}

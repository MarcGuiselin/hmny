use crate::text::{
    bevy_color_to_cosmic, ActiveEditor, Attrs, AttrsList, AttrsOwned, Buffer, BufferLine,
    CosmicAttrs, CosmicEditSpriteBundle, CosmicEditor, CosmicMetrics, CosmicTextPosition, Editor,
    Family, Metrics, Shaping,
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

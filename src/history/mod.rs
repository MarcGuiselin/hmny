use crate::canvas;
use crate::canvas::layout;
use crate::wrap::{WrapKey, Wraps};
use bevy::prelude::*;
use hmny_common::prelude::*;

pub struct HistoryPlugin;

impl Plugin for HistoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut wraps: ResMut<Wraps>, mut commands: Commands, mut images: ResMut<Assets<Image>>) {
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
                    let dimension_entity = commands
                        .spawn((TransformBundle::default(), VisibilityBundle::default()))
                        .id();
                    for element in dimension.children.into_iter() {
                        summon_element(element, dimension_entity, &mut commands, &mut images);
                    }
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

fn summon_element(
    element: hmny_common::prelude::Element,
    dimension_entity: Entity,
    commands: &mut Commands,
    images: &mut ResMut<Assets<Image>>,
) {
    match element {
        Element::Canvas(Canvas { texts }) => {
            println!("Canvas: {:?}", texts);
            let canvas_entity = commands
                .spawn(canvas::CanvasBundle {
                    canvas: canvas::Canvas {
                        max_dimension: 400.,
                        layout: layout::Layout::FlexBasic(layout::FlexBasic {
                            direction: layout::Direction::Vertical,
                            gap: 10.,
                        }),
                    },
                    transform: Transform {
                        translation: Vec3::new(-200., 200., 0.),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .set_parent(dimension_entity)
                .id();

            for text in texts.into_iter() {
                //texts.pop();
                let texture = images.add(Image::default());
                commands
                    .spawn(canvas::RichTextBundle {
                        rich_text: canvas::RichText(text),
                        texture,
                        sprite: Sprite {
                            anchor: bevy::sprite::Anchor::TopLeft,
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .set_parent(canvas_entity);
            }
        }
    }
}

use crate::*;
use std::sync::Mutex;

mod ffi;
pub mod layout;

#[derive(Default)]
pub struct CanvasPlugin;

impl Plugin for CanvasPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup).add_systems(
            Update,
            (
                on_rich_text_change.before(on_canvas_change),
                on_canvas_change,
            ),
        );
    }
}

// TODO: I'm unsure about whether it's thread-safe to use context this way. Need to do more research
#[derive(Resource)]
pub struct PangoContext(Mutex<pango::Context>);
impl PangoContext {
    // Pango context is kept safe by requiring we always get a mutable reference to PangoContext
    pub fn lock(&mut self) -> std::sync::LockResult<std::sync::MutexGuard<'_, pango::Context>> {
        self.0.lock()
    }
}
unsafe impl Send for PangoContext {}
unsafe impl Sync for PangoContext {}

#[derive(Component, Clone, Default)]
pub struct Canvas {
    pub max_dimension: f32,
    pub layout: layout::Layout,
}

#[derive(Component, Clone, Default)]
pub struct CanvasComputed {
    pub dimensions: Vec2,
}

#[derive(Bundle, Default)]
pub struct CanvasBundle {
    pub canvas: Canvas,
    pub canvas_computed: CanvasComputed,
    /// Describe the position of an entity. If the entity has a parent, the position is relative to its parent position.
    pub transform: Transform,
    /// Describe the position of an entity relative to the reference frame.
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
}

#[derive(Component, Clone, Default)]
pub struct RichText(pub interface::Text);

#[derive(Bundle, Default)]
pub struct RichTextBundle {
    pub rich_text: RichText,
    pub sprite: Sprite,
    pub texture: Handle<Image>,
    /// Describe the position of an entity. If the entity has a parent, the position is relative to its parent position.
    pub transform: Transform,
    /// Describe the position of an entity relative to the reference frame.
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub computed_visibility: ComputedVisibility,
}

pub const TEXT_FAMILY: &str = "Atkinson Hyperlegible";
pub const EMOJI_FAMILY: &str = "Twitter Color Emoji";

fn get_font_description(family: &str, size: Option<f32>) -> pango::FontDescription {
    let mut text_font = pango::FontDescription::new();
    text_font.set_family(family);
    if let Some(size) = size {
        text_font.set_size((size * pango::SCALE as f32) as i32);
    }
    text_font
}

/// System that runs once at startup to initialize font config
fn startup(mut commands: Commands) {
    use pango::prelude::*;

    // Initialize GTK (which initializes Pango as well)
    // Seems to be unnecessary
    // gtk::init().expect("Failed to initialize GTK.");

    ffi::font_config_init();

    // Add fonts from asset folder
    let paths = std::fs::read_dir("./assets/fonts").expect("Failed to read wraps load directory");
    for path in paths {
        let path = path.unwrap().path();
        match path.extension() {
            Some(ext) if ext == "ttf" => {
                if !ffi::font_config_add_file(path.clone()) {
                    println!("Error while attempting to load plugin {:?}", path);
                }
            }
            _ => {}
        }
    }

    // Generate single context
    let font_map = pangocairo::FontMap::for_font_type(cairo::FontType::FontTypeFt)
        .expect("Failed to create font map");
    let context: pango::Context = font_map.create_context();

    // Load fonts
    let text_font = get_font_description(TEXT_FAMILY, None);
    let emoji_font = get_font_description(EMOJI_FAMILY, None);
    context.set_font_description(Some(&text_font));
    context.load_font(&text_font);
    context.load_font(&emoji_font);

    // Add context to resources
    commands.insert_resource(PangoContext(Mutex::new(context)));
}

fn get_foreground(color: &interface::TextColor) -> pango::AttrColor {
    pango::AttrColor::new_foreground(
        ((color.r as f32 / 255.) * 65535 as f32) as u16,
        ((color.g as f32 / 255.) * 65535 as f32) as u16,
        ((color.b as f32 / 255.) * 65535 as f32) as u16,
    )
}

#[cfg(target_endian = "big")]
#[inline]
fn cairo_texture_chunk_to_wgpu(chunk: &[u8]) -> [u8; 4] {
    [chunk[1], chunk[2], chunk[3], chunk[0]]
}

#[cfg(target_endian = "little")]
#[inline]
fn cairo_texture_chunk_to_wgpu(chunk: &[u8]) -> [u8; 4] {
    [chunk[2], chunk[1], chunk[0], chunk[3]]
}

/// Whenever a richtext is changed, we need to do the following:
/// - Calculate minimum image size needed to render text
/// - Draw text to image
/// - Resize sprite
/// - Resize canvas
fn on_rich_text_change(
    mut rich_texts: Query<(&RichText, &mut Sprite, &Handle<Image>, &Parent), Changed<RichText>>,
    mut canvases: Query<(&Canvas, &mut CanvasComputed)>,
    mut context: ResMut<PangoContext>,
    mut images: ResMut<Assets<Image>>,
) {
    let context = context.lock().unwrap();

    for (
        RichText(interface::Text {
            spans,
            color: default_color,
            font_size,
            line_height,
        }),
        mut sprite,
        image_handle,
        parent,
    ) in rich_texts.iter_mut()
    {
        // Get available canvas width
        let (canvas, mut canvas_computed) = canvases
            .get_mut(parent.get())
            .expect("RichText parent must be a Canvas");
        let pango_max_dimension = canvas.max_dimension as i32 * pango::SCALE;

        // Setup fonts
        let font_size: Option<f32> = Some(*font_size);
        let emoji_font = get_font_description(EMOJI_FAMILY, font_size);

        // Build attributes
        let attrs = pango::AttrList::new();
        attrs.change(get_foreground(default_color));

        let mut text = String::new();
        let mut start_index = 0;
        for interface::TextSpan {
            text: text_span,
            color: override_color,
            style,
            weight,
        } in spans.into_iter()
        {
            text.push_str(text_span);
            let end_index = start_index + text_span.len() as u32;

            // Apply optional color override
            if let Some(color) = override_color {
                let mut attr = get_foreground(color);
                attr.set_start_index(start_index);
                attr.set_end_index(end_index);
                attrs.change(attr);
            }

            // Apply font styling
            let mut font = get_font_description(TEXT_FAMILY, font_size);
            font.set_weight(match weight {
                // See https://docs.gtk.org/Pango/enum.Weight.html
                100 => pango::Weight::Thin,
                200 => pango::Weight::Ultralight,
                300 => pango::Weight::Light,
                350 => pango::Weight::Semilight,
                380 => pango::Weight::Book,
                400 => pango::Weight::Normal,
                500 => pango::Weight::Medium,
                600 => pango::Weight::Semibold,
                700 => pango::Weight::Bold,
                800 => pango::Weight::Ultrabold,
                900 => pango::Weight::Heavy,
                1000 => pango::Weight::Ultraheavy,
                weight => pango::Weight::__Unknown(*weight as _),
            });
            font.set_style(match style {
                interface::Style::Normal => pango::Style::Normal,
                interface::Style::Italic => pango::Style::Italic,
                interface::Style::Oblique => pango::Style::Oblique,
            });
            let mut attr = pango::AttrFontDesc::new(&font);
            attr.set_start_index(start_index);
            attr.set_end_index(end_index);
            attrs.change(attr);

            // Apply emoji attributes
            unic::segment::GraphemeIndices::new(text_span)
                .into_iter()
                // Filter out non-emojis
                .filter(|(_, grapheme)| {
                    grapheme
                        .chars()
                        .any(|char| unic::emoji::char::is_emoji(char))
                })
                // Combine groups of emojis together if they are adjacent (i.e. no other characters in between)
                .fold::<Vec<(u32, u32)>, _>(vec![], |mut acc, (index, grapheme)| {
                    let start_index = start_index + index as u32;
                    let end_index = start_index + grapheme.len() as u32;

                    if acc
                        .last()
                        .map(|last| last.1 == start_index)
                        .unwrap_or(false)
                    {
                        acc.last_mut().unwrap().1 = end_index;
                    } else {
                        acc.push((start_index, end_index));
                    }

                    acc
                })
                .into_iter()
                .for_each(|(start_index, end_index)| {
                    let mut attr = pango::AttrFontDesc::new(&emoji_font);
                    attr.set_start_index(start_index);
                    attr.set_end_index(end_index);
                    attrs.change(attr);

                    // For some reason pango always falls back to the wrong font for emojis
                    let mut attr = pango::AttrInt::new_fallback(false);
                    attr.set_start_index(start_index);
                    attr.set_end_index(end_index);
                    attrs.change(attr);
                });

            start_index = end_index;
        }

        // Generate the text layout
        let layout = pango::Layout::new(&context);
        layout.set_spacing(((*line_height as f32 - 1.) * pango::SCALE as f32) as _);
        layout.set_attributes(Some(&attrs));
        layout.set_text(&text);
        match &canvas.layout {
            layout::Layout::FlexBasic(layout::FlexBasic { direction, .. }) => match direction {
                layout::Direction::Vertical => {
                    layout.set_width(pango_max_dimension);
                }
                layout::Direction::Horizontal => {
                    layout.set_height(pango_max_dimension);
                }
            },
        }

        // Get true size of the rendered text
        let (width, height) = layout.size();
        let width = width / pango::SCALE;
        let height = height / pango::SCALE;

        // TODO: handle device scale factor
        // windows: Query<&Window, With<PrimaryWindow>>,
        // let primary_window = windows.single();
        // let scale_factor = primary_window.scale_factor() as f32;

        // Draw the text
        let surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).unwrap();
        {
            let cx = cairo::Context::new(&surface).unwrap();

            // Draw white bg
            cx.set_source_rgba(1., 1., 1., 1.);
            cx.rectangle(0., 0., width as _, height as _);
            let _ = cx.fill();

            pangocairo::update_layout(&cx, &layout);
            pangocairo::show_layout(&cx, &layout);
        }
        let data = surface.take_data();

        // Copy the image data into a bevy image
        let image = images.get_mut(image_handle).unwrap();
        image.texture_descriptor.size = bevy::render::render_resource::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        };
        image.data = {
            // Convert from ARGB -> RGBA
            // TODO: cairo pre-multiplies alpha, so we should use a custom material and a mesh
            data.unwrap()
                .chunks_exact(4)
                .flat_map(cairo_texture_chunk_to_wgpu)
                .collect()
        };

        // Update sprite
        let new_size = Vec2::new(width as f32, height as f32);
        let old_size = sprite.custom_size.unwrap_or_default();
        sprite.custom_size.replace(new_size);

        // Depending on the layout, we need to update the canvas dimensions
        // Do this by removing the old size and adding the new size
        match &canvas.layout {
            layout::Layout::FlexBasic(layout::FlexBasic { direction, .. }) => match direction {
                layout::Direction::Vertical => {
                    canvas_computed.dimensions.y += new_size.y - old_size.y;
                }
                layout::Direction::Horizontal => {
                    canvas_computed.dimensions.x += new_size.x - old_size.x;
                }
            },
        }
    }
}

/// Whenever a canvas child changed, we need to do the following:
/// it updates the canvas dimensions
/// When that happens, we need to reflow/reposition all children
fn on_canvas_change(
    canvases: Query<(&Canvas, &Children), Changed<CanvasComputed>>,
    mut rich_texts: Query<(&mut Transform, &Sprite), With<RichText>>,
) {
    for (canvas, children) in canvases.iter() {
        // Start at origin
        let mut position = Vec3::splat(0.);
        for child in children.iter() {
            let (mut transform, sprite) = rich_texts
                .get_mut(*child)
                .expect("Canvas child must be a RichText");

            transform.translation = position;

            match &canvas.layout {
                layout::Layout::FlexBasic(layout::FlexBasic { direction, gap }) => {
                    let custom_size = sprite.custom_size.unwrap_or_default();
                    match direction {
                        layout::Direction::Vertical => {
                            position.y -= custom_size.y + gap;
                        }
                        layout::Direction::Horizontal => {
                            position.x += custom_size.x + gap;
                        }
                    }
                }
            }
        }
    }
}

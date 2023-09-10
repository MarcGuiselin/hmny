use bevy::prelude::*;

mod ffi;

#[derive(Default)]
pub struct PangoPlugin;

impl Plugin for PangoPlugin {
    fn build(&self, app: &mut App) {
        ffi::font_config_init();

        // Add fonts from asset folder
        let paths =
            std::fs::read_dir("./assets/fonts").expect("Failed to read wraps load directory");
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

        app.add_systems(Startup, setup);
        // let dir_path = PathBuf::from(FONTS_LOAD_DIR);
        //
        // let mut font_system = cosmic_text::FontSystem::new();
        // font_system.db_mut().load_fonts_dir(dir_path);
        //
        // app.add_systems(PreUpdate, cosmic_editor_builder)
        //     .add_systems(
        //         Update,
        //         (
        //             cosmic_edit_bevy_events,
        //             cosmic_edit_set_redraw,
        //             on_scale_factor_change,
        //             cosmic_edit_redraw_buffer_ui
        //                 .before(cosmic_edit_set_redraw)
        //                 .before(on_scale_factor_change),
        //             cosmic_edit_redraw_buffer.before(on_scale_factor_change),
        //         ),
        //     )
        //     .init_resource::<ActiveEditor>()
        //     // .add_asset::<CosmicFont>()
        //     .insert_resource(SwashCacheState {
        //         swash_cache: SwashCache::new(),
        //     })
        //     .insert_resource(CosmicFontSystem(font_system));
    }
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

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    use pango::prelude::*;

    // Initialize GTK (which initializes Pango as well)
    // Seems to be unnecessary
    // gtk::init().expect("Failed to initialize GTK.");

    let width = 500;
    let mut text_font = pango::FontDescription::new();
    text_font.set_family("Atkinson Hyperlegible");
    text_font.set_size(32 * pango::SCALE);
    let mut emoji_font = pango::FontDescription::new();
    emoji_font.set_family("Twitter Color Emoji");
    emoji_font.set_size(32 * pango::SCALE);

    let font_map = pangocairo::FontMap::for_font_type(cairo::FontType::FontTypeFt)
        .expect("Failed to create font map");
    let context = font_map.create_context();

    // Load fonts
    context.set_font_description(Some(&text_font));
    context.load_font(&text_font);
    context.load_font(&emoji_font);

    let text = "👾a🤖b🎃🧜🏾👩‍👩‍👧‍👦hello world";
    let attrs = pango::AttrList::new();

    // Set text color to red
    attrs.change(pango::AttrColor::new_foreground(65535, 0, 0));

    // Apply emoji attributes
    unic::segment::GraphemeIndices::new(text)
        .into_iter()
        // Filter out non-emojis
        .filter(|(_, grapheme)| {
            grapheme
                .chars()
                .any(|char| unic::emoji::char::is_emoji(char))
        })
        // Combine groups of emojis together if they are adjacent (i.e. no other characters in between)
        .fold::<Vec<(u32, u32)>, _>(vec![], |mut acc, (start_index, grapheme)| {
            let start_index = start_index as u32;
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

    // Generate the text layout
    let layout = pango::Layout::new(&context);
    layout.set_width(width * pango::SCALE);
    layout.set_attributes(Some(&attrs));
    layout.set_text(text);

    // Get true size of the rendered text
    let (width, height) = layout.size();
    let width = width / pango::SCALE;
    let height = height / pango::SCALE;

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
    let texture = images.add(Image::new(
        bevy::render::render_resource::Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        // Convert from ARGB -> RGBA
        // TODO: cairo pre-multiplies alpha, so we should use a custom material and a mesh
        data.unwrap()
            .chunks_exact(4)
            .flat_map(cairo_texture_chunk_to_wgpu)
            .collect(),
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
    ));

    // Add the sprite to the scene
    commands.spawn(SpriteBundle {
        texture,
        sprite: Sprite {
            custom_size: Some(Vec2::new(width as f32, height as f32)),
            anchor: bevy::sprite::Anchor::TopLeft,
            ..Default::default()
        },
        ..Default::default()
    });
}

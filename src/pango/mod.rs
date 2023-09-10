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

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    use pango::prelude::*;

    // Initialize GTK (which initializes Pango as well)
    // Seems to be unnecessary
    // gtk::init().expect("Failed to initialize GTK.");

    let width = 500;
    let mut font = pango::FontDescription::new();
    font.set_family("Atkinson Hyperlegible");
    font.set_size(32 * pango::SCALE);

    let font_map = pangocairo::FontMap::for_font_type(cairo::FontType::FontTypeFt)
        .expect("Failed to create font map");
    let context = font_map.create_context();

    // Load fonts
    context.set_font_description(Some(&font));
    context.load_font(&font);

    let text = "hello world";
    let attrs = pango::AttrList::new();

    // Set text color to red
    attrs.change(pango::AttrColor::new_foreground(65535, 0, 0));

    // Generate the text layout
    let layout = pango::Layout::new(&context);
    layout.set_width(width);
    layout.set_font_description(Some(&font));
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
        data.unwrap().to_vec(),
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

use bevy::prelude::*;

pub struct WebViewPlugin;

impl Plugin for WebViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup<'a>(app: &'a mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    start_server()?;

    // This one
    let handle = app.handle();

    tauri::async_runtime::spawn(async move {
        // also added move here
        let verify_result = verify_local_server().await;
        match verify_result {
            Ok(_) => {
                println!("Local Server is running");
            }
            Err(err) => {
                handle.emit_all("local-server-down", ()); // changed this to handle.
                println!("Local Server is not running");
                println!("{}", err);
            }
        }
    });
    Ok(())
}

fn setup(mut wraps: ResMut<Wraps>, mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    match wraps.signal(WrapKey::HomeScreen, HomescreenQuery::AskHomeScreen) {
        Ok(HomescreenResponse::HomeScreen { mime_type, data }) => {
            info!(
                r#"Load home screen with mimetype: "{}" data: "{:?}""#,
                mime_type, data
            );

            match wraps.signal(
                WrapKey::Mimetype(mime_type),
                MimetypeQuery::AskParse { data },
            ) {
                Ok(MimetypeResponse::Dimension(dimension)) => {
                    info!(r#"Loading dimension: "{:?}""#, dimension);
                    let dimension_entity = commands.spawn(SpatialBundle::default()).id();
                    for element in dimension.children.into_iter() {
                        summon_element(element, dimension_entity, &mut commands, &mut images);
                    }
                }
                other => {
                    error!("Could not load dimension: {:?}", other);
                }
            }
        }
        other => {
            error!("Could not load home screen data: {:?}", other);
        }
    }
}

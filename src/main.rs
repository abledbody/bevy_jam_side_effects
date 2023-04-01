use bevy::prelude::*;

const TITLE: &'static str = "My Title";
const CLEAR_COLOR: Color = Color::AQUAMARINE;

mod player;
mod math;

fn main() {
    // Hot reload assets
    #[cfg(feature = "debug_mode")]
    let watch_for_changes = true;
    #[cfg(not(feature = "debug_mode"))]
    let watch_for_changes = false;

    let mut app = App::new();

    // Resources
    app.insert_resource(ClearColor(CLEAR_COLOR));

    // Plugins
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: TITLE.to_string(),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                watch_for_changes,
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    );

    // Startup systems
    app.add_startup_system(spawn_camera);

    // UI systems
    app.add_system(bevy::window::close_on_esc)
		.add_system(player::Player::player_movement);

    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

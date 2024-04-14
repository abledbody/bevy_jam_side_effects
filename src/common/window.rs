use bevy::prelude::*;
use bevy::window::WindowPlugin as BevyWindowPlugin;

pub struct WindowPlugin;

impl Plugin for WindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BevyWindowPlugin {
            primary_window: Some(Window {
                title: TITLE.to_string(),
                canvas: Some("#bevy".to_string()),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        });

        #[cfg(not(feature = "web"))]
        app.add_systems(Update, bevy::window::close_on_esc);
    }
}

const TITLE: &str = "Sai Defects";

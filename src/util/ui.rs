pub mod alarm_meter;
mod backdrop;
pub mod font;
pub mod health_bar;
pub mod nametag;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            alarm_meter::AlarmMeterPlugin,
            font::FontPlugin,
            health_bar::HealthBarPlugin,
        ));
    }
}

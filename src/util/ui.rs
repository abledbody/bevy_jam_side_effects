pub mod alarm_meter;
mod backdrop;
pub mod font_size_hack;
pub mod health_bar;
pub mod nametag;

use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            alarm_meter::AlarmMeterPlugin,
            font_size_hack::FontSizeHackPlugin,
            health_bar::HealthBarPlugin,
        ));
    }
}

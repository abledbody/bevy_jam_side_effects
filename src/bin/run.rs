// Disable console on windows for release builds
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

use bevy::prelude::*;
use sai_defects::AppPlugin;

fn main() {
    App::new().add_plugins(AppPlugin).run();
}

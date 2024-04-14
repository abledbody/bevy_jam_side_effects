// Disable common false-positive clippy warnings
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

mod animation;
mod asset;
mod camera;
mod combat;
mod cutscene;
mod debug;
mod game;
mod hud;
mod map;
mod math;
mod mob;
mod music;
mod util;
mod vfx;

use bevy::prelude::*;

use crate::game::GamePlugin;

fn main() {
    App::new().add_plugins(GamePlugin).run();
}

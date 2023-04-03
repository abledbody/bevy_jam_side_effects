use bevy::prelude::*;

use crate::game::GamePlugin;

mod animation;
mod asset;
mod camera;
mod combat;
mod debug;
mod game;
mod map;
mod math;
mod mob;
mod util;
mod vfx;

fn main() {
    App::new().add_plugin(GamePlugin).run();
}

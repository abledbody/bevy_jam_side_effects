use bevy::prelude::*;

use crate::game::GamePlugin;

mod animation;
mod asset;
mod combat;
mod debug;
mod game;
mod math;
mod mob;
mod vfx;

fn main() {
    App::new().add_plugin(GamePlugin).run();
}

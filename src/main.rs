use bevy::prelude::*;

use crate::game::GamePlugin;

mod animation;
mod asset;
mod debug;
mod game;
mod math;
mod mob;

fn main() {
    App::new().add_plugin(GamePlugin).run();
}

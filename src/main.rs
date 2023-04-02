use bevy::prelude::*;

use crate::game::GamePlugin;

mod animation;
mod asset;
mod combat;
mod debug;
mod game;
mod math;
mod mob;

fn main() {
    App::new().add_plugin(GamePlugin).run();
}

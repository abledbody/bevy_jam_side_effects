use crate::game::GamePlugin;
use bevy::prelude::*;

mod asset;
mod debug;
mod game;
mod math;
mod mob;
mod player;

fn main() {
    App::new().add_plugin(GamePlugin).run();
}

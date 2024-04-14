use bevy::prelude::*;

use crate::game::GamePlugin;

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

fn main() {
    App::new().add_plugins(GamePlugin).run();
}

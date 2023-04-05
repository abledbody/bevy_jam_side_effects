use bevy::prelude::*;

use crate::game::GamePlugin;

mod animation;
mod asset;
mod camera;
mod combat;
mod debug;
mod game;
mod hud;
mod map;
mod math;
mod mob;
mod util;
mod vfx;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn main() {
    App::new().add_plugin(GamePlugin).run();
}

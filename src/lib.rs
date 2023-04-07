use bevy::prelude::*;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

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

#[cfg(feature = "wasm")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn run() {
    App::new().add_plugin(GamePlugin).run();
}

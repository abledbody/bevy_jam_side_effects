//! Self-contained, re-usable utilities that are not specific to this game

#![allow(dead_code)]
#![allow(unused_imports)]

pub use crate::util::despawn::DespawnSet;

pub mod animation;
mod despawn;
pub mod math;
pub mod ui;
pub mod vfx;
pub mod y_sort;

use bevy::prelude::*;

pub struct UtilPlugin;

impl Plugin for UtilPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            animation::AnimationPlugin,
            despawn::DespawnPlugin,
            ui::UiPlugin,
            vfx::VfxPlugin,
            y_sort::YSortPlugin,
        ));
    }
}

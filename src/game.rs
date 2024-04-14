use bevy::prelude::*;

pub mod combat;
pub mod cutscene;
pub mod map;
pub mod mob;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, _app: &mut App) {}
}

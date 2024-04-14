use bevy::prelude::*;

pub mod combat;
pub mod cutscene;
pub mod map;
pub mod mob;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            combat::CombatPlugin,
            cutscene::CutscenePlugin,
            map::MapPlugin,
            mob::MobPlugin,
        ));
    }
}

use bevy::prelude::*;

pub mod actor;
pub mod combat;
pub mod cutscene;
pub mod map;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            combat::CombatPlugin,
            cutscene::CutscenePlugin,
            map::MapPlugin,
            actor::ActorPlugin,
        ));
    }
}

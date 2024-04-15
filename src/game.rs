use bevy::prelude::*;

pub mod actor;
pub mod alarm;
pub mod combat;
pub mod cutscene;
pub mod level;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            actor::ActorPlugin,
            alarm::AlarmPlugin,
            combat::CombatPlugin,
            cutscene::CutscenePlugin,
            level::LevelPlugin,
        ));
    }
}

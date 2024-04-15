mod boot;
pub mod game;

use bevy::prelude::*;
use strum::EnumIter;

pub struct SequencePlugin;

impl Plugin for SequencePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SequenceState>()
            .add_plugins((boot::BootStatePlugin, game::GameStatePlugin));
    }
}

#[derive(States, Reflect, Default, Copy, Clone, Eq, PartialEq, Hash, Debug, EnumIter)]
pub enum SequenceState {
    #[default]
    Boot,
    // TODO: Workaround for https://github.com/bevyengine/bevy/issues/9130
    RestartGame,
    Game,
}

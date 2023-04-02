use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::common_conditions::input_just_pressed,
    prelude::*,
};
#[cfg(feature = "editor")]
use bevy_editor_pls::EditorPlugin;
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{Facing, Offset, WalkAnimation},
    asset::Handles,
    combat::Effects,
    mob::{
        enemy::{EnemyAi, Loot},
        player::{Gold, PlayerControl},
        Health,
        Mob,
        MobInputs,
    },
};

const TOGGLE_KEY: KeyCode = KeyCode::F3;

#[derive(Default)]
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        // Hot-reload assets
        app.world
            .resource::<AssetServer>()
            .asset_io()
            .watch_for_changes()
            .unwrap();

        // Plugins
        app.add_plugin(RapierDebugRenderPlugin::default())
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(LogDiagnosticsPlugin::default());

        #[cfg(feature = "editor")]
        app.add_plugin(EditorPlugin::default());

        // Systems
        app.add_system(DebugPlugin::toggle.run_if(input_just_pressed(TOGGLE_KEY)));

        // Types
        app.register_type::<Handles>()
            .register_type::<Health>()
            .register_type::<Gold>()
            .register_type::<Loot>()
            .register_type::<Mob>()
            .register_type::<MobInputs>()
            .register_type::<PlayerControl>()
            .register_type::<EnemyAi>()
            .register_type::<Facing>()
            .register_type::<Offset>()
            .register_type::<WalkAnimation>()
            .register_type::<Effects>();

        // Disable Rapier debug initially
        app.world.resource_mut::<DebugRenderContext>().enabled = false;
    }
}

impl DebugPlugin {
    fn toggle(mut debug_render_context: ResMut<DebugRenderContext>) {
        debug_render_context.enabled = !debug_render_context.enabled;
    }
}

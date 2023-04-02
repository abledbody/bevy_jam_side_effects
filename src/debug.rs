use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::input::common_conditions::input_just_pressed;
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;

use crate::asset::Handles;
use crate::mob::Health;
use crate::mob::Mob;
use crate::mob::MobInputs;
use crate::player::Player;

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
            .add_plugin(WorldInspectorPlugin::new().run_if(input_toggle_active(false, TOGGLE_KEY)))
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(LogDiagnosticsPlugin::default());

        // Systems
        app.add_system(DebugPlugin::toggle.run_if(input_just_pressed(TOGGLE_KEY)));

        // Types
        app.register_type::<Handles>()
            .register_type::<Health>()
            .register_type::<Mob>()
            .register_type::<MobInputs>()
            .register_type::<Player>();

        // Disable Rapier debug initially
        app.world.resource_mut::<DebugRenderContext>().enabled = false;
    }
}

impl DebugPlugin {
    fn toggle(mut debug_render_context: ResMut<DebugRenderContext>) {
        debug_render_context.enabled = !debug_render_context.enabled;
    }
}

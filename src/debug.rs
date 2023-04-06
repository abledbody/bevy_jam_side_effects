use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    input::common_conditions::input_just_pressed,
    prelude::*,
};
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{
        AttackAnimation,
        DeathAnimation,
        Facing,
        FlinchAnimation,
        Lifetime,
        Offset,
        VirtualParent,
        WalkAnimation,
    },
    asset::Handles,
    camera::GameCamera,
    combat::{DeathEffects, HitEffects, HurtEffects},
    hud::{AlarmMeter, HealthBar},
    map::{Exit, Gate, Plate, Wall},
    mob::{
        enemy::{Alarm, DifficultyCurve, EnemyAi},
        player::{Gold, PlayerControl},
        DeadBody,
        Health,
        Mob,
        MobInputs,
    },
    util::{DespawnSet, ZRampByY},
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
        app.add_plugin(bevy_editor_pls::EditorPlugin::default());

        // Systems
        app.add_system(DebugPlugin::toggle.run_if(input_just_pressed(TOGGLE_KEY)));

        // Types
        app.register_type::<Handles>()
            .register_type::<Health>()
            .register_type::<Gold>()
            .register_type::<Mob>()
            .register_type::<MobInputs>()
            .register_type::<PlayerControl>()
            .register_type::<DifficultyCurve>()
            .register_type::<EnemyAi>()
            .register_type::<DeadBody>()
            .register_type::<HitEffects>()
            .register_type::<HurtEffects>()
            .register_type::<DeathEffects>()
            .register_type::<VirtualParent>()
            .register_type::<ZRampByY>()
            .register_type::<DespawnSet>()
            .register_type::<Offset>()
            .register_type::<WalkAnimation>()
            .register_type::<AttackAnimation>()
            .register_type::<FlinchAnimation>()
            .register_type::<DeathAnimation>()
            .register_type::<Facing>()
            .register_type::<Lifetime>()
            .register_type::<Wall>()
            .register_type::<Exit>()
            .register_type::<Plate>()
            .register_type::<Gate>()
            .register_type::<GameCamera>()
            .register_type::<HealthBar>()
            .register_type::<Alarm>()
            .register_type::<AlarmMeter>();

        // Disable Rapier debug initially
        app.world.resource_mut::<DebugRenderContext>().enabled = false;
    }
}

impl DebugPlugin {
    fn toggle(mut debug_render_context: ResMut<DebugRenderContext>) {
        debug_render_context.enabled = !debug_render_context.enabled;
    }
}

use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::common::asset::Handles;
use crate::common::camera::GameCamera;
use crate::common::music::Music;
use crate::game::combat::DeathEffects;
use crate::game::combat::HitEffects;
use crate::game::combat::HurtEffects;
use crate::game::cutscene::Cutscene;
use crate::game::cutscene::Message;
use crate::game::map::Exit;
use crate::game::map::Gate;
use crate::game::map::Plate;
use crate::game::map::Wall;
use crate::game::mob::animation::AttackAnimation;
use crate::game::mob::animation::DeathAnimation;
use crate::game::mob::animation::FlinchAnimation;
use crate::game::mob::animation::WalkAnimation;
use crate::game::mob::enemy::Alarm;
use crate::game::mob::enemy::DifficultyCurve;
use crate::game::mob::enemy::EnemyAi;
use crate::game::mob::player::PlayerControl;
use crate::game::mob::player::Playthrough;
use crate::game::mob::Body;
use crate::game::mob::Health;
use crate::game::mob::Mob;
use crate::game::mob::MobInputs;
use crate::util::animation::facing::Facing;
use crate::util::animation::follow::Follow;
use crate::util::animation::lifetime::Lifetime;
use crate::util::animation::offset::Offset;
use crate::util::ui::alarm_meter::AlarmMeter;
use crate::util::ui::font_size_hack::FontSizeHack;
use crate::util::ui::health_bar::HealthBar;
use crate::util::y_sort::YSort;
use crate::util::DespawnSet;

const TOGGLE_KEY: KeyCode = KeyCode::F3;

#[derive(Default)]
pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        // Plugins
        app.add_plugins((
            RapierDebugRenderPlugin::default(),
            FrameTimeDiagnosticsPlugin,
            LogDiagnosticsPlugin::default(),
        ));
        app.add_plugins(bevy_editor_pls::EditorPlugin::new().in_new_window(Window {
            title: "bevy_editor_pls".to_string(),
            focused: false,
            ..default()
        }));

        // Systems
        app.add_systems(
            Update,
            DebugPlugin::toggle.run_if(input_just_pressed(TOGGLE_KEY)),
        );

        // Types
        app.register_type::<Handles>()
            .register_type::<Health>()
            .register_type::<Mob>()
            .register_type::<MobInputs>()
            .register_type::<Body>()
            .register_type::<PlayerControl>()
            .register_type::<Playthrough>()
            .register_type::<DifficultyCurve>()
            .register_type::<EnemyAi>()
            .register_type::<HitEffects>()
            .register_type::<HurtEffects>()
            .register_type::<DeathEffects>()
            .register_type::<Follow>()
            .register_type::<YSort>()
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
            .register_type::<Music>()
            .register_type::<GameCamera>()
            .register_type::<Cutscene>()
            .register_type::<Message>()
            .register_type::<FontSizeHack>()
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

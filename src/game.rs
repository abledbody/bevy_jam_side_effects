use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::common_conditions::action_just_pressed;
use leafwing_input_manager::prelude::*;

use crate::common::asset::AudioKey;
use crate::common::asset::Handles;
use crate::game::actor::enemy::AlertEvent;
use crate::game::actor::player::Playthrough;
use crate::game::alarm::Alarm;
use crate::game::alarm::AlarmMeter;
use crate::game::alarm::AlarmMeterTemplate;
use crate::game::combat::DeathEvent;
use crate::game::combat::HitEvent;
use crate::game::cutscene::CutsceneTemplate;
use crate::game::cutscene::Message;
use crate::game::map::MapTemplate;
use crate::game::map::Victory;
use crate::util::ui::UiRoot;
use crate::util::DespawnSet;

pub mod actor;
pub mod alarm;
mod combat;
mod cutscene;
pub mod map;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_game);

        app.init_resource::<ActionState<GameAction>>()
            .insert_resource(
                InputMap::default()
                    .insert(GameAction::Restart, KeyCode::KeyR)
                    .insert(GameAction::Confirm, KeyCode::Space)
                    .insert(GameAction::Confirm, KeyCode::Enter)
                    .insert(GameAction::Confirm, MouseButton::Left)
                    .build(),
            )
            .add_plugins(InputManagerPlugin::<GameAction>::default())
            .add_systems(
                First,
                restart_game.run_if(action_just_pressed(GameAction::Restart)),
            );

        app.add_plugins((
            actor::ActorPlugin,
            alarm::AlarmPlugin,
            combat::CombatPlugin,
            cutscene::CutscenePlugin,
            map::MapPlugin,
        ));
    }
}

// TODO: This should be handled in the respective plugins on game state exit / enter
fn spawn_game(mut commands: Commands, ui_root: Res<UiRoot>, handle: Res<Handles>) {
    // Spawn map
    MapTemplate.spawn(&mut commands, &handle);

    // Spawn HUD
    let alarm_meter = AlarmMeterTemplate.spawn(&mut commands, &handle);
    commands.entity(alarm_meter).set_parent(ui_root.body);

    let cutscene = CutsceneTemplate.spawn(&mut commands, &handle);
    commands.entity(cutscene).set_parent(ui_root.body);
}

#[derive(Actionlike, Reflect, Clone, Hash, PartialEq, Eq)]
enum GameAction {
    Restart,
    Confirm,
}

// TODO: This should be handled in the respective plugins on game state exit / enter
fn restart_game(
    mut commands: Commands,
    mut despawn: ResMut<DespawnSet>,
    handle: Res<Handles>,
    entity_query: Query<
        Entity,
        (
            Or<(With<Handle<LdtkProject>>, With<Message>)>,
            Without<Parent>,
        ),
    >,
    mut alarm_meter_query: Query<&mut AlarmMeter>,
    mut collision_events: ResMut<Events<CollisionEvent>>,
    mut hit_events: ResMut<Events<HitEvent>>,
    mut death_events: ResMut<Events<DeathEvent>>,
    mut detect_events: ResMut<Events<AlertEvent>>,
    mut level_selection: ResMut<LevelSelection>,
    mut playthrough: ResMut<Playthrough>,
    mut victory: ResMut<Victory>,
    mut alarm: ResMut<Alarm>,
    audio: Res<Audio>,
) {
    // Despawn entities
    for entity in &entity_query {
        despawn.recursive(entity);
    }

    // Respawn map
    MapTemplate.spawn(&mut commands, &handle);

    // Reset alarm meter shake
    for mut alarm_meter in &mut alarm_meter_query {
        alarm_meter.old_alarm = 0.0;
        alarm_meter.shake = 0.0;
    }

    // Reset events
    collision_events.clear();
    hit_events.clear();
    death_events.clear();
    detect_events.clear();

    // Reset resources
    *level_selection = default();
    *playthrough = default();
    *victory = default();
    *alarm = default();

    // Play restart sound
    audio.play(handle.audio[&AudioKey::Pop1].clone());
}

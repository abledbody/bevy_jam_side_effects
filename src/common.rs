pub mod asset;
pub mod camera;
mod debug;
mod music;

use bevy::prelude::*;
use bevy::transform::TransformSystem;
use bevy::ui::UiSystem;
use bevy_ecs_ldtk::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::common_conditions::action_just_pressed;
use leafwing_input_manager::prelude::*;

use crate::common::asset::AudioKey;
use crate::common::asset::Handles;
use crate::common::camera::GameCameraTemplate;
use crate::game::actor::enemy::Alarm;
use crate::game::actor::enemy::AlertEvent;
use crate::game::actor::player::Playthrough;
use crate::game::combat::DeathEvent;
use crate::game::combat::HitEvent;
use crate::game::cutscene::CutsceneTemplate;
use crate::game::cutscene::Message;
use crate::game::map::MapTemplate;
use crate::game::map::Victory;
use crate::util::ui::alarm_meter::AlarmMeter;
use crate::util::ui::alarm_meter::AlarmMeterTemplate;

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        // Resources
        app.insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..default()
        })
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            level_background: LevelBackground::Nonexistent,
            ..default()
        })
        .init_resource::<LevelSelection>()
        .init_resource::<ActionState<GameAction>>()
        .insert_resource(
            InputMap::default()
                .insert(KeyCode::R, GameAction::Restart)
                .insert(KeyCode::Space, GameAction::Confirm)
                .insert(KeyCode::Return, GameAction::Confirm)
                .insert(MouseButton::Left, GameAction::Confirm)
                .build(),
        );

        // Game logic system ordering
        app.configure_sets(
            Update,
            (
                UpdateSet::HandleActions,
                UpdateSet::HandleActionsFlush,
                UpdateSet::Start,
                UpdateSet::Update,
                UpdateSet::RecordIntents,
                UpdateSet::ApplyIntents,
                UpdateSet::HandleEvents,
                UpdateSet::QueueDespawn,
                UpdateSet::ApplyDeferred,
                UpdateSet::UpdateUi,
                UpdateSet::End,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                apply_deferred.in_set(UpdateSet::HandleActionsFlush),
                apply_deferred.in_set(UpdateSet::ApplyDeferred),
            ),
        );

        // Post-processing system ordering
        app.configure_sets(
            PostUpdate,
            (
                (UiSystem::Layout, PhysicsSet::Writeback),
                PostTransformSet::Save,
                PostTransformSet::Blend,
                PostTransformSet::ApplyFacing,
                TransformSystem::TransformPropagate,
                PostTransformSet::Finish,
                // GlobalTransform may be slightly out of sync with Transform at this point...
            )
                .chain(),
        );

        // Plugins
        app.add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: TITLE.to_string(),
                        canvas: Some("#bevy".to_string()),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            AudioPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            InputManagerPlugin::<GameAction>::default(),
            LdtkPlugin,
        ));
        app.add_plugins((asset::AssetPlugin, camera::CameraPlugin, music::MusicPlugin));
        #[cfg(feature = "dev")]
        app.add_plugins(crate::common::debug::DebugPlugin);

        // Startup systems
        app.add_systems(Startup, spawn_game);

        // First systems
        app.add_systems(
            First,
            restart_game.run_if(action_just_pressed(GameAction::Restart)),
        );

        // UI systems
        #[cfg(not(feature = "web"))]
        app.add_systems(Update, bevy::window::close_on_esc);
    }
}

const TITLE: &str = "Sai Defects";

/// (Update) Game logic system ordering
#[derive(SystemSet, Clone, Eq, PartialEq, Hash, Debug)]
pub enum UpdateSet {
    /// Handle actions pressed this frame
    HandleActions,
    /// Apply deferred effects from HandleActions
    HandleActionsFlush,
    /// Initialize start-of-frame values and tick timers
    Start,
    /// Step game logic
    Update,
    /// Record player and AI intents
    RecordIntents,
    /// Apply player and AI intents
    ApplyIntents,
    /// Handle events emitted this frame
    HandleEvents,
    /// Queue despawn commands from DespawnSet
    QueueDespawn,
    /// Apply spawn / despawn and other commands
    ApplyDeferred,
    /// Update UI
    UpdateUi,
    /// Synchronize end-of-frame values
    End,
}

/// (PostUpdate) Transform post-processing system ordering
#[derive(SystemSet, Clone, Eq, PartialEq, Hash, Debug)]
pub enum PostTransformSet {
    /// Save the base transform as a backup
    Save,
    /// Blend via transform multplication (add translation, add rotation, multiply scale)
    Blend,
    /// Apply facing (may multiply translation.x by -1)
    ApplyFacing,
    /// Apply finishing touches to GlobalTransform, like rounding to the nearest pixel
    Finish,
}

#[derive(Actionlike, Reflect, Clone, Hash, PartialEq, Eq)]
pub enum GameAction {
    Restart,
    Confirm,
}

fn restart_game(
    mut commands: Commands,
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
        commands.entity(entity).despawn_recursive();
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

fn spawn_game(mut commands: Commands, handle: Res<Handles>) {
    // Spawn map
    MapTemplate.spawn(&mut commands, &handle);

    // Spawn HUD
    AlarmMeterTemplate.spawn(&mut commands, &handle);
    CutsceneTemplate.spawn(&mut commands, &handle);

    // Spawn camera
    GameCameraTemplate.spawn(&mut commands);
}

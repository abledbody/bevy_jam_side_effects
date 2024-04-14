pub mod asset;
pub mod camera;
mod debug;
mod music;

use bevy::prelude::*;
use bevy::transform::systems::propagate_transforms;
use bevy::transform::systems::sync_simple_transforms;
use bevy_ecs_ldtk::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::common_conditions::action_just_pressed;
use leafwing_input_manager::prelude::*;

use crate::common::asset::AudioKey;
use crate::common::asset::Handles;
use crate::common::camera::GameCamera;
use crate::common::camera::GameCameraTemplate;
use crate::common::music::Music;
use crate::game::combat::DeathEffects;
use crate::game::combat::DeathEvent;
use crate::game::combat::HitEffects;
use crate::game::combat::HitEvent;
use crate::game::combat::HurtEffects;
use crate::game::cutscene::Cutscene;
use crate::game::cutscene::CutsceneTemplate;
use crate::game::cutscene::Message;
use crate::game::map::spawn_level_entities;
use crate::game::map::Exit;
use crate::game::map::MapTemplate;
use crate::game::map::Plate;
use crate::game::map::Victory;
use crate::game::map::VictorySquare;
use crate::game::mob::animation::AttackAnimation;
use crate::game::mob::animation::DeathAnimation;
use crate::game::mob::animation::FlinchAnimation;
use crate::game::mob::animation::WalkAnimation;
use crate::game::mob::enemy::Alarm;
use crate::game::mob::enemy::DetectEvent;
use crate::game::mob::enemy::DifficultyCurve;
use crate::game::mob::enemy::EnemyAi;
use crate::game::mob::player::PlayerAction;
use crate::game::mob::player::PlayerControl;
use crate::game::mob::player::Playthrough;
use crate::game::mob::Mob;
use crate::util::animation::facing::Facing;
use crate::util::animation::follow::Follow;
use crate::util::animation::lifetime::Lifetime;
use crate::util::animation::offset::Offset;
use crate::util::ui::alarm_meter::AlarmMeter;
use crate::util::ui::alarm_meter::AlarmMeterTemplate;
use crate::util::ui::font_size_hack::FontSizeHack;
use crate::util::ui::health_bar::HealthBar;
use crate::util::y_sort::YSort;
use crate::util::DespawnSet;

const TITLE: &str = "Sai Defects";

#[derive(Actionlike, Reflect, Clone, Hash, PartialEq, Eq)]
pub enum GameAction {
    Restart,
    Confirm,
}

#[derive(SystemSet, Clone, Debug, Eq, PartialEq, Hash)]
pub enum UpdateSet {
    // <-- Spawn level
    Synchronize,
    Animate,
    PostAnimate,
    Combat,
    CombatFlush,
    QueueDespawn,
    // <-- Physics
}

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
        )
        .init_resource::<Handles>()
        .init_resource::<DespawnSet>()
        .init_resource::<Music>()
        .init_resource::<Playthrough>()
        .init_resource::<Victory>()
        .init_resource::<Alarm>();

        // Events
        app.add_event::<HitEvent>()
            .add_event::<DeathEvent>()
            .add_event::<DetectEvent>();

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
            InputManagerPlugin::<PlayerAction>::default(),
            LdtkPlugin,
        ));
        #[cfg(feature = "dev")]
        app.add_plugins(crate::common::debug::DebugPlugin);

        // Startup systems
        app.add_systems(Startup, (Handles::load, spawn_game).chain());

        // First systems
        app.add_systems(
            First,
            (
                restart_game.run_if(action_just_pressed(GameAction::Restart)),
                Cutscene::advance.run_if(action_just_pressed(GameAction::Confirm)),
            ),
        );

        // Pre-update systems
        app.add_systems(PreUpdate, (Exit::detect, spawn_level_entities).chain());

        // Game logic system sets
        app.configure_sets(
            Update,
            (
                UpdateSet::Synchronize,
                UpdateSet::Animate,
                UpdateSet::PostAnimate,
                UpdateSet::Combat,
                UpdateSet::CombatFlush,
            )
                .chain(),
        );

        // Synchronization systems
        app.add_systems(
            Update,
            (
                (
                    sync_simple_transforms,
                    propagate_transforms,
                    YSort::apply,
                    Follow::apply,
                    Offset::apply,
                )
                    .chain(),
                Plate::detect,
                VictorySquare::detect,
                (
                    (
                        HitEvent::detect,
                        DetectEvent::detect,
                        DifficultyCurve::apply,
                    ),
                    EnemyAi::think,
                )
                    .chain(),
                PlayerControl::record_inputs,
            )
                .in_set(UpdateSet::Synchronize),
        );

        // Animation systems
        app.add_systems(
            Update,
            (
                Playthrough::detect_defection,
                GameCamera::cut_to_new_target,
                GameCamera::follow_target,
                FontSizeHack::scale,
                (
                    WalkAnimation::trigger,
                    WalkAnimation::play_step_sound,
                    WalkAnimation::update,
                    WalkAnimation::apply,
                )
                    .chain(),
                (DeathAnimation::update, DeathAnimation::apply).chain(),
                (
                    AttackAnimation::trigger,
                    AttackAnimation::update,
                    AttackAnimation::apply,
                )
                    .chain(),
                (FlinchAnimation::update, FlinchAnimation::apply).chain(),
            )
                .in_set(UpdateSet::Animate),
        );

        // Post-animation systems
        app.add_systems(
            Update,
            (Mob::set_facing, Facing::apply)
                .chain()
                .in_set(UpdateSet::PostAnimate),
        );

        // Combat systems
        app.add_systems(
            Update,
            (
                Mob::apply_movement,
                HurtEffects::apply,
                (
                    HitEffects::apply,
                    DeathEffects::apply,
                    HitEffects::cleanup,
                    HitEffects::spawn_from_inputs,
                )
                    .chain(),
                Lifetime::apply,
            )
                .in_set(UpdateSet::Combat),
        );
        app.add_systems(Update, apply_deferred.in_set(UpdateSet::CombatFlush));

        // UI systems
        #[cfg(not(feature = "web"))]
        app.add_systems(Update, bevy::window::close_on_esc);
        app.add_systems(
            Update,
            (
                HealthBar::update,
                AlarmMeter::update,
                Cutscene::update,
                Music::update,
                Message::show_death_message,
                Message::show_victory_message,
            ),
        );
    }
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
    mut detect_events: ResMut<Events<DetectEvent>>,
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

use bevy::{
    prelude::*,
    transform::systems::{propagate_transforms, sync_simple_transforms},
};
use bevy_ecs_ldtk::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::{common_conditions::action_just_pressed, prelude::*};

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
    asset::{AudioKey, Handles},
    camera::{GameCamera, GameCameraTemplate},
    combat::{DeathEffects, DeathEvent, HitEffects, HitEvent, HurtEffects},
    cutscene::{Cutscene, CutsceneTemplate, Message},
    hud::{AlarmMeter, AlarmMeterTemplate, FontSizeHack, HealthBar},
    map::{spawn_level_entities, Exit, MapTemplate, Plate, Victory, VictorySquare},
    mob::{
        enemy::{Alarm, DetectEvent, DifficultyCurve, EnemyAi},
        player::{PlayerAction, PlayerControl, Playthrough},
        Mob,
    },
    music::Music,
    util::{DespawnSet, ZRampByY},
};

const TITLE: &str = "Sai Defects";

#[derive(Actionlike, Clone)]
pub enum GameAction {
    Restart,
    Confirm,
}

#[derive(SystemSet, Clone, Debug, Eq, PartialEq, Hash)]
enum UpdateSet {
    // <-- Spawn level
    Synchronize,
    Animate,
    PostAnimate,
    Combat,
    CombatFlush,
    // <-- Physics
    // <-- Spawn / Despawn
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
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
        app.add_plugins(
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
        )
        .add_plugin(AudioPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(InputManagerPlugin::<GameAction>::default())
        .add_plugin(InputManagerPlugin::<PlayerAction>::default())
        .add_plugin(LdtkPlugin);
        #[cfg(feature = "debug_mode")]
        app.add_plugin(crate::debug::DebugPlugin::default());

        // Startup systems
        app.add_startup_system(Handles::load.in_base_set(StartupSet::PreStartup));
        app.add_startup_system(spawn_game);

        // First systems
        app.add_systems(
            (
                restart_game.run_if(action_just_pressed(GameAction::Restart)),
                Cutscene::advance.run_if(action_just_pressed(GameAction::Confirm)),
            )
                .in_base_set(CoreSet::First),
        );

        // Pre-update systems
        app.add_systems(
            (Exit::detect, spawn_level_entities)
                .chain()
                .in_base_set(CoreSet::PreUpdate),
        );

        // Game logic system sets
        app.configure_sets(
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
            (
                sync_simple_transforms,
                propagate_transforms,
                ZRampByY::apply,
                VirtualParent::copy_transform.after(ZRampByY::apply),
                Offset::apply.after(VirtualParent::copy_transform),
                Plate::detect,
                VictorySquare::detect,
                HitEvent::detect.before(EnemyAi::think),
                DetectEvent::detect.before(EnemyAi::think),
                DifficultyCurve::apply.before(EnemyAi::think),
                EnemyAi::think,
                PlayerControl::record_inputs,
            )
                .in_set(UpdateSet::Synchronize),
        );

        // Animation systems
        app.add_systems(
            (
                Playthrough::detect_defection,
                GameCamera::cut_to_new_target,
                GameCamera::follow_target,
                FontSizeHack::scale,
                WalkAnimation::trigger,
                WalkAnimation::play_step_sound.after(WalkAnimation::trigger),
                WalkAnimation::update.after(WalkAnimation::play_step_sound),
                WalkAnimation::apply.after(WalkAnimation::update),
                DeathAnimation::update,
                DeathAnimation::apply.after(DeathAnimation::update),
                AttackAnimation::trigger,
                AttackAnimation::update.after(AttackAnimation::trigger),
                AttackAnimation::apply.after(AttackAnimation::update),
                FlinchAnimation::update,
                FlinchAnimation::apply.after(FlinchAnimation::update),
            )
                .in_set(UpdateSet::Animate),
        );

        // Post-animation systems
        app.add_systems(
            (Mob::set_facing, Facing::apply.after(Mob::set_facing)).in_set(UpdateSet::PostAnimate),
        );

        // Combat systems
        app.add_systems(
            (
                Mob::apply_movement,
                HitEffects::apply,
                HurtEffects::apply,
                DeathEffects::apply.after(HitEffects::apply),
                HitEffects::cleanup.after(DeathEffects::apply),
                HitEffects::spawn_from_inputs.after(HitEffects::cleanup),
                Lifetime::apply,
            )
                .in_set(UpdateSet::Combat),
        );
        app.add_system(apply_system_buffers.in_set(UpdateSet::CombatFlush));

        // Spawn / despawn systems
        app.add_system(DespawnSet::apply.in_base_set(CoreSet::Last));

        // UI systems
        #[cfg(not(feature = "wasm"))]
        app.add_system(bevy::window::close_on_esc);
        app.add_systems((
            HealthBar::update,
            AlarmMeter::update,
            Cutscene::update,
            Music::update,
            Message::show_death_message,
            Message::show_victory_message,
        ));
    }
}

fn restart_game(
    mut commands: Commands,
    handle: Res<Handles>,
    entity_query: Query<
        Entity,
        (
            Or<(With<Handle<LdtkAsset>>, With<Message>)>,
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

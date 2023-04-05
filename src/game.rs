use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::InputManagerPlugin;

use crate::{
    animation::{
        AttackAnimation,
        DeathAnimation,
        Facing,
        Lifetime,
        Offset,
        VirtualParent,
        WalkAnimation,
    },
    asset::{Handles, LevelKey},
    camera::{CameraPlugin, GameCameraTemplate},
    combat::{DeathEffects, DeathEvent, HitEffects, HitEvent, HurtEffects},
    hud::{AlarmMeter, AlarmMeterTemplate, HealthBar},
    map::MapPlugin,
    mob::{
        enemy::{Alarm, DetectEvent, EnemyAi, EnemyTemplate},
        player::{PlayerAction, PlayerControl, PlayerTemplate},
        Mob,
    },
    util::{DespawnSet, ZRampByY},
};

const TITLE: &str = "Sai Defects";

#[derive(SystemSet, Clone, Debug, Eq, PartialEq, Hash)]
enum UpdateSet {
    Synchronize,
    Animate,
    PostAnimate,
    Combat,
    CombatFlush,
    // <-- Physics
    // Spawn / Despawn
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Resources
        app.insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..default()
        })
        .init_resource::<Handles>()
        .init_resource::<DespawnSet>()
        .insert_resource(Alarm::empty(100.0));

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
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(InputManagerPlugin::<PlayerAction>::default())
        .add_plugin(MapPlugin)
        .add_plugin(CameraPlugin);
        #[cfg(feature = "debug_mode")]
        app.add_plugin(crate::debug::DebugPlugin::default());

        // Startup systems
        app.add_startup_system(Handles::load.in_base_set(StartupSet::PreStartup));
        app.add_startup_system(spawn_scene);

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
                ZRampByY::apply,
                VirtualParent::copy_transform.after(ZRampByY::apply),
                Offset::apply.after(VirtualParent::copy_transform),
                HitEvent::detect,
                DetectEvent::detect.before(EnemyAi::think),
                Alarm::scale_difficulty.before(EnemyAi::think),
                EnemyAi::think,
                PlayerControl::record_inputs,
            )
                .in_set(UpdateSet::Synchronize),
        );

        // Animation systems
        app.add_systems(
            (
                WalkAnimation::trigger,
                WalkAnimation::play_step_sound.after(WalkAnimation::trigger),
                WalkAnimation::update.after(WalkAnimation::play_step_sound),
                WalkAnimation::apply.after(WalkAnimation::update),
                DeathAnimation::update,
                DeathAnimation::apply.after(DeathAnimation::update),
                AttackAnimation::trigger,
                AttackAnimation::update.after(AttackAnimation::trigger),
                AttackAnimation::apply.after(AttackAnimation::update),
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
        app.add_systems((DespawnSet::apply, spawn_instances).in_base_set(CoreSet::Last));

        // UI systems
        app.add_systems((
            bevy::window::close_on_esc,
            HealthBar::update,
            AlarmMeter::update,
        ));
    }
}

fn spawn_scene(mut commands: Commands, handle: Res<Handles>) {
    // Map
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: handle.levels[&LevelKey::TestLevel].clone(),
        ..default()
    });

    // HUD
    AlarmMeterTemplate.spawn(&mut commands);

    // Camera
    GameCameraTemplate::<PlayerControl>::default().spawn(&mut commands);
}

pub fn spawn_instances(
    mut commands: Commands,
    handle: Res<Handles>,
    entity_query: Query<(Entity, &Transform, &EntityInstance, &Parent), Added<EntityInstance>>,
    transform_query: Query<&Transform, (With<Children>, Without<EntityInstance>)>,
    player_query: Query<&PlayerControl>,
) {
    for (entity, transform, instance, parent) in &entity_query {
        match instance.identifier.as_str() {
            "Player" => {
                // We only want one player and LDtk doesn't know that
                if player_query.is_empty() {
                    // Since we're going to create a new entity, and we therefore will not inherit the parent's
                    // transform automatically, we need to manually add it.
                    let parent_transform = transform_query
                        .get(parent.get())
                        .copied()
                        .unwrap_or_default();
                    let position = (transform.translation + parent_transform.translation).xy();
                    PlayerTemplate {
                        position,
                        ..default()
                    }
                    .spawn(&mut commands, &handle);
                }
                // Despawn the marker entity
                commands.entity(entity).despawn_recursive();
            },
            "Enemy" => {
                EnemyTemplate {
                    position: transform.translation.xy(),
                    ..default()
                }
                .with_random_name()
                .spawn(&mut commands, &handle, entity);
            },
            _ => (),
        }
    }
}

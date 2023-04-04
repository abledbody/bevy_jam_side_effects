use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{DeathAnimation, Facing, Lifetime, Offset, VirtualParent, WalkAnimation, AttackAnimation},
    asset::{Handles, LevelKey},
    camera::{CameraPlugin, GameCameraTemplate},
    combat::{DeathEffects, DeathEvent, HitEffects, HitEvent},
    hud::HealthBar,
    map::MapPlugin,
    mob::{
        enemy::EnemyTemplate,
        player::{PlayerControl, PlayerTemplate},
        Mob, MobInputs,
    },
    util::{DespawnSet, ZRampByY},
};

const TITLE: &str = "Sai Defects";

#[derive(SystemSet, Clone, Debug, Eq, PartialEq, Hash)]
enum UpdateSet {
    Input,
    Combat,
    CombatFlush,
    Animate,
    SpawnDespawn,
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
        .init_resource::<DespawnSet>();

        // Events
        app.add_event::<HitEvent>().add_event::<DeathEvent>();

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
                UpdateSet::Input,
                UpdateSet::Combat,
                UpdateSet::CombatFlush,
                UpdateSet::Animate,
                UpdateSet::SpawnDespawn,
            )
                .chain(),
        );

        // Input systems
        app.add_systems(
            (
                PlayerControl::record_inputs,
                Mob::apply_movement,
                Mob::set_facing,
            )
                .chain()
                .in_set(UpdateSet::Input),
        );

        // Combat systems
        app.add_systems(
            (
                HitEvent::detect,
                HitEffects::apply.after(HitEvent::detect),
                DeathEffects::apply.after(HitEffects::apply),
                HitEffects::cleanup.after(DeathEffects::apply),
                HitEffects::spawn_from_inputs.after(HitEffects::cleanup),
				MobInputs::animate_attack,
                Lifetime::apply,
            )
                .in_set(UpdateSet::Combat),
        );
        app.add_system(apply_system_buffers.in_set(UpdateSet::CombatFlush));

        // Animation systems
        app.add_systems(
            (
                HealthBar::update,
                ZRampByY::apply,
                VirtualParent::copy_transform.after(ZRampByY::apply),
                Offset::apply.after(VirtualParent::copy_transform),
                WalkAnimation::update,
                WalkAnimation::apply
                    .after(Offset::apply)
                    .before(Facing::apply)
                    .after(WalkAnimation::update),
                DeathAnimation::update,
                DeathAnimation::apply
                    .after(Offset::apply)
                    .before(Facing::apply)
                    .after(DeathAnimation::update),
				AttackAnimation::update,
				AttackAnimation::apply
					.after(Offset::apply)
					.before(Facing::apply)
					.after(AttackAnimation::update),
                Facing::apply,
            )
                .in_set(UpdateSet::Animate),
        );

        // Spawn / despawn systems
        app.add_systems((DespawnSet::apply, spawn_instances).in_set(UpdateSet::SpawnDespawn));

        // UI systems
        app.add_system(bevy::window::close_on_esc);
    }
}

fn spawn_scene(mut commands: Commands, handle: Res<Handles>) {
    // Map
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: handle.levels[&LevelKey::TestLevel].clone(),
        ..default()
    });

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

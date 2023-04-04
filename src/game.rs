use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{self, DeathAnimation, Facing, Lifetime, Offset, WalkAnimation},
    asset::{Handles, LevelKey},
    camera::{CameraPlugin, GameCameraTemplate},
    combat::{DeathEffects, DeathEvent, HitEffects, HitEvent},
    map::MapPlugin,
    mob::{
        enemy::EnemyTemplate,
        player::{PlayerControl, PlayerTemplate},
        Mob,
    },
    util::{DespawnSet, ZRampByY},
};

const TITLE: &str = "Sai Defects";
const CLEAR_COLOR: Color = Color::rgba(0.18, 0.15, 0.23, 1.0);

#[derive(SystemSet, Clone, Debug, Eq, PartialEq, Hash)]
enum UpdateSet {
    Input,
    Combat,
    SpawnDespawn,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Resources
        app.insert_resource(ClearColor(CLEAR_COLOR))
            .insert_resource(RapierConfiguration {
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
        app.configure_sets((UpdateSet::Input, UpdateSet::Combat, UpdateSet::SpawnDespawn).chain());

        // Input systems
        app.add_systems(
            (
                PlayerControl::record_inputs,
                Mob::apply_input.after(PlayerControl::record_inputs),
            )
                .in_set(UpdateSet::Input),
        );

        // Combat systems
        app.add_systems(
            (
                HitEvent::detect,
                HitEffects::apply.after(HitEvent::detect),
                DeathEffects::apply.after(HitEffects::apply),
                Lifetime::apply,
            )
                .in_set(UpdateSet::Combat),
        );

        // Spawn / despawn systems
        app.add_systems((DespawnSet::apply, spawn_instances).in_set(UpdateSet::SpawnDespawn));

        // Visual systems
        app.add_systems((
            ZRampByY::apply,
            Mob::set_facing,
            Facing::update_sprites.after(Mob::set_facing),
            WalkAnimation::update.after(Mob::set_facing),
            DeathAnimation::update.after(Mob::set_facing),
            animation::sum_animations.after(WalkAnimation::update),
        ));

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
    entity_query: Query<(Entity, &Transform, &EntityInstance)>,
) {
    for (entity, transform, instance) in &entity_query {
        // Despawn the marker entity
        commands.entity(entity).despawn_recursive();

        // Replace with the actual entity
        let position = transform.translation.xy();
        match instance.identifier.as_str() {
            "Player" => {
                PlayerTemplate {
                    position,
                    ..default()
                }
                .spawn(&mut commands, &handle);
            },
            "Enemy" => {
                EnemyTemplate {
                    position,
                    ..default()
                }
                .spawn(&mut commands, &handle);
            },
            _ => (),
        }
    }
}

use bevy::{
    math::{vec2, Vec3Swizzles},
    prelude::*,
};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    asset::Handles,
    combat::COLLISION_GROUP,
    mob::{
        enemy::EnemyTemplate,
        player::{PlayerControl, PlayerTemplate},
    },
};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            ..default()
        })
        .insert_resource(LevelSelection::Index(0))
        .add_plugin(LdtkPlugin)
        .add_systems((spawn_walls, spawn_instances, update_level_selection));
    }
}

#[derive(Component, Reflect, Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct Wall;

#[derive(Bundle, Clone, Default, Debug)]
pub struct WallBundle {
    wall: Wall,
    collider: Collider,
    collision_groups: CollisionGroups,
    rigid_body: RigidBody,
    friction: Friction,
}

fn spawn_walls(mut commands: Commands, tags: Query<(Entity, &TileEnumTags), Added<TileEnumTags>>) {
    for (entity, tag) in &tags {
        if tag.tags.iter().find(|s| s.as_str() == "Full").is_none() {
            continue;
        }

        commands.entity(entity).insert(WallBundle {
            collider: Collider::cuboid(8.0, 8.0),
            collision_groups: CollisionGroups {
                memberships: COLLISION_GROUP,
                filters: COLLISION_GROUP,
            },
            rigid_body: RigidBody::Fixed,
            ..default()
        });
    }
}

fn spawn_instances(
    mut commands: Commands,
    handle: Res<Handles>,
    entity_query: Query<(Entity, &Transform, &EntityInstance, &Parent), Added<EntityInstance>>,
    transform_query: Query<&Transform, (With<Children>, Without<EntityInstance>)>,
    player_query: Query<&PlayerControl>,
) {
    for (entity, transform, instance, parent) in &entity_query {
        let parent_transform = transform_query
            .get(parent.get())
            .copied()
            .unwrap_or_default();

        // Despawn the marker entity
        commands.entity(entity).despawn_recursive();

        // Replace with the actual entity
        match instance.identifier.as_str() {
            "Player" => {
                // We only want one player and LDtk doesn't know that
                if player_query.is_empty() {
                    // Since we're going to create a new entity, and we therefore will not inherit the parent's
                    // transform automatically, we need to manually add it.
                    let position = (transform.translation + parent_transform.translation).xy();
                    PlayerTemplate {
                        position,
                        ..default()
                    }
                    .spawn(&mut commands, &handle);
                }
            },
            "Enemy" => {
                let enemy = EnemyTemplate {
                    position: transform.translation.xy(),
                    ..default()
                }
                .with_random_name()
                .spawn(&mut commands, &handle);

                commands.entity(parent.get()).add_child(enemy);
            },
            _ => (),
        }
    }
}

fn update_level_selection(
    level_query: Query<(&Handle<LdtkLevel>, &Transform), Without<PlayerControl>>,
    player_query: Query<&Transform, With<PlayerControl>>,
    mut level_selection: ResMut<LevelSelection>,
    ldtk_levels: Res<Assets<LdtkLevel>>,
) {
    let Ok(player_transform) = player_query.get_single() else { return };
    let player_pos = player_transform.translation.xy();

    // Iterate over unselected levels to find one that contains the player
    for (level_handle, level_transform) in &level_query {
        let Some(ldtk_level) = ldtk_levels.get(level_handle) else { continue };
        if level_selection.is_match(&usize::MAX, &ldtk_level.level) {
            continue;
        }

        let level_pos = level_transform.translation.xy();
        let level_size = vec2(
            ldtk_level.level.px_wid as f32,
            ldtk_level.level.px_hei as f32,
        );
        let level_bounds = Rect::from_corners(level_pos, level_pos + level_size);

        if level_bounds.contains(player_pos) {
            println!("Updating level selection: {}", ldtk_level.level.identifier);
            *level_selection = LevelSelection::Iid(ldtk_level.level.iid.clone());
            break;
        }
    }
}

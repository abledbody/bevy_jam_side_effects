use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{combat::COLLISION_GROUP, mob::player::PlayerControl};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LdtkPlugin)
            .insert_resource(LdtkSettings {
                level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                    load_level_neighbors: true,
                },
                set_clear_color: SetClearColor::FromLevelBackground,
                ..Default::default()
            })
            .insert_resource(LevelSelection::Index(0))
            .add_system(spawn_walls)
            .add_system(update_level_selection);
    }
}

#[derive(Component, Reflect, Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct Wall;

#[derive(Clone, Default, Debug, Bundle)]
pub struct WallBundle {
    wall: Wall,
    collider: Collider,
    collision_groups: CollisionGroups,
    rigid_body: RigidBody,
    friction: Friction,
}

fn spawn_walls(mut commands: Commands, tags: Query<(Entity, &TileEnumTags), Added<TileEnumTags>>) {
    for (entity, tag) in &tags {
        if tag.tags.iter().find(|s| s.as_str() == "Full").is_some() {
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
}

pub fn update_level_selection(
    level_query: Query<(&Handle<LdtkLevel>, &Transform), Without<PlayerControl>>,
    player_query: Query<&Transform, With<PlayerControl>>,
    mut level_selection: ResMut<LevelSelection>,
    ldtk_levels: Res<Assets<LdtkLevel>>,
) {
    for (level_handle, level_transform) in &level_query {
        if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
            let level_bounds = Rect {
                min: Vec2::new(level_transform.translation.x, level_transform.translation.y),
                max: Vec2::new(
                    level_transform.translation.x + ldtk_level.level.px_wid as f32,
                    level_transform.translation.y + ldtk_level.level.px_hei as f32,
                ),
            };

            for player_transform in &player_query {
                if player_transform.translation.x < level_bounds.max.x
                    && player_transform.translation.x >= level_bounds.min.x
                    && player_transform.translation.y < level_bounds.max.y
                    && player_transform.translation.y >= level_bounds.min.y
                    && !level_selection.is_match(&0, &ldtk_level.level)
                {
                    *level_selection = LevelSelection::Iid(ldtk_level.level.iid.clone());
                }
            }
        }
    }
}

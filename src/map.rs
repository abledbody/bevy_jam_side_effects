use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::combat::COLLISION_GROUP;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(LdtkPlugin)
            .insert_resource(LevelSelection::Index(0))
            .add_system(spawn_walls);
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

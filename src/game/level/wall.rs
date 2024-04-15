use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::game::combat::COLLISION_GROUP;

pub struct WallPlugin;

impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Wall>();
    }
}

#[derive(Component, Reflect)]
pub struct Wall;

pub struct WallTemplate {
    pub transform: Transform,
}

impl WallTemplate {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        commands
            .spawn((
                Name::new("Wall"),
                TransformBundle::from_transform(self.transform),
                Collider::cuboid(8.0, 8.0),
                CollisionGroups {
                    memberships: COLLISION_GROUP,
                    filters: COLLISION_GROUP,
                },
                Friction::new(0.0),
                RigidBody::Fixed,
                Wall,
            ))
            .id()
    }
}

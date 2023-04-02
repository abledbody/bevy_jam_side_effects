use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component, Reflect)]
pub struct Hitbox {
    damage: f32,
    knockback: f32,
}

impl Hitbox {
    fn spawn(commands: &mut Commands, radius: f32) -> Entity {
        let mut entity = commands.spawn((
            RigidBody::KinematicPositionBased,
            Collider::ball(radius),
            CollisionGroups {
                memberships: Group::ALL,
                filters: Group::ALL,
            },
            ActiveEvents::COLLISION_EVENTS,
        ));
        #[cfg(feature = "debug_mode")]
        entity.insert(Name::new("Hitbox"));

        entity.id()
    }
}

#[derive(Component, Reflect)]
pub struct Hurtbox;

impl Hurtbox {
    fn spawn(commands: &mut Commands) -> Entity {
        let mut entity = commands.spawn((
            RigidBody::KinematicPositionBased,
            Collider::ball(1.0),
            CollisionGroups {
                memberships: Group::ALL,
                filters: Group::ALL,
            },
            ActiveEvents::COLLISION_EVENTS,
        ));
        #[cfg(feature = "debug_mode")]
        entity.insert(Name::new("Hitbox"));

        entity.id()
    }
}

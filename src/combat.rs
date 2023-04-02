use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::animation::Offset;

#[derive(Component, Reflect)]
pub struct Effects {
    damage: f32,
    knockback: f32,
}

pub struct Hitbox {
    pub offset: Offset,
    pub radius: f32,
    pub effects: Effects,
}

impl Hitbox {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let mut entity = commands.spawn((
            self.offset,
            self.effects,
            RigidBody::KinematicPositionBased,
            Collider::ball(self.radius),
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

pub struct Hurtbox {
    pub offset: Offset,
    pub radius: f32,
}

impl Hurtbox {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let mut entity = commands.spawn((
            self.offset,
            RigidBody::KinematicPositionBased,
            Collider::ball(self.radius),
            CollisionGroups {
                memberships: Group::ALL,
                filters: Group::ALL,
            },
            ActiveEvents::COLLISION_EVENTS,
        ));
        #[cfg(feature = "debug_mode")]
        entity.insert(Name::new("Hurtbox"));

        entity.id()
    }
}

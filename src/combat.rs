use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::animation::Offset;

pub const COLLISION_GROUP: Group = Group::GROUP_1;

#[derive(Component, Reflect)]
pub struct Effects {
    damage: f32,
    knockback: f32,
}

pub struct HitboxTemplate {
    pub offset: Vec2,
    pub radius: f32,
    pub damage: f32,
    pub knockback: f32,
}

impl HitboxTemplate {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let mut entity = commands.spawn((
            Offset(self.offset),
            TransformBundle::default(),
            Collider::ball(self.radius),
            Sensor,
            CollisionGroups {
                memberships: Group::ALL,
                filters: Group::ALL,
            },
            ActiveEvents::COLLISION_EVENTS,
            Effects {
                damage: self.damage,
                knockback: self.knockback,
            },
        ));
        #[cfg(feature = "debug_mode")]
        entity.insert(Name::new("Hitbox"));

        entity.id()
    }
}

pub struct HurtboxTemplate {
    pub offset: Vec2,
    pub radius: f32,
}

impl HurtboxTemplate {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let mut entity = commands.spawn((
            Offset(self.offset),
            TransformBundle::default(),
            Collider::ball(self.radius),
            Sensor,
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

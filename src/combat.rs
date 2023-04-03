use bevy::{math::Vec3Swizzles, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{Lifetime, Offset},
    mob::Health,
    util::VirtualParent,
};

pub const COLLISION_GROUP: Group = Group::GROUP_1;
pub const HITBOX_GROUP: Group = Group::GROUP_2;
pub const PLAYER_HURTBOX_GROUP: Group = Group::GROUP_3;
pub const ENEMY_HURTBOX_GROUP: Group = Group::GROUP_4;

#[derive(Copy, Clone)]
pub enum Faction {
    Player,
    Enemy,
}

impl Faction {
    pub fn hitbox_groups(&self) -> CollisionGroups {
        CollisionGroups {
            memberships: HITBOX_GROUP,
            filters: match self {
                Faction::Player => ENEMY_HURTBOX_GROUP,
                Faction::Enemy => PLAYER_HURTBOX_GROUP,
            },
        }
    }

    pub fn hurtbox_groups(&self) -> CollisionGroups {
        CollisionGroups {
            memberships: match self {
                Faction::Player => PLAYER_HURTBOX_GROUP,
                Faction::Enemy => ENEMY_HURTBOX_GROUP,
            },
            filters: HITBOX_GROUP,
        }
    }
}

#[derive(Component, Reflect)]
pub struct HitEffects {
    damage: f32,
    knockback: f32,
}

impl HitEffects {
    pub fn apply(
        mut collision_events: EventReader<CollisionEvent>,
        hit_effects_query: Query<&HitEffects>,
        mut health_query: Query<&mut Health>,
        mut velocity_query: Query<&mut Velocity>,
        virtual_parent_query: Query<&VirtualParent>,
        transform_query: Query<&Transform>,
    ) {
        for &event in collision_events.iter() {
            let CollisionEvent::Started(entity1, entity2, _) = event else {
                continue
            };

            let mut handle_collision = |actor: Entity, target: Entity| {
                let Ok(effect) = hit_effects_query.get(actor) else { return };
                if let Ok(mut health) = health_query.get_mut(target) {
                    // TODO: System that detects when health <= 0 and triggers a Death event
                    health.0 -= effect.damage;
                }
                if let Ok(mut velocity) = velocity_query.get_mut(target) {
                    let Ok(actor_transform) = virtual_parent_query
                        .get(actor)
                        .and_then(|parent| transform_query.get(parent.0)) else {
                        return
                    };
                    let Ok(target_transform) = transform_query.get(target) else {
                        return
                    };

                    let scale = 40.0;
                    let direction = (target_transform.translation.xy()
                        - actor_transform.translation.xy())
                    .normalize_or_zero();
                    velocity.linvel = effect.knockback * scale * direction;
                }
            };

            handle_collision(entity1, entity2);
            handle_collision(entity2, entity1);
        }
    }
}

pub struct HitboxTemplate {
    pub offset: Vec2,
    pub radius: f32,
    pub damage: f32,
    pub knockback: f32,
    pub faction: Faction,
    pub lifetime: f32,
    pub parent: Entity,
}

impl HitboxTemplate {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let mut entity = commands.spawn((
            Offset(self.offset),
            TransformBundle::default(),
            Collider::ball(self.radius),
            Sensor,
            self.faction.hitbox_groups(),
            ActiveEvents::COLLISION_EVENTS,
            HitEffects {
                damage: self.damage,
                knockback: self.knockback,
            },
            Lifetime(self.lifetime),
            VirtualParent(self.parent),
        ));
        #[cfg(feature = "debug_mode")]
        entity.insert(Name::new("Hitbox"));

        entity.id()
    }
}

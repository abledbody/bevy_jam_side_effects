use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{animation::Offset, game::TIME_STEP, mob::Health};

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
    ) {
        for &event in collision_events.iter() {
            let CollisionEvent::Started(entity1, entity2, _) = event else {
                continue
            };

            let mut handle_collision = |x: Entity, y: Entity| {
                if let Ok(effect1) = hit_effects_query.get(x) {
                    if let Ok(mut health2) = health_query.get_mut(y) {
                        // TODO: System that detects when health <= 0 and triggers a Death event
                        health2.0 -= effect1.damage;
                    }
                    if let Ok(mut velocity2) = velocity_query.get_mut(y) {
                        // TODO: Actually implement knockback
                        velocity2.linvel = 100.0 * effect1.knockback * Vec2::ONE;
                    }
                }
            };

            handle_collision(entity1, entity2);
            handle_collision(entity2, entity1);
        }
    }
}

#[derive(Component, Reflect)]
pub struct Lifetime(pub f32);

impl Lifetime {
    pub fn apply(mut commands: Commands, mut lifetime_query: Query<(Entity, &mut Lifetime)>) {
        for (entity, mut lifetime) in &mut lifetime_query {
            lifetime.0 -= TIME_STEP;
            if lifetime.0 <= 0.0 {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

pub struct HitboxTemplate {
    pub offset: Vec2,
    pub radius: f32,
    pub damage: f32,
    pub knockback: f32,
    pub faction: Faction,
    // TODO: Implement this
    pub lifetime: f32,
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
        ));
        #[cfg(feature = "debug_mode")]
        entity.insert(Name::new("Hitbox"));

        entity.id()
    }
}

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{game::TIME_STEP, math::MoveTowards};

pub mod enemy;
pub mod player;

#[derive(Component, Reflect)]
pub struct Health(pub f32);

#[derive(Component, Reflect)]
pub struct Mob {
    speed: f32,
    acceleration: f32,
    brake_deceleration: f32,
    idle_threshold: f32,
}

impl Mob {
    pub fn flip_by_direction(
        mob_query: Query<(&Mob, &MobInputs, &Velocity, &Children)>,
        mut sprite_query: Query<(&mut Sprite, &mut Transform)>,
    ) {
        for (mob, mob_inputs, velocity, children) in &mob_query {
            for child in children {
                let Ok((mut sprite, transform)) =
					sprite_query.get_mut(*child) else {continue;};

                if mob_inputs.movement.x == 0.0 {
                    continue;
                }
                if velocity.linvel.x.abs() < mob.idle_threshold {
                    sprite.flip_x = mob_inputs.movement.x < 0.0;
                } else {
                    sprite.flip_x = velocity.linvel.x < 0.0;
                }
            }
        }
    }

    pub fn apply_input(mut mob_query: Query<(&Mob, &mut Velocity, &MobInputs)>) {
        for (mob, mut velocity, mob_inputs) in &mut mob_query {
            let input_direction = mob_inputs.movement.normalize_or_zero();
            let input_magnitude = mob_inputs.movement.length().min(1.0);

            let mut acceleration = mob.acceleration;
            if input_direction.dot(velocity.linvel) < 0.0 {
                acceleration = mob.brake_deceleration;
            }

            let target_velocity = input_direction * input_magnitude * mob.speed;
            velocity.linvel = velocity
                .linvel
                .move_towards(target_velocity, acceleration * TIME_STEP);
        }
    }

    pub fn player() -> Self {
        Self {
            speed: 110.0,
            acceleration: 900.0,
            brake_deceleration: 1800.0,
            idle_threshold: 10.0,
        }
    }
}

impl Default for Mob {
    fn default() -> Self {
        Mob::player()
    }
}

#[derive(Bundle)]
pub struct MobBundle {
    pub mob: Mob,
    pub mob_inputs: MobInputs,
    pub health: Health,
    pub velocity: Velocity,
    pub rigid_body: RigidBody,
    pub locked_axes: LockedAxes,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub solver_groups: SolverGroups,
}

impl Default for MobBundle {
    fn default() -> Self {
        Self {
            mob: Mob::default(),
            mob_inputs: MobInputs::default(),
            health: Health(100.0),
            velocity: Velocity::default(),
            rigid_body: RigidBody::default(),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            collider: Collider::ball(12.0),
            collision_groups: CollisionGroups {
                memberships: Group::ALL,
                filters: Group::ALL,
            },
            solver_groups: SolverGroups {
                memberships: Group::ALL,
                filters: Group::ALL,
            },
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct MobInputs {
    pub movement: Vec2,
}

#[derive(Component)]
pub struct Offset(pub Vec2);

impl Offset {
    pub fn apply(mut offset_query: Query<(&Offset, &Sprite, &mut Transform)>) {
        for (offset, sprite, mut transform) in &mut offset_query {
            transform.translation.x = offset.0.x * if sprite.flip_x { -1.0 } else { 1.0 };
            transform.translation.y = offset.0.y * if sprite.flip_y { -1.0 } else { 1.0 };
        }
    }
}

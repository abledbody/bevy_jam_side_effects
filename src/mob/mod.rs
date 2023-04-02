use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::game::TIME_STEP;
use crate::math::MoveTowards;

pub mod enemy;
pub mod player;

#[derive(Component, Reflect)]
pub struct Health(pub f32);

#[derive(Component, Reflect)]
pub struct Mob {
    speed: f32,
    acceleration: f32,
}

impl Mob {
    pub fn apply_input(mut mob_query: Query<(&Mob, &mut Velocity, &MobInputs)>) {
        for (mob, mut velocity, mob_inputs) in &mut mob_query {
            let input_direction = mob_inputs.movement.normalize_or_zero();
            let input_magnitude = mob_inputs.movement.length().min(1.0);

            let target_velocity = input_direction * input_magnitude * mob.speed;
            velocity.linvel = velocity
                .linvel
                .move_towards(target_velocity, mob.acceleration * TIME_STEP);
        }
    }

    pub fn player() -> Self {
        Self {
            speed: 130.0,
            acceleration: 500.0,
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct MobInputs {
    pub movement: Vec2,
}

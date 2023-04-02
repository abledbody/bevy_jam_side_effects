use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::math::MoveTowards;

#[derive(Component)]
pub struct Mob {
    speed: f32,
    acceleration: f32,
}

impl Mob {
    pub fn apply_input(mut mob_query: Query<(&Mob, &mut Velocity, &MobInputs)>) {
        for (mob, mut velocity, mob_inputs) in &mut mob_query {
            // FIXME: Should be a fixed delta timestep.
            let dt = 1.0 / 60.0;

            let input_direction = mob_inputs.movement.normalize_or_zero();
            let input_magnitude = mob_inputs.movement.length().min(1.0);

            let target_velocity = input_direction * input_magnitude * mob.speed;
            velocity.linvel = velocity
                .linvel
                .move_towards(target_velocity, mob.acceleration * dt);
        }
    }
}

impl Default for Mob {
    fn default() -> Self {
        Self {
            speed: 130.0,
            acceleration: 500.0,
        }
    }
}

#[derive(Component)]
pub struct MobInputs {
    pub movement: Vec2,
}

impl Default for MobInputs {
    fn default() -> Self {
        Self {
            movement: Vec2::ZERO,
        }
    }
}

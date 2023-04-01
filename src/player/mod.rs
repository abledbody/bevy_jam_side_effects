use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::math::MoveTowards;

#[derive(Component)]
pub struct Player {
	speed: f32,
	acceleration: f32,
}

impl Player {
	pub fn player_movement(
		mut player_query: Query<(&Player, &mut Velocity)>,
		input_resource: Res<Input<KeyCode>>,
	) {
		for (player, mut velocity) in &mut player_query {
			// FIXME: Should be a fixed delta timestep.
			let dt = 1.0/60.0;

			let mut input = Vec2::ZERO;
			// It'd be nice to make bindings for this, but hey, it's a gamejam.
			// we could look at leafwing_input_manager
			if input_resource.pressed(KeyCode::A) { input.x -= 1.0 }
			if input_resource.pressed(KeyCode::D) { input.x += 1.0 }
			if input_resource.pressed(KeyCode::W) { input.y += 1.0 }
			if input_resource.pressed(KeyCode::S) { input.y -= 1.0 }

			let input_direction = input.normalize_or_zero();
			let input_magnitude = input.clamp_length_min(1.0);

			let target_velocity = input_direction * input_magnitude * player.speed;
			velocity.linvel =
				velocity.linvel.move_towards(target_velocity, player.acceleration * dt);
		}
	}
}
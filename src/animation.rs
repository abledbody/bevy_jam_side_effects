use bevy::prelude::*;

use crate::mob::MobInputs;

#[derive(Component)]
pub struct WalkAnimation {
	air_time: f32,
	height: f32,
	t: f32,
}

impl WalkAnimation {
	pub fn update(
		mut mob_query: Query<(&mut WalkAnimation, &MobInputs)>,
		time: Res<Time>,
	) {
		for (mut walk_animation, mob_inputs) in &mut mob_query {
			let moving = mob_inputs.movement.length() != 0.0;
			if walk_animation.t <= 0.0 || !moving {continue;}

			walk_animation.t += time.delta_seconds() / walk_animation.air_time;
			if walk_animation.t > 1.0 {
				if moving {
					walk_animation.t -= walk_animation.t.floor();
				}
				else {
					walk_animation.t = 0.0;
				}
			}
		}
	}
}

pub fn sum_animations(
	mob_query: Query<&WalkAnimation>,
	mut sprite_query: Query<&mut Sprite>,
) {
	
}
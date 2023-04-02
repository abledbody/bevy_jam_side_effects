use std::f32::consts::PI;

use bevy::prelude::*;

use crate::mob::{MobInputs, Offset};

#[derive(Component, Default)]
pub struct WalkAnimation {
	pub air_time: f32,
	pub height: f32,
	pub t: f32,
}

impl WalkAnimation {
	pub fn update(
		mob_query: Query<(&MobInputs, &Children)>,
		mut animator_query: Query<&mut WalkAnimation>,
		time: Res<Time>,
	) {
		for (mob_inputs, children) in &mob_query {
			let moving = mob_inputs.movement.length() != 0.0;

			for child in children {
				let Ok(mut walk_animation) = animator_query.get_mut(*child) else {continue;};
				
				if walk_animation.t <= 0.0 && !moving {continue;}
	
				walk_animation.t += time.delta_seconds() / walk_animation.air_time;
				
				// The rest of this manages the loop, or lack thereof.
				if walk_animation.t < 1.0 {continue;}
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
	mut sprite_query: Query<(&mut Offset, &WalkAnimation)>,
) {
	for (mut offset, walk_animation) in &mut sprite_query {
		// PI is used here because we only want half a rotation.
		offset.0.y = walk_animation.height * (walk_animation.t * PI).sin();
	}
}
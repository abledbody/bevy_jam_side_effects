use crate::mob::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

impl Player {
    pub fn record_controls(
        mut player_query: Query<&mut MobInputs, With<Player>>,
        input_resource: Res<Input<KeyCode>>,
    ) {
        for mut mob_inputs in &mut player_query {
            let mut movement = Vec2::ZERO;

            // It'd be nice to make bindings for this, but hey, it's a gamejam.
            // we could look at leafwing_input_manager
            if input_resource.pressed(KeyCode::A) {
                movement.x -= 1.0;
            }
            if input_resource.pressed(KeyCode::D) {
                movement.x += 1.0;
            }
            if input_resource.pressed(KeyCode::W) {
                movement.y += 1.0;
            }
            if input_resource.pressed(KeyCode::S) {
                movement.y -= 1.0;
            }

            mob_inputs.movement = movement;
        }
    }
}

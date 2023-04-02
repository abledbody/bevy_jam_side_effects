use bevy::{math::vec3, prelude::*};
use bevy_rapier2d::prelude::*;

use super::{Health, Mob, MobInputs};
use crate::asset::{Handles, ImageKey};

#[derive(Component, Reflect)]
pub struct Player;

#[derive(Component, Reflect, Default)]
pub struct Gold(f32);

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

    pub fn spawn(mut commands: Commands, handle: Res<Handles>) {
        let texture = ImageKey::GreenGnoll;
        let health = 100.0;
        let position = vec3(0.0, 0.0, 500.0);

        let mut entity = commands.spawn((
            SpriteBundle {
                texture: handle.image[&texture].clone(),
                transform: Transform::from_translation(position),
                ..default()
            },
            Mob::player(),
            MobInputs::default(),
            Player,
            Health(health),
            Gold::default(),
            (
                Velocity::default(),
                RigidBody::default(),
                LockedAxes::ROTATION_LOCKED,
            ),
        ));
        #[cfg(feature = "debug_mode")]
        entity.insert(Name::new("Player"));
    }
}

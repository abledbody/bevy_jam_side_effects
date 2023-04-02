use bevy::{math::{vec3, vec2}, prelude::*};

use super::{Health, Mob, MobBundle, MobInputs, Offset};
use crate::{asset::{Handles, ImageKey}, animation::WalkAnimation};

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

        // Sprite
        let mut sprite = commands.spawn((
            SpriteBundle {
                texture: handle.image[&texture].clone(),
                ..default()
            },
            Offset(vec2(2.0, 0.0)),
			WalkAnimation {
				air_time: 0.18,
				height: 3.0,
				..default()
			},
        ));
        #[cfg(feature = "debug_mode")]
        sprite.insert(Name::new("Sprite"));
        let sprite = sprite.id();

        // Drop shadow
        let mut drop_shadow = commands.spawn((
			SpriteBundle {
				texture: handle.image[&ImageKey::DropShadow].clone(),
				transform: Transform::from_xyz(0.0, -11.0, -position.z + 50.0),
				..default()
			}
		));
        #[cfg(feature = "debug_mode")]
        drop_shadow.insert(Name::new("DropShadow"));
        let drop_shadow = drop_shadow.id();

        // Parent entity
        let mut entity = commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(position),
                ..default()
            },
            Player,
            MobBundle {
                mob: Mob::player(),
                health: Health(health),
                ..default()
            }
        ));
        #[cfg(feature = "debug_mode")]
        entity.insert(Name::new("Player"));

        entity.add_child(sprite);
        entity.add_child(drop_shadow);
    }
}

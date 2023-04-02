use bevy::{
    math::{vec2, vec3},
    prelude::*,
};

use super::{Health, Mob, MobBundle, MobInputs};
use crate::{
    animation::{Offset, WalkAnimation},
    asset::{Handles, ImageKey},
    vfx,
};

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

        // Body
        let mut body = commands.spawn((
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
        body.insert(Name::new("Body"));
        let body = body.id();

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
            },
        ));
        #[cfg(feature = "debug_mode")]
        entity.insert(Name::new("Player"));

        entity.add_child(body);
        entity.add_child(vfx::drop_shadow(position.z, vec2(0.0, -11.0)));
    }
}

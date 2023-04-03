use bevy::{
    math::{vec2, vec3},
    prelude::*,
};

use super::{Health, Mob, MobBundle, MobInputs};
use crate::{
    asset::{Handles, ImageKey},
    combat::HitboxTemplate,
    mob::BodyTemplate,
    vfx::DropShadowTemplate,
};

#[derive(Component, Reflect, Default)]
pub struct Gold(f32);

#[derive(Component, Reflect)]
pub struct PlayerControl;

impl PlayerControl {
    pub fn record_inputs(
        mut player_query: Query<&mut MobInputs, With<PlayerControl>>,
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

pub struct PlayerTemplate {
    position: Vec3,
    health: f32,
}

impl Default for PlayerTemplate {
    fn default() -> Self {
        Self {
            // TODO: Use the map definition to set this
            position: vec3(19.0 * 8.0, 13.0 * 8.0, 500.0),
            health: 100.0,
        }
    }
}

impl PlayerTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        // Children
        let body = BodyTemplate {
            texture: ImageKey::GreenGnoll,
            offset: vec2(2.0, 0.0),
        }
        .spawn(commands, handle);
        let drop_shadow = DropShadowTemplate {
            parent_z: self.position.z,
            offset: vec2(0.0, -11.0),
        }
        .spawn(commands, handle);
        let axe_hitbox = HitboxTemplate {
            offset: vec2(4.0, 4.0),
            radius: 5.0,
            damage: 8.0,
            knockback: 5.0,
        }
        .spawn(commands);

        // Parent
        let mut entity = commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(self.position),
                ..default()
            },
            MobBundle {
                mob: Mob::player(),
                health: Health(self.health),
                ..default()
            },
            PlayerControl,
        ));
        #[cfg(feature = "debug_mode")]
        entity.insert(Name::new("Player"));

        entity.add_child(body);
        entity.add_child(drop_shadow);
        entity.add_child(axe_hitbox);

        entity.id()
    }
}

use bevy::{math::vec2, prelude::*};

use crate::{
    animation::Offset,
    asset::{Handles, ImageKey},
    mob::{Body, Health, MobBundle},
    vfx::DropShadow,
};

#[derive(Component, Reflect)]
pub struct Loot {
    pub gold: f32,
}

// TODO: Use this for AI
#[derive(Component, Reflect)]
pub struct Enemy;

impl Enemy {
    pub fn spawn(commands: &mut Commands, handle: &Handles, position: Vec3) -> Entity {
        let health = 20.0;
        let gold = 10.0;

        // Children
        let body = Body {
            texture: ImageKey::RedGnoll,
            offset: Offset(vec2(2.0, 0.0)),
        }
        .spawn(commands, handle);
        let drop_shadow = DropShadow {
            parent_z: position.z,
            offset: Offset(vec2(0.0, -11.0)),
        }
        .spawn(commands, handle);

        // Parent entity
        let mut entity = commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(position),
                ..default()
            },
            MobBundle {
                health: Health(health),
                ..default()
            },
            Enemy,
            Loot { gold },
        ));
        #[cfg(feature = "debug_mode")]
        entity.insert(Name::new("Enemy"));

        entity.add_child(body);
        entity.add_child(drop_shadow);

        entity.id()
    }
}

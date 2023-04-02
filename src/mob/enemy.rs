use bevy::{prelude::*, math::vec2};

use crate::{
    asset::{Handles, ImageKey},
    mob::{Health, MobBundle}, animation::WalkAnimation,
};

use super::Offset;

#[derive(Component, Reflect)]
pub struct Loot {
    pub gold: f32,
}

// TODO: Use this for AI
#[derive(Component, Reflect)]
pub struct Enemy;

impl Enemy {
    pub fn spawn(commands: &mut Commands, handle: &Handles, position: Vec3) -> Entity {
        let texture = ImageKey::RedGnoll;
        let health = 20.0;
        let gold = 10.0;

        // Sprite
        let mut sprite = commands.spawn((
            SpriteBundle {
                texture: handle.image[&texture].clone(),
                ..default()
            },
            Offset(vec2(2.0, 0.0)),
			WalkAnimation {
				air_time: 0.25,
				height: 4.0,
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
            MobBundle {
                health: Health(health),
                ..default()
            },
            Enemy,
            Loot { gold },
        ));
        #[cfg(feature = "debug_mode")]
        entity.insert(Name::new("Enemy"));

        entity.add_child(sprite);
        entity.add_child(drop_shadow);

        entity.id()
    }
}

use bevy::prelude::*;

use crate::{
    asset::{Handles, ImageKey},
    mob::{Health, MobBundle},
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
        let texture = ImageKey::RedGnoll;
        let health = 20.0;
        let gold = 10.0;

        let mut entity = commands.spawn((
            SpriteBundle {
                texture: handle.image[&texture].clone(),
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

        entity.id()
    }
}

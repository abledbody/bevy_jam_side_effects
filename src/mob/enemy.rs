use bevy::{
    math::{vec2, vec3},
    prelude::*,
};

use crate::{
    asset::{Handles, ImageKey},
    combat::CollisionboxTemplate,
    mob::{BodyTemplate, Health, MobBundle},
    vfx::DropShadowTemplate,
};

#[derive(Component, Reflect)]
pub struct Loot {
    pub gold: f32,
}

// TODO: Use this for AI
#[derive(Component, Reflect)]
pub struct EnemyAi;

pub struct EnemyTemplate {
    pub position: Vec3,
    pub variant: ImageKey,
    pub health: f32,
    pub gold: f32,
}

impl Default for EnemyTemplate {
    fn default() -> Self {
        Self {
            position: vec3(0.0, 0.0, 400.0),
            variant: ImageKey::RedGnoll,
            health: 20.0,
            gold: 10.0,
        }
    }
}

impl EnemyTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        // Children
        let body = BodyTemplate {
            texture: self.variant,
            offset: vec2(2.0, 0.0),
        }
        .spawn(commands, handle);
        let drop_shadow = DropShadowTemplate {
            parent_z: self.position.z,
            offset: vec2(0.0, -11.0),
        }
        .spawn(commands, handle);
        let body_collisionbox = CollisionboxTemplate {
            offset: Vec2::ZERO,
            radius: 8.0,
        }
        .spawn(commands);

        // Parent entity
        let mut entity = commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(self.position),
                ..default()
            },
            MobBundle {
                health: Health(self.health),
                ..default()
            },
            EnemyAi,
            Loot { gold: self.gold },
        ));
        #[cfg(feature = "debug_mode")]
        entity.insert(Name::new("Enemy"));

        entity.add_child(body);
        entity.add_child(drop_shadow);
        entity.add_child(body_collisionbox);

        entity.id()
    }
}

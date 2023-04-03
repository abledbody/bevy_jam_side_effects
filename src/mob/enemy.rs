use bevy::{math::vec2, prelude::*};

use crate::{
    asset::{Handles, ImageKey},
    combat::Faction,
    mob::{BodyTemplate, Health, MobBundle},
    vfx::DropShadowTemplate,
};

#[derive(Component, Reflect)]
pub struct Loot {
    pub gold: f32,
}

impl Default for Loot {
    fn default() -> Self {
        Self { gold: 10.0 }
    }
}

#[derive(Default, Component, Reflect)]
pub struct EnemyAi;

pub struct EnemyTemplate {
    pub position: Vec2,
    pub variant: ImageKey,
    pub health: f32,
    pub gold: f32,
}

impl Default for EnemyTemplate {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            variant: ImageKey::RedGnoll,
            health: 20.0,
            gold: 10.0,
        }
    }
}

impl EnemyTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        const FACTION: Faction = Faction::Enemy;

        // Children
        let body = BodyTemplate {
            texture: ImageKey::RedGnoll,
            offset: vec2(2.0, 0.0),
        }
        .spawn(commands, handle);
        let drop_shadow = DropShadowTemplate {
            offset: vec2(0.0, -11.0),
        }
        .spawn(commands, handle);

        // Parent entity
        let mut enemy = commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(self.position.extend(0.0)),
                ..default()
            },
            MobBundle {
                health: Health(self.health),
                ..default()
            }
            .with_faction(FACTION),
            EnemyAi,
            Loot { gold: self.gold },
        ));
        #[cfg(feature = "debug_mode")]
        enemy.insert(Name::new("Enemy"));

        enemy.add_child(body);
        enemy.add_child(drop_shadow);

        enemy.id()
    }
}

use bevy::{
    math::{vec2, Vec3Swizzles},
    prelude::*,
};
use bevy_ecs_ldtk::EntityInstance;

use crate::{
    asset::{Handles, ImageKey},
    combat::Faction,
    mob::{BodyTemplate, Health, MobBundle},
    vfx::DropShadowTemplate,
};

#[derive(Component, Reflect)]
pub struct Loot {
    pub gold: usize,
}

impl Default for Loot {
    fn default() -> Self {
        Self { gold: 10 }
    }
}

// TODO: Use this for AI
#[derive(Default, Component, Reflect)]
pub struct EnemyAi;

#[derive(Default, Bundle, Reflect)]
pub struct EnemyBundle {
    #[reflect(ignore)]
    spatial_bundle: SpatialBundle,
    mob_bundle: MobBundle,
    enemy_ai: EnemyAi,
    loot: Loot,
}

impl EnemyBundle {
    pub fn spawn(
        mut commands: Commands,
        handle: Res<Handles>,
        entity_query: Query<(Entity, &Transform, &EntityInstance), Added<EntityInstance>>,
    ) {
        for (entity, transform, instance) in &entity_query {
            if &instance.identifier == "Enemy" {
                const HEALTH: f32 = 20.0;
                const Z_IDX: f32 = 400.0;
                let faction = Faction::Enemy;

                // Children
                let body = BodyTemplate {
                    texture: ImageKey::RedGnoll,
                    offset: vec2(2.0, 0.0),
                }
                .spawn(&mut commands, &handle);
                let drop_shadow = DropShadowTemplate {
                    parent_z: Z_IDX,
                    offset: vec2(0.0, -11.0),
                }
                .spawn(&mut commands, &handle);

                // Parent entity
                let mut entity = commands.entity(entity);
                entity.insert(EnemyBundle {
                    spatial_bundle: SpatialBundle {
                        transform: Transform::from_translation(
                            transform.translation.xy().extend(Z_IDX),
                        ),
                        ..default()
                    },
                    mob_bundle: MobBundle {
                        health: Health(HEALTH),
                        ..default()
                    }
                    .with_faction(faction),
                    ..default()
                });
                #[cfg(feature = "debug_mode")]
                entity.insert(Name::new("Enemy"));

                entity.add_child(body);
                entity.add_child(drop_shadow);
            }
        }
    }
}

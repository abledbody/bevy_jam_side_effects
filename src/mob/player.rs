use bevy::{
    math::{vec2, Vec3Swizzles},
    prelude::*,
};
use bevy_ecs_ldtk::{EntityInstance, Worldly};

use super::{Health, Mob, MobBundle, MobInputs};
use crate::{
    asset::{Handles, ImageKey},
    combat::{Faction, HitboxTemplate},
    mob::BodyTemplate,
    vfx::DropShadowTemplate,
};

#[derive(Component, Reflect, Default)]
pub struct Gold(f32);

#[derive(Debug, Default, Component, Reflect)]
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

#[derive(Default, Component, Reflect)]
pub struct Player;

#[derive(Default, Bundle, Reflect)]
pub struct PlayerBundle {
    player: Player,
    #[reflect(ignore)]
    spatial_bundle: SpatialBundle,
    mob_bundle: MobBundle,
    player_control: PlayerControl,
    worldly: Worldly,
}

impl PlayerBundle {
    pub fn spawn(
        mut commands: Commands,
        handle: Res<Handles>,
        entity_query: Query<(Entity, &Transform, &EntityInstance), Added<EntityInstance>>,
    ) {
        for (entity, transform, instance) in &entity_query {
            if &instance.identifier == "Player" {
                const HEALTH: f32 = 100.0;
                const Z_IDX: f32 = 500.0;

                let faction = Faction::Player;

                // Children
                let body = BodyTemplate {
                    texture: ImageKey::GreenGnoll,
                    offset: vec2(2.0, 0.0),
                }
                .spawn(&mut commands, &handle);
                let drop_shadow = DropShadowTemplate {
                    parent_z: Z_IDX,
                    offset: vec2(0.0, -11.0),
                }
                .spawn(&mut commands, &handle);
                // TODO: Component to "Attach" hitbox to another entity.
                //       Like a child entity but not a child entity because Rapier.

                // Parent
                let mut player = commands.entity(entity);
                player.insert(PlayerBundle {
                    spatial_bundle: SpatialBundle {
                        transform: Transform::from_translation(
                            transform.translation.xy().extend(Z_IDX),
                        ),
                        ..default()
                    },
                    mob_bundle: MobBundle {
                        mob: Mob::player(),
                        health: Health(HEALTH),
                        ..default()
                    }
                    .with_faction(faction),
                    ..default()
                });
                #[cfg(feature = "debug_mode")]
                player.insert(Name::new("Player"));

                player.add_child(body);
                player.add_child(drop_shadow);

                // Axe hitbox
                HitboxTemplate {
                    offset: vec2(10.0, 4.0),
                    radius: 6.0,
                    damage: 8.0,
                    knockback: 5.0,
                    faction,
                    lifetime: f32::INFINITY,
                    parent: player.id(),
                }
                .spawn(&mut commands);
            }
        }
    }
}

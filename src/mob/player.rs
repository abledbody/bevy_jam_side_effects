use bevy::{math::vec2, prelude::*};

use super::{Health, Mob, MobBundle, MobInputs};
use crate::{
    asset::{Handles, ImageKey},
    combat::{Faction, HitboxTemplate},
    mob::BodyTemplate,
    vfx::{DropShadowTemplate, NametagTemplate},
};

const PLAYER_NAME: &str = "Sai";

#[derive(Component, Reflect, Default)]
pub struct Gold(pub f32);

#[derive(Component, Reflect, Default, Debug)]
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

#[derive(Component, Reflect)]
pub struct PlayerTemplate {
    pub position: Vec2,
    pub health: f32,
}

impl Default for PlayerTemplate {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            health: 100.0,
        }
    }
}

impl PlayerTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        const HEALTH: f32 = 100.0;
        const FACTION: Faction = Faction::Player;

        // Children
        let body = BodyTemplate {
            texture: ImageKey::GreenGnoll,
            offset: vec2(2.0, 11.0),
        }
        .spawn(commands, handle);
        let drop_shadow = DropShadowTemplate::default().spawn(commands, handle);
        let nametag = NametagTemplate {
            offset: vec2(0.0, 26.0),
            name: PLAYER_NAME.to_string(),
        }
        .spawn(commands, handle);

        // Parent
        let mut player = commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(self.position.extend(0.0)),
                ..default()
            },
            MobBundle {
                mob: Mob::player(),
                health: Health(HEALTH),
                ..default()
            }
            .with_faction(FACTION),
            PlayerControl,
        ));
        #[cfg(feature = "debug_mode")]
        player.insert(Name::new("Player"));

        player.add_child(body);
        player.add_child(drop_shadow);
        player.add_child(nametag);
        let player = player.id();

        // Axe hitbox
        HitboxTemplate {
            offset: vec2(10.0, 4.0),
            radius: 7.0,
            damage: 8.0,
            knockback: 5.0,
            faction: FACTION,
            lifetime: f32::INFINITY,
            parent: player,
        }
        .spawn(commands, handle);

        player
    }
}

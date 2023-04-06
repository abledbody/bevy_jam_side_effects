use bevy::{
    math::{vec3, Vec3Swizzles},
    prelude::*,
    window::PrimaryWindow,
};
use leafwing_input_manager::prelude::*;

use super::{Health, Mob, MobBundle, MobInputs};
use crate::{
    asset::{Handles, ImageKey},
    camera::{GameCamera, CAMERA_SCALE},
    combat::Faction,
    hud::{HealthBarTemplate, NametagTemplate},
    mob::{enemy::Alarm, BodyTemplate},
    vfx::DropShadowTemplate,
};

const PLAYER_NAME: &str = "Sai";

#[derive(Component, Reflect, Default)]
pub struct Gold(pub f32);

#[derive(Debug, Copy, Clone, PartialEq, Actionlike)]
pub enum PlayerAction {
    Move,
    Aim,
    Attack,
}

#[derive(Component, Reflect, Default, Debug)]
pub struct PlayerControl;

impl PlayerControl {
    pub fn record_inputs(
        mut player_query: Query<
            (&ActionState<PlayerAction>, &mut MobInputs, &GlobalTransform),
            With<PlayerControl>,
        >,
        primary_window_query: Query<&Window, With<PrimaryWindow>>,
        camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
        mut alarm: ResMut<Alarm>,
    ) {
        let window = primary_window_query.single();
        let (camera, cam_gt) = camera.single();

        for (action_state, mut inputs, mob_gt) in &mut player_query {
            inputs.movement = Vec2::ZERO;
            if action_state.pressed(PlayerAction::Move) {
                if let Some(axis_pair) = action_state.clamped_axis_pair(PlayerAction::Move) {
                    inputs.movement = axis_pair.xy();
                    alarm.current = alarm.current.max(6.5);
                }
            }

            inputs.attack = None;
            let mut direction = None;
            if let Some(axis_pair) = action_state.clamped_axis_pair(PlayerAction::Aim) {
                let axis_pair = axis_pair.xy();
                if axis_pair != Vec2::ZERO {
                    direction = Some(axis_pair);
                }
            }

            if action_state.just_pressed(PlayerAction::Attack) {
                inputs.attack = direction
                    .or_else(|| {
                        window
                            .cursor_position()
                            .and_then(|p| camera.viewport_to_world_2d(cam_gt, p))
                            .map(|p| p - mob_gt.translation().xy())
                    })
                    .map(|d| d.normalize());
            }
        }
    }
}

#[derive(Component, Reflect)]
pub struct PlayerTemplate {
    pub transform: Transform,
    pub health: f32,
}

impl Default for PlayerTemplate {
    fn default() -> Self {
        Self {
            transform: default(),
            health: 200.0,
        }
    }
}

impl PlayerTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        const FACTION: Faction = Faction::Player;

        // Children
        let body = BodyTemplate {
            texture: ImageKey::GnollRed,
            offset: Transform::from_xyz(2.0, 11.0, 0.0),
        }
        .spawn(commands, handle);
        let drop_shadow = DropShadowTemplate::default().spawn(commands, handle);
        let nametag = NametagTemplate {
            offset: Transform::from_xyz(0.0, 26.0, 0.0).with_scale(vec3(
                CAMERA_SCALE,
                CAMERA_SCALE,
                1.0,
            )),
            name: PLAYER_NAME.to_string(),
        }
        .spawn(commands, handle);
        let health_bar = HealthBarTemplate {
            offset: Transform::from_xyz(0.0, -6.0, 0.0),
        }
        .spawn(commands);

        // Parent
        let mut player = commands.spawn((
            SpatialBundle {
                transform: self.transform,
                ..default()
            },
            MobBundle {
                mob: Mob::player(),
                health: Health::full(self.health),
                ..default()
            }
            .with_faction(FACTION),
            PlayerControl,
            InputManagerBundle::<PlayerAction> {
                input_map: InputMap::default()
                    .insert(VirtualDPad::wasd(), PlayerAction::Move)
                    .insert(VirtualDPad::arrow_keys(), PlayerAction::Move)
                    .insert(DualAxis::left_stick(), PlayerAction::Move)
                    .insert(DualAxis::right_stick(), PlayerAction::Aim)
                    .insert(GamepadButtonType::RightTrigger, PlayerAction::Attack)
                    .insert(MouseButton::Left, PlayerAction::Attack)
                    .build(),
                ..default()
            },
        ));
        #[cfg(feature = "debug_mode")]
        player.insert(Name::new("Player"));

        player.add_child(body);
        player.add_child(drop_shadow);
        player.add_child(nametag);
        player.add_child(health_bar);
        let player = player.id();

        player
    }
}

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

use super::Health;
use super::Mob;
use super::MobBundle;
use super::MobInputs;
use crate::common::asset::AudioKey;
use crate::common::asset::Handles;
use crate::common::asset::ImageKey;
use crate::common::camera::GameCamera;
use crate::common::UpdateSet;
use crate::game::combat::Faction;
use crate::game::combat::HurtEffects;
use crate::game::map::Plate;
use crate::game::mob::enemy::Alarm;
use crate::game::mob::Body;
use crate::game::mob::BodyTemplate;
use crate::util::ui::health_bar::HealthBarTemplate;
use crate::util::ui::nametag::NametagTemplate;
use crate::util::vfx::DropShadowTemplate;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerAction>()
            .add_plugins(InputManagerPlugin::<PlayerAction>::default());

        app.register_type::<Playthrough>()
            .init_resource::<Playthrough>()
            .add_systems(
                Update,
                Playthrough::detect_defection.in_set(UpdateSet::Start),
            );

        app.register_type::<PlayerControl>().add_systems(
            Update,
            PlayerControl::record_inputs.in_set(UpdateSet::RecordIntents),
        );
    }
}

const PLAYER_NAME: &str = "Sai";

#[derive(Actionlike, Reflect, Clone, Hash, PartialEq, Eq)]
pub enum PlayerAction {
    Move,
    Aim,
    Attack,
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct Playthrough {
    pub defected: bool,
    pub start_time: f32,
    pub health: Option<f32>,
}

impl Playthrough {
    pub fn detect_defection(
        mut collision_events: EventReader<CollisionEvent>,
        plate_query: Query<(), With<Plate>>,
        player_query: Query<&Children, With<PlayerControl>>,
        mut body_query: Query<&mut Handle<Image>, With<Body>>,
        handle: Res<Handles>,
        mut playthrough: ResMut<Playthrough>,
        mut alarm: ResMut<Alarm>,
        time: Res<Time>,
    ) {
        if playthrough.defected {
            return;
        }
        let Ok(children) = player_query.get_single() else {
            return;
        };

        for &event in collision_events.read() {
            let CollisionEvent::Started(entity1, entity2, _) = event else {
                continue;
            };

            if !plate_query.contains(entity1) && !plate_query.contains(entity2) {
                continue;
            }

            playthrough.defected = true;
            playthrough.start_time = time.elapsed_seconds();
            alarm.increase(0.065);
            for &child in children {
                let Ok(mut body) = body_query.get_mut(child) else {
                    continue;
                };
                *body = handle.image[&ImageKey::GnollBlue].clone();
            }

            return;
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct PlayerControl {
    pub deny_input: bool,
}

impl PlayerControl {
    pub fn record_inputs(
        mut player_query: Query<(
            &ActionState<PlayerAction>,
            &mut MobInputs,
            &GlobalTransform,
            &PlayerControl,
        )>,
        primary_window_query: Query<&Window, With<PrimaryWindow>>,
        camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
    ) {
        let Ok(window) = primary_window_query.get_single() else {
            return;
        };
        let Ok((camera, cam_gt)) = camera.get_single() else {
            return;
        };
        let Ok((action, mut inputs, player_gt, player)) = player_query.get_single_mut() else {
            return;
        };
        if player.deny_input {
            return;
        }

        inputs.movement = Vec2::ZERO;
        if action.pressed(PlayerAction::Move) {
            if let Some(axis_pair) = action.clamped_axis_pair(PlayerAction::Move) {
                inputs.movement = axis_pair.xy();
            }
        }

        let mut aim = None;
        if let Some(axis_pair) = action.clamped_axis_pair(PlayerAction::Aim) {
            let axis_pair = axis_pair.xy();
            if axis_pair != Vec2::ZERO {
                aim = Some(axis_pair);
            }
        }

        inputs.attack = None;
        if action.just_pressed(PlayerAction::Attack) {
            inputs.attack = aim
                .or_else(|| {
                    window
                        .cursor_position()
                        .and_then(|p| camera.viewport_to_world_2d(cam_gt, p))
                        .map(|p| p - player_gt.translation().xy())
                })
                .map(|d| d.normalize());
        }
    }
}

pub struct PlayerTemplate {
    pub transform: Transform,
    pub texture: ImageKey,
    pub current_health: f32,
    pub max_health: f32,
}

impl Default for PlayerTemplate {
    fn default() -> Self {
        Self {
            transform: default(),
            texture: ImageKey::GnollRed,
            current_health: 200.0,
            max_health: 200.0,
        }
    }
}

impl PlayerTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        const FACTION: Faction = Faction::Player;

        // Children
        let body = BodyTemplate {
            texture: self.texture,
            offset: Transform::from_xyz(2.0, 11.0, 0.0),
            walk_sound: Some(handle.audio[&AudioKey::GnollWalk].clone()),
            is_corpse: false,
        }
        .spawn(commands, handle);
        let drop_shadow = DropShadowTemplate::default().spawn(commands, handle);
        let nametag = NametagTemplate {
            offset: Transform::from_xyz(0.0, 26.0, 0.0),
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
                health: Health {
                    current: self.current_health,
                    max: self.max_health,
                },
                ..default()
            }
            .with_faction(FACTION),
            ColliderMassProperties::Mass(5.0),
            HurtEffects {
                sound: Some(handle.audio[&AudioKey::GnollHurt].clone()),
                ..default()
            },
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
            PlayerControl::default(),
        ));
        #[cfg(feature = "dev")]
        player.insert(Name::new("Player"));

        player.add_child(body);
        player.add_child(drop_shadow);
        player.add_child(nametag);
        player.add_child(health_bar);

        player.id()
    }
}

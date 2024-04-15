use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::common::camera::GameCamera;
use crate::common::UpdateSet;
use crate::game::actor::body::Body;
use crate::game::actor::body::BodyTemplate;
use crate::game::actor::health::Health;
use crate::game::actor::intent::ActorIntent;
use crate::game::actor::Actor;
use crate::game::actor::ActorAssets;
use crate::game::actor::ActorBundle;
use crate::game::alarm::Alarm;
use crate::game::combat::Faction;
use crate::game::combat::HurtEffects;
use crate::game::level::plate::Plate;
use crate::util::ui::health_bar::HealthBarTemplate;
use crate::util::ui::nametag::NametagTemplate;
use crate::util::vfx::DropShadowTemplate;
use crate::util::vfx::VfxAssets;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerAction>()
            .add_plugins(InputManagerPlugin::<PlayerAction>::default());

        app.register_type::<Playthrough>()
            .init_resource::<Playthrough>()
            .add_systems(Update, detect_defection.in_set(UpdateSet::Start));

        app.register_type::<PlayerControl>().add_systems(
            Update,
            record_player_intent.in_set(UpdateSet::RecordIntents),
        );
    }
}

const PLAYER_NAME: &str = "Sai";

#[derive(Actionlike, Reflect, Clone, Hash, PartialEq, Eq)]
enum PlayerAction {
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

fn detect_defection(
    mut collision_events: EventReader<CollisionEvent>,
    plate_query: Query<(), With<Plate>>,
    player_query: Query<&Children, With<PlayerControl>>,
    mut body_query: Query<&mut Handle<Image>, With<Body>>,
    mut playthrough: ResMut<Playthrough>,
    mut alarm: ResMut<Alarm>,
    actor_assets: Res<ActorAssets>,
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
            *body = actor_assets.gnoll_blue.clone();
        }

        return;
    }
}

#[derive(Component, Reflect, Default)]
pub struct PlayerControl {
    pub deny_input: bool,
}

fn record_player_intent(
    mut player_query: Query<(
        &ActionState<PlayerAction>,
        &mut ActorIntent,
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
    let Ok((action, mut intent, player_gt, player)) = player_query.get_single_mut() else {
        return;
    };
    if player.deny_input {
        return;
    }

    intent.movement = Vec2::ZERO;
    if action.pressed(&PlayerAction::Move) {
        if let Some(axis_pair) = action.clamped_axis_pair(&PlayerAction::Move) {
            intent.movement = axis_pair.xy();
        }
    }

    let mut aim = None;
    if let Some(axis_pair) = action.clamped_axis_pair(&PlayerAction::Aim) {
        let axis_pair = axis_pair.xy();
        if axis_pair != Vec2::ZERO {
            aim = Some(axis_pair);
        }
    }

    intent.attack = None;
    if action.just_pressed(&PlayerAction::Attack) {
        intent.attack = aim
            .or_else(|| {
                window
                    .cursor_position()
                    .and_then(|p| camera.viewport_to_world_2d(cam_gt, p))
                    .map(|p| p - player_gt.translation().xy())
            })
            .map(|d| d.normalize());
    }
}

pub struct PlayerTemplate {
    pub transform: Transform,
    pub texture: Handle<Image>,
    pub current_health: f32,
    pub max_health: f32,
}

impl Default for PlayerTemplate {
    fn default() -> Self {
        Self {
            transform: default(),
            texture: default(),
            current_health: 200.0,
            max_health: 200.0,
        }
    }
}

impl PlayerTemplate {
    pub fn spawn(
        self,
        commands: &mut Commands,
        actor_assets: &ActorAssets,
        vfx_assets: &VfxAssets,
    ) -> Entity {
        const FACTION: Faction = Faction::Player;

        // Children
        let body = BodyTemplate {
            texture: self.texture,
            offset: Transform::from_xyz(2.0, 11.0, 0.0),
            step_sound: Some(actor_assets.step.clone()),
            is_corpse: false,
        }
        .spawn(commands);
        let drop_shadow = DropShadowTemplate::default().spawn(commands, vfx_assets);
        let nametag = NametagTemplate {
            offset: Transform::from_xyz(0.0, 26.0, 0.0),
            name: PLAYER_NAME.to_string(),
        }
        .spawn(commands);
        let health_bar = HealthBarTemplate {
            offset: Transform::from_xyz(0.0, -6.0, 0.0),
        }
        .spawn(commands);

        // Parent
        commands
            .spawn((
                Name::new("Player"),
                SpatialBundle {
                    transform: self.transform,
                    ..default()
                },
                ActorBundle {
                    actor: Actor::player(),
                    health: Health {
                        current: self.current_health,
                        max: self.max_health,
                    },
                    ..default()
                }
                .with_faction(FACTION),
                ColliderMassProperties::Mass(5.0),
                HurtEffects {
                    sound: Some(actor_assets.hurt.clone()),
                    ..default()
                },
                InputManagerBundle::<PlayerAction> {
                    input_map: InputMap::default()
                        .insert(PlayerAction::Move, VirtualDPad::wasd())
                        .insert(PlayerAction::Move, VirtualDPad::arrow_keys())
                        .insert(PlayerAction::Move, DualAxis::left_stick())
                        .insert(PlayerAction::Aim, DualAxis::right_stick())
                        .insert(PlayerAction::Attack, GamepadButtonType::RightTrigger)
                        .insert(PlayerAction::Attack, MouseButton::Left)
                        .build(),
                    ..default()
                },
                PlayerControl::default(),
            ))
            .add_child(body)
            .add_child(drop_shadow)
            .add_child(nametag)
            .add_child(health_bar)
            .id()
    }
}

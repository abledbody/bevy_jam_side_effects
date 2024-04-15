use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::common::UpdateSet;
use crate::game::combat::COLLISION_GROUP;
use crate::game::combat::PLAYER_HURTBOX_GROUP;
use crate::game::level::gate::Gate;
use crate::game::level::gate::GateAssets;

pub struct PlatePlugin;

impl Plugin for PlatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlateAssets>()
            .init_collection::<PlateAssets>();

        app.register_type::<Plate>()
            .add_systems(Update, activate_plates.in_set(UpdateSet::Start));
    }
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct PlateAssets {
    #[asset(path = "image/interactive/plate_pressed.png")]
    pressed: Handle<Image>,
    #[asset(path = "image/interactive/plate_unpressed.png")]
    unpressed: Handle<Image>,

    #[asset(path = "sound/sfx/button_gate.wav")]
    trigger_gate: Handle<AudioSource>,
}

#[derive(Component, Reflect, Default)]
pub struct Plate {
    gates: Vec<Entity>,
    pressed: bool,
}

fn activate_plates(
    mut collision_events: EventReader<CollisionEvent>,
    mut plate_query: Query<(&mut Plate, &mut Handle<Image>)>,
    mut gate_query: Query<(&mut Gate, &mut Handle<Image>, &mut CollisionGroups), Without<Plate>>,
    gate_assets: Res<GateAssets>,
    plate_assets: Res<PlateAssets>,
    audio: Res<Audio>,
) {
    for &event in collision_events.read() {
        let CollisionEvent::Started(entity1, entity2, _) = event else {
            continue;
        };

        let mut handle_collision = |entity: Entity| {
            let Ok((mut plate, mut plate_image)) = plate_query.get_mut(entity) else {
                return;
            };
            if plate.pressed {
                return;
            }
            plate.pressed = true;
            *plate_image = plate_assets.pressed.clone();

            audio
                .play(plate_assets.trigger_gate.clone())
                .with_volume(0.8);

            for &entity in &plate.gates {
                let Ok((mut gate, mut gate_image, mut gate_groups)) = gate_query.get_mut(entity)
                else {
                    continue;
                };

                gate.open = !gate.open;
                (gate_groups.filters, *gate_image) = if gate.open {
                    (Group::empty(), gate_assets.open.clone())
                } else {
                    (COLLISION_GROUP, gate_assets.closed.clone())
                };
            }
        };

        handle_collision(entity1);
        handle_collision(entity2);
    }
}

pub struct PlateTemplate {
    pub transform: Transform,
    pub gates: Vec<Entity>,
}

impl PlateTemplate {
    pub fn spawn(self, commands: &mut Commands, plate_assets: &PlateAssets) -> Entity {
        commands
            .spawn((
                Name::new("Plate"),
                SpriteBundle {
                    transform: self.transform,
                    texture: plate_assets.unpressed.clone(),
                    ..default()
                },
                Collider::ball(2.0),
                CollisionGroups {
                    memberships: COLLISION_GROUP,
                    filters: PLAYER_HURTBOX_GROUP,
                },
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                Plate {
                    gates: self.gates,
                    ..default()
                },
            ))
            .id()
    }
}

use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_ldtk::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::common::asset::AudioKey;
use crate::common::asset::Handles;
use crate::common::asset::ImageKey;
use crate::common::asset::MapKey;
use crate::common::UpdateSet;
use crate::game::actor::enemy::EnemyTemplate;
use crate::game::actor::player::PlayerControl;
use crate::game::actor::player::PlayerTemplate;
use crate::game::actor::player::Playthrough;
use crate::game::actor::Health;
use crate::game::combat::COLLISION_GROUP;
use crate::game::combat::PLAYER_HURTBOX_GROUP;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, spawn_level_entities);

        app.register_type::<Wall>();

        app.register_type::<Exit>()
            .add_systems(Update, detect_exit.in_set(UpdateSet::Start));

        app.register_type::<Victory>().init_resource::<Victory>();

        app.register_type::<VictorySquare>()
            .add_systems(Update, detect_victory.in_set(UpdateSet::Start));

        app.register_type::<Plate>()
            .add_systems(Update, detect_plate_activation.in_set(UpdateSet::Start));

        app.register_type::<Gate>();
    }
}

pub struct MapTemplate;

impl MapTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        commands
            .spawn((
                Name::new("Map"),
                LdtkWorldBundle {
                    ldtk_handle: handle.map[&MapKey::Game].clone(),
                    ..default()
                },
            ))
            .id()
    }
}

#[derive(Component, Reflect)]
pub struct Wall;

struct WallTemplate {
    transform: Transform,
}

impl WallTemplate {
    fn spawn(self, commands: &mut Commands) -> Entity {
        commands
            .spawn((
                Name::new("Wall"),
                TransformBundle::from_transform(self.transform),
                Collider::cuboid(8.0, 8.0),
                CollisionGroups {
                    memberships: COLLISION_GROUP,
                    filters: COLLISION_GROUP,
                },
                Friction::new(0.0),
                RigidBody::Fixed,
                Wall,
            ))
            .id()
    }
}

#[derive(Component, Reflect)]
pub struct Exit;

fn detect_exit(
    mut collision_events: EventReader<CollisionEvent>,
    mut level_selection: ResMut<LevelSelection>,
    player_query: Query<&Health, With<PlayerControl>>,
    mut playthrough: ResMut<Playthrough>,
    exit_query: Query<(), With<Exit>>,
) {
    let LevelSelection::Indices(idx) = *level_selection else {
        return;
    };
    let Ok(player_health) = player_query.get_single() else {
        return;
    };

    for &event in collision_events.read() {
        let CollisionEvent::Started(entity1, entity2, _) = event else {
            continue;
        };
        if exit_query.contains(entity1) || exit_query.contains(entity2) {
            *level_selection = LevelSelection::Indices(LevelIndices::in_root(idx.level + 1));
            playthrough.health = Some(player_health.current);
            break;
        }
    }
}

struct ExitTemplate {
    transform: Transform,
}

impl ExitTemplate {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        commands
            .spawn((
                Name::new("Exit"),
                TransformBundle::from_transform(self.transform),
                Collider::ball(4.0),
                CollisionGroups {
                    memberships: COLLISION_GROUP,
                    filters: PLAYER_HURTBOX_GROUP,
                },
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                Exit,
            ))
            .id()
    }
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct Victory(pub bool);

#[derive(Component, Reflect)]
pub struct VictorySquare;

fn detect_victory(
    mut collision_events: EventReader<CollisionEvent>,
    victory_query: Query<(), With<VictorySquare>>,
    mut victory: ResMut<Victory>,
) {
    for &event in collision_events.read() {
        let CollisionEvent::Started(entity1, entity2, _) = event else {
            continue;
        };
        if victory_query.contains(entity1) || victory_query.contains(entity2) {
            victory.0 = true;
            break;
        }
    }
}

struct VictorySquareTemplate {
    transform: Transform,
}

impl VictorySquareTemplate {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        commands
            .spawn((
                Name::new("VictorySquare"),
                TransformBundle::from_transform(self.transform),
                Collider::ball(4.0),
                CollisionGroups {
                    memberships: COLLISION_GROUP,
                    filters: PLAYER_HURTBOX_GROUP,
                },
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                VictorySquare,
            ))
            .id()
    }
}

#[derive(Component, Reflect, Default)]
pub struct Plate {
    gates: Vec<Entity>,
    pressed: bool,
}

fn detect_plate_activation(
    mut collision_events: EventReader<CollisionEvent>,
    mut plate_query: Query<(&mut Plate, &mut Handle<Image>)>,
    mut gate_query: Query<(&mut Gate, &mut Handle<Image>, &mut CollisionGroups), Without<Plate>>,
    handle: Res<Handles>,
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
            *plate_image = handle.image[&ImageKey::PlatePressed].clone();

            audio
                .play(handle.audio[&AudioKey::PlateTriggerGate].clone())
                .with_volume(0.8);

            for &entity in &plate.gates {
                let Ok((mut gate, mut gate_image, mut gate_groups)) = gate_query.get_mut(entity)
                else {
                    continue;
                };

                gate.open = !gate.open;
                (gate_groups.filters, *gate_image) = if gate.open {
                    (Group::empty(), handle.image[&ImageKey::GateOpen].clone())
                } else {
                    (COLLISION_GROUP, handle.image[&ImageKey::GateClosed].clone())
                };
            }
        };

        handle_collision(entity1);
        handle_collision(entity2);
    }
}

struct PlateTemplate {
    transform: Transform,
    gates: Vec<Entity>,
}

impl PlateTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        commands
            .spawn((
                Name::new("Plate"),
                SpriteBundle {
                    transform: self.transform,
                    texture: handle.image[&ImageKey::PlateUnpressed].clone(),
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

#[derive(Component, Reflect)]
pub struct Gate {
    open: bool,
}

struct GateTemplate {
    transform: Transform,
    open: bool,
}

impl GateTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        let (filters, texture) = if self.open {
            (Group::empty(), handle.image[&ImageKey::GateOpen].clone())
        } else {
            (COLLISION_GROUP, handle.image[&ImageKey::GateClosed].clone())
        };

        commands
            .spawn((
                Name::new("Gate"),
                SpriteBundle {
                    transform: self.transform,
                    texture,
                    ..default()
                },
                Collider::ball(8.0),
                CollisionGroups {
                    memberships: COLLISION_GROUP,
                    filters,
                },
                Friction::new(0.0),
                RigidBody::Fixed,
                Gate { open: self.open },
            ))
            .id()
    }
}

pub fn spawn_level_entities(
    mut commands: Commands,
    handle: Res<Handles>,
    entity_query: Query<(Entity, &Parent, &Transform, &EntityInstance), Added<EntityInstance>>,
    tile_query: Query<(&Parent, &Transform, &TileEnumTags), Added<TileEnumTags>>,
    playthrough: Res<Playthrough>,
) {
    let mut gate_map = HashMap::new();

    for (_, parent, &transform, instance) in &entity_query {
        let entity = match instance.identifier.as_str() {
            "player" => PlayerTemplate {
                transform,
                current_health: playthrough.health.unwrap_or(200.0),
                texture: if playthrough.defected {
                    ImageKey::GnollBlue
                } else {
                    ImageKey::GnollRed
                },
                ..default()
            }
            .spawn(&mut commands, &handle),
            "enemy" => EnemyTemplate {
                transform,
                ..default()
            }
            .with_random_name()
            .spawn(&mut commands, &handle),
            "corpse" => EnemyTemplate {
                transform,
                ..default()
            }
            .with_random_name()
            .dead()
            .spawn(&mut commands, &handle),
            "open_gate" => {
                let gate = GateTemplate {
                    transform,
                    open: true,
                }
                .spawn(&mut commands, &handle);

                gate_map.insert(&instance.iid, gate);
                gate
            },
            "closed_gate" => {
                let gate = GateTemplate {
                    transform,
                    open: false,
                }
                .spawn(&mut commands, &handle);

                gate_map.insert(&instance.iid, gate);
                gate
            },
            "exit" => ExitTemplate { transform }.spawn(&mut commands),
            "victory" => VictorySquareTemplate { transform }.spawn(&mut commands),
            _ => continue,
        };
        commands.entity(parent.get()).add_child(entity);
    }

    // Spawn plates last so they can link to gates by Entity
    for (entity, parent, &transform, instance) in &entity_query {
        // Despawn marker entity
        commands.entity(entity).despawn_recursive();

        let entity = match instance.identifier.as_str() {
            "plate" => {
                let mut gates = vec![];
                for field in &instance.field_instances {
                    if field.identifier != "gates" {
                        continue;
                    }
                    let FieldValue::EntityRefs(entity_refs) = &field.value else {
                        continue;
                    };

                    gates.extend(
                        entity_refs
                            .iter()
                            .filter_map(|x| x.as_ref())
                            .filter_map(|entity_ref| gate_map.get(&entity_ref.entity_iid)),
                    );
                    break;
                }

                PlateTemplate { transform, gates }.spawn(&mut commands, &handle)
            },
            _ => continue,
        };
        commands.entity(parent.get()).add_child(entity);
    }

    for (parent, &transform, tile) in &tile_query {
        if !tile.tags.iter().any(|s| s.as_str() == "wall") {
            continue;
        }

        let entity = WallTemplate { transform }.spawn(&mut commands);
        commands.entity(parent.get()).add_child(entity);
    }
}

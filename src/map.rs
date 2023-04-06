use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    asset::{Handles, LevelKey},
    combat::{COLLISION_GROUP, PLAYER_HURTBOX_GROUP},
    mob::{enemy::EnemyTemplate, player::PlayerTemplate},
};

pub struct MapTemplate;

impl MapTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        let mut map = commands.spawn(LdtkWorldBundle {
            ldtk_handle: handle.levels[&LevelKey::GameMap].clone(),
            ..default()
        });
        #[cfg(feature = "debug_mode")]
        map.insert(Name::new("Map"));

        map.id()
    }
}

#[derive(Component, Reflect)]
pub struct Wall;

struct WallTemplate {
    transform: Transform,
}

impl WallTemplate {
    fn spawn(self, commands: &mut Commands) -> Entity {
        let mut wall = commands.spawn((
            TransformBundle::from_transform(self.transform),
            Collider::cuboid(8.0, 8.0),
            CollisionGroups {
                memberships: COLLISION_GROUP,
                filters: COLLISION_GROUP,
            },
            Friction::new(0.0),
            RigidBody::Fixed,
            Wall,
        ));
        #[cfg(feature = "debug_mode")]
        wall.insert(Name::new("Wall"));

        wall.id()
    }
}

#[derive(Component, Reflect)]
pub struct Exit;

impl Exit {
    pub fn detect(
        mut collision_events: EventReader<CollisionEvent>,
        mut level_selection: ResMut<LevelSelection>,
        exit_query: Query<(), With<Exit>>,
    ) {
        let LevelSelection::Index(idx) = *level_selection else { return };

        for &event in collision_events.iter() {
            let CollisionEvent::Started ( entity1, entity2, _) = event else { continue };
            if exit_query.contains(entity1) || exit_query.contains(entity2) {
                *level_selection = LevelSelection::Index(idx + 1);
                break;
            }
        }
    }
}

struct ExitTemplate {
    transform: Transform,
}

impl ExitTemplate {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let mut exit = commands.spawn((
            TransformBundle::from_transform(self.transform),
            Collider::cuboid(8.0, 8.0),
            CollisionGroups {
                memberships: COLLISION_GROUP,
                filters: PLAYER_HURTBOX_GROUP,
            },
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
            Exit,
        ));
        #[cfg(feature = "debug_mode")]
        exit.insert(Name::new("Exit"));

        exit.id()
    }
}

// TODO: Determine which gates to toggle
#[derive(Component, Reflect, Default)]
pub struct Plate {
    gates: Vec<Entity>,
    pressed: bool,
}

impl Plate {
    pub fn detect(
        mut collision_events: EventReader<CollisionEvent>,
        mut plate_query: Query<&mut Plate>,
        mut gate_query: Query<(&mut Gate, &mut CollisionGroups)>,
    ) {
        for &event in collision_events.iter() {
            let CollisionEvent::Started ( entity1, entity2, _) = event else { continue };

            let mut handle_collision = |entity: Entity| {
                let Ok(mut plate) = plate_query.get_mut(entity) else { return };
                if plate.pressed {
                    return;
                }
                plate.pressed = true;

                for &entity in &plate.gates {
                    let Ok((mut gate, mut collision_groups)) = gate_query.get_mut(entity) else {
                        continue
                    };

                    gate.open = !gate.open;
                    collision_groups.filters = if gate.open {
                        Group::empty()
                    } else {
                        COLLISION_GROUP
                    };
                }
            };

            handle_collision(entity1);
            handle_collision(entity2);
        }
    }
}

struct PlateTemplate {
    transform: Transform,
    gates: Vec<Entity>,
}

impl PlateTemplate {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let mut plate = commands.spawn((
            TransformBundle::from_transform(self.transform),
            Collider::cuboid(8.0, 8.0),
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
        ));
        #[cfg(feature = "debug_mode")]
        plate.insert(Name::new("Plate"));

        plate.id()
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
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let filters = if self.open {
            Group::empty()
        } else {
            COLLISION_GROUP
        };

        let mut gate = commands.spawn((
            TransformBundle::from_transform(self.transform),
            Collider::cuboid(8.0, 8.0),
            CollisionGroups {
                memberships: COLLISION_GROUP,
                filters,
            },
            Friction::new(0.0),
            RigidBody::Fixed,
            Gate { open: self.open },
        ));
        #[cfg(feature = "debug_mode")]
        gate.insert(Name::new("Gate"));

        gate.id()
    }
}

pub fn spawn_level_entities(
    mut commands: Commands,
    handle: Res<Handles>,
    entity_query: Query<(&Parent, &Transform, &EntityInstance), Added<EntityInstance>>,
    tile_query: Query<(&Parent, &Transform, &TileEnumTags), Added<TileEnumTags>>,
) {
    let mut gate_map = HashMap::new();

    for (parent, &transform, instance) in &entity_query {
        let entity = match instance.identifier.as_str() {
            "player" => PlayerTemplate {
                transform,
                ..default()
            }
            .spawn(&mut commands, &handle),
            "enemy" => EnemyTemplate {
                transform,
                ..default()
            }
            .with_random_name()
            .spawn(&mut commands, &handle),
            "open_gate" => {
                let gate = GateTemplate {
                    transform,
                    open: true,
                }
                .spawn(&mut commands);

                gate_map.insert(&instance.iid, gate);
                gate
            },
            "closed_gate" => {
                let gate = GateTemplate {
                    transform,
                    open: false,
                }
                .spawn(&mut commands);

                gate_map.insert(&instance.iid, gate);
                gate
            },
            "exit" => ExitTemplate { transform }.spawn(&mut commands),
            _ => continue,
        };
        commands.entity(parent.get()).add_child(entity);
    }

    // Spawn plates last so they can link to gates by Entity
    for (parent, &transform, instance) in &entity_query {
        let entity = match instance.identifier.as_str() {
            "plate" => {
                let mut gates = vec![];
                for field in &instance.field_instances {
                    if field.identifier != "gates" {
                        continue;
                    }
                    let FieldValue::EntityRefs(entity_refs) = &field.value else { continue };

                    gates.extend(
                        entity_refs
                            .into_iter()
                            .filter_map(|x| x.as_ref())
                            .filter_map(|entity_ref| gate_map.get(&entity_ref.entity_iid)),
                    );
                    break;
                }

                PlateTemplate { transform, gates }.spawn(&mut commands)
            },
            _ => continue,
        };
        commands.entity(parent.get()).add_child(entity);
    }

    for (parent, &transform, tile) in &tile_query {
        if tile.tags.iter().find(|s| s.as_str() == "wall").is_none() {
            continue;
        }

        let entity = WallTemplate { transform }.spawn(&mut commands);
        commands.entity(parent.get()).add_child(entity);
    }
}

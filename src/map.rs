use bevy::prelude::*;
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
        // If this line returns, there's a bug.
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

pub fn spawn_level_entities(
    mut commands: Commands,
    handle: Res<Handles>,
    entity_query: Query<(&Parent, &Transform, &EntityInstance), Added<EntityInstance>>,
    tile_query: Query<(&Parent, &Transform, &TileEnumTags), Added<TileEnumTags>>,
) {
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
            "exit" => ExitTemplate { transform }.spawn(&mut commands),
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

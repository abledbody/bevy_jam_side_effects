use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::common::UpdateSet;
use crate::game::actor::health::Health;
use crate::game::actor::player::PlayerControl;
use crate::game::actor::player::Playthrough;
use crate::game::combat::COLLISION_GROUP;
use crate::game::combat::PLAYER_HURTBOX_GROUP;

pub struct ExitPlugin;

impl Plugin for ExitPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Exit>()
            .add_systems(Update, detect_exit.in_set(UpdateSet::Start));
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

pub struct ExitTemplate {
    pub transform: Transform,
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

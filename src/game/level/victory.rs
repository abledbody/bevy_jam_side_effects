use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::common::UpdateSet;
use crate::game::combat::COLLISION_GROUP;
use crate::game::combat::PLAYER_HURTBOX_GROUP;

pub struct VictoryPlugin;

impl Plugin for VictoryPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Victory>().init_resource::<Victory>();

        app.register_type::<VictorySquare>()
            .add_systems(Update, detect_victory.in_set(UpdateSet::Start));
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

pub struct VictorySquareTemplate {
    pub transform: Transform,
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

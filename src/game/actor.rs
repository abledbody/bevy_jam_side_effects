pub mod body;
pub mod enemy;
pub mod health;
pub mod intent;
pub mod player;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::game::actor::health::Health;
use crate::game::actor::intent::ActorIntent;
use crate::game::combat::Faction;
use crate::game::combat::COLLISION_GROUP;
use crate::util::animation::facing::Facing;
use crate::util::y_sort::YSort;

pub struct ActorPlugin;

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Actor>();

        app.add_plugins((
            body::BodyPlugin,
            enemy::EnemyPlugin,
            health::HealthPlugin,
            intent::IntentPlugin,
            player::PlayerPlugin,
        ));
    }
}

#[derive(Component, Reflect)]
pub struct Actor {
    pub speed: f32,
    pub acceleration: f32,
    pub brake_deceleration: f32,
    pub idle_threshold: f32,
    pub faction: Faction,
}

impl Actor {
    pub fn player() -> Self {
        Self {
            speed: 110.0,
            acceleration: 900.0,
            brake_deceleration: 1800.0,
            idle_threshold: 10.0,
            faction: Faction::Player,
        }
    }

    pub fn enemy() -> Self {
        Self {
            speed: 80.0,
            acceleration: 900.0,
            brake_deceleration: 1800.0,
            idle_threshold: 10.0,
            faction: Faction::Enemy,
        }
    }
}

impl Default for Actor {
    fn default() -> Self {
        Actor::player()
    }
}

#[derive(Bundle)]
pub struct ActorBundle {
    pub actor: Actor,
    pub actor_intent: ActorIntent,
    pub facing: Facing,
    pub health: Health,
    pub velocity: Velocity,
    pub y_sort: YSort,
    pub rigid_body: RigidBody,
    pub locked_axes: LockedAxes,
    pub friction: Friction,
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub solver_groups: SolverGroups,
}

impl Default for ActorBundle {
    fn default() -> Self {
        Self {
            actor: default(),
            actor_intent: default(),
            facing: default(),
            health: Health::full(100.0),
            y_sort: YSort,
            velocity: default(),
            rigid_body: default(),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            friction: Friction::new(0.0),
            collider: Collider::ball(6.0),
            collision_groups: CollisionGroups {
                memberships: COLLISION_GROUP,
                filters: COLLISION_GROUP,
            },
            solver_groups: SolverGroups {
                memberships: COLLISION_GROUP,
                filters: COLLISION_GROUP,
            },
        }
    }
}

impl ActorBundle {
    pub fn with_faction(mut self, faction: Faction) -> Self {
        let hurtbox_groups = faction.hurtbox_groups();
        self.collision_groups.memberships |= hurtbox_groups.memberships;
        self.collision_groups.filters |= hurtbox_groups.filters;
        self.actor.faction = faction;
        self
    }
}

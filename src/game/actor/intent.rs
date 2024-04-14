use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::common::UpdateSet;
use crate::game::actor::body::AttackAnimation;
use crate::game::actor::Actor;
use crate::util::animation::facing::Facing;
use crate::util::math::MoveTowards;

pub struct IntentPlugin;

impl Plugin for IntentPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ActorIntent>().add_systems(
            Update,
            (
                set_actor_facing.in_set(UpdateSet::ApplyIntents),
                apply_actor_movement.in_set(UpdateSet::ApplyIntents),
            ),
        );
    }
}

#[derive(Component, Reflect, Default)]
pub struct ActorIntent {
    pub movement: Vec2,
    pub attack: Option<Vec2>,
}

fn set_actor_facing(
    mut actor_query: Query<(&ActorIntent, Option<&Children>, &mut Facing)>,
    attack_animation_query: Query<&AttackAnimation>,
) {
    for (intent, children, mut facing) in &mut actor_query {
        if intent.movement.x == 0.0 && intent.attack.is_none() {
            continue;
        }

        *facing = if intent.attack.map(|dir| dir.x < 0.0).unwrap_or_else(|| {
            children
                .into_iter()
                .flatten()
                .filter_map(|&child| {
                    attack_animation_query
                        .get(child)
                        .ok()
                        .filter(|anim| anim.t < 1.0)
                        .map(|anim| anim.x_sign < 0.0)
                })
                .next()
                .unwrap_or(intent.movement.x < 0.0)
        }) {
            Facing::Left
        } else {
            Facing::Right
        };
    }
}

fn apply_actor_movement(
    mut actor_query: Query<(&Actor, &mut Velocity, Option<&ActorIntent>)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (actor, mut velocity, intent) in &mut actor_query {
        let (intent_direction, intent_magnitude) = if let Some(intent) = intent {
            (
                intent.movement.normalize_or_zero(),
                intent.movement.length().min(1.0),
            )
        } else {
            (Vec2::ZERO, 0.0)
        };

        let acceleration = if intent_direction.dot(velocity.linvel) <= 0.0 {
            actor.brake_deceleration
        } else {
            actor.acceleration
        };

        let target_velocity = intent_direction * intent_magnitude * actor.speed;
        velocity.linvel = velocity
            .linvel
            .move_towards(target_velocity, acceleration * dt);
    }
}

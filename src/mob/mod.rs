use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{AttackAnimation, Facing, FlinchAnimation, Offset, WalkAnimation},
    asset::{AudioKey, Handles, ImageKey},
    combat::{Faction, COLLISION_GROUP},
    math::MoveTowards,
    util::ZRampByY,
};

pub mod enemy;
pub mod player;

#[derive(Component, Reflect, Debug)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn full(max: f32) -> Self {
        Self { current: max, max }
    }
}

#[derive(Component, Reflect, Debug)]
pub struct Mob {
    speed: f32,
    acceleration: f32,
    brake_deceleration: f32,
    idle_threshold: f32,
    pub faction: Faction,
}

impl Mob {
    pub fn set_facing(
        mut mob_query: Query<(&MobInputs, Option<&Children>, &mut Facing)>,
        attack_animation_query: Query<&AttackAnimation>,
    ) {
        for (inputs, children, mut facing) in &mut mob_query {
            if inputs.movement.x == 0.0 && inputs.attack.is_none() {
                continue;
            }

            *facing = if children
                .map(|children| {
                    children
                        .iter()
                        .filter_map(|&child| {
                            attack_animation_query
                                .get(child)
                                .ok()
                                .filter(|anim| anim.t < 1.0)
                                .map(|anim| anim.x_sign < 0.0)
                        })
                        .next()
                })
                .flatten()
                .unwrap_or_else(|| inputs.movement.x < 0.0)
            {
                Facing::Left
            } else {
                Facing::Right
            };
        }
    }

    pub fn apply_movement(
        mut mob_query: Query<(&Mob, &mut Velocity, Option<&MobInputs>)>,
        time: Res<Time>,
    ) {
        let dt = time.delta_seconds();
        for (mob, mut velocity, inputs) in &mut mob_query {
            let (input_direction, input_magnitude) = if let Some(inputs) = inputs {
                (
                    inputs.movement.normalize_or_zero(),
                    inputs.movement.length().min(1.0),
                )
            } else {
                (Vec2::ZERO, 0.0)
            };

            let mut acceleration = mob.acceleration;
            if input_direction.dot(velocity.linvel) < 0.0 {
                acceleration = mob.brake_deceleration;
            }

            let target_velocity = input_direction * input_magnitude * mob.speed;
            velocity.linvel = velocity
                .linvel
                .move_towards(target_velocity, acceleration * dt);
        }
    }

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

impl Default for Mob {
    fn default() -> Self {
        Mob::player()
    }
}

#[derive(Bundle, Reflect, Debug)]
pub struct MobBundle {
    pub mob: Mob,
    pub mob_inputs: MobInputs,
    pub facing: Facing,
    pub health: Health,
    pub velocity: Velocity,
    pub z_ramp_by_y: ZRampByY,
    pub rigid_body: RigidBody,
    pub locked_axes: LockedAxes,
    pub friction: Friction,
    #[reflect(ignore)]
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub solver_groups: SolverGroups,
}

impl Default for MobBundle {
    fn default() -> Self {
        Self {
            mob: default(),
            mob_inputs: default(),
            facing: default(),
            health: Health::full(100.0),
            z_ramp_by_y: ZRampByY,
            velocity: default(),
            rigid_body: default(),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            friction: Friction::new(0.0),
            collider: Collider::ball(5.0),
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

impl MobBundle {
    pub fn with_faction(mut self, faction: Faction) -> Self {
        let hurtbox_groups = faction.hurtbox_groups();
        self.collision_groups.memberships |= hurtbox_groups.memberships;
        self.collision_groups.filters |= hurtbox_groups.filters;
        self.mob.faction = faction;
        self
    }
}

#[derive(Component, Reflect, Default, Debug)]
pub struct MobInputs {
    pub movement: Vec2,
    pub attack: Option<Vec2>,
}

pub struct BodyTemplate {
    texture: ImageKey,
    offset: Transform,
}

impl BodyTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        let mut body = commands.spawn((
            SpriteBundle {
                texture: handle.image[&self.texture].clone(),
                ..default()
            },
            Offset(self.offset),
            WalkAnimation {
                sound: Some(handle.audio[&AudioKey::GnollWalk].clone()),
                ..default()
            },
            AttackAnimation::default(),
            FlinchAnimation::default(),
        ));
        #[cfg(feature = "debug_mode")]
        body.insert(Name::new("Body"));

        body.id()
    }
}

#[derive(Component, Reflect)]
pub struct DeadBody;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{Facing, Offset, WalkAnimation},
    asset::{AudioKey, Handles, ImageKey},
    combat::{Faction, COLLISION_GROUP},
    game::TIME_STEP,
    math::MoveTowards,
    util::ZRampByY,
};

pub mod enemy;
pub mod player;

#[derive(Debug, Component, Reflect)]
pub struct Health(pub f32);

#[derive(Debug, Component, Reflect)]
pub struct Mob {
    speed: f32,
    acceleration: f32,
    brake_deceleration: f32,
    idle_threshold: f32,
}

impl Mob {
    pub fn set_facing(mut mob_query: Query<(&Mob, &MobInputs, &Velocity, &mut Facing)>) {
        for (mob, mob_inputs, velocity, mut facing) in &mut mob_query {
            if mob_inputs.movement.x == 0.0 {
                continue;
            }

            let idle = velocity.linvel.x.abs() < mob.idle_threshold;
            let input_left = mob_inputs.movement.x < 0.0;
            let move_left = velocity.linvel.x < 0.0;
            *facing = if (idle && input_left) || (!idle && move_left) {
                Facing::Left
            } else {
                Facing::Right
            };
        }
    }

    pub fn apply_input(mut mob_query: Query<(&Mob, &mut Velocity, &MobInputs)>) {
        for (mob, mut velocity, mob_inputs) in &mut mob_query {
            let input_direction = mob_inputs.movement.normalize_or_zero();
            let input_magnitude = mob_inputs.movement.length().min(1.0);

            let mut acceleration = mob.acceleration;
            if input_direction.dot(velocity.linvel) < 0.0 {
                acceleration = mob.brake_deceleration;
            }

            let target_velocity = input_direction * input_magnitude * mob.speed;
            velocity.linvel = velocity
                .linvel
                .move_towards(target_velocity, acceleration * TIME_STEP);
        }
    }

    pub fn player() -> Self {
        Self {
            speed: 110.0,
            acceleration: 900.0,
            brake_deceleration: 1800.0,
            idle_threshold: 10.0,
        }
    }
}

impl Default for Mob {
    fn default() -> Self {
        Mob::player()
    }
}

#[derive(Debug, Bundle, Reflect)]
pub struct MobBundle {
    pub mob: Mob,
    pub mob_inputs: MobInputs,
    pub facing: Facing,
    pub health: Health,
    pub velocity: Velocity,
    pub z_ramp_by_y: ZRampByY,
    pub rigid_body: RigidBody,
    pub locked_axes: LockedAxes,
    #[reflect(ignore)]
    pub collider: Collider,
    pub collision_groups: CollisionGroups,
    pub solver_groups: SolverGroups,
}

impl Default for MobBundle {
    fn default() -> Self {
        Self {
            mob: Mob::default(),
            mob_inputs: MobInputs::default(),
            facing: Facing::default(),
            health: Health(100.0),
            z_ramp_by_y: ZRampByY,
            velocity: Velocity::default(),
            rigid_body: RigidBody::default(),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            collider: Collider::ball(7.0),
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
        self
    }
}

#[derive(Debug, Component, Reflect, Default)]
pub struct MobInputs {
    pub movement: Vec2,
}

pub struct BodyTemplate {
    texture: ImageKey,
    offset: Vec2,
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
                air_time: 0.18,
                height: 3.0,
                base_height: self.offset.y,
                sound: Some(handle.audio[&AudioKey::GnollWalk].clone()),
                ..default()
            },
        ));
        #[cfg(feature = "debug_mode")]
        body.insert(Name::new("Body"));

        body.id()
    }
}

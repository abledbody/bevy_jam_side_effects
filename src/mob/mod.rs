use bevy::{prelude::*, math::vec2};
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{AttackAnimation, Facing, Offset, WalkAnimation},
    asset::{AudioKey, Handles, ImageKey},
    combat::{Faction, COLLISION_GROUP},
    math::MoveTowards,
    util::ZRampByY,
};

pub mod enemy;
pub mod player;

#[derive(Debug, Component, Reflect)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn full(max: f32) -> Self {
        Self { current: max, max }
    }
}

#[derive(Debug, Component, Reflect)]
pub struct Mob {
    speed: f32,
    acceleration: f32,
    brake_deceleration: f32,
    idle_threshold: f32,
    pub faction: Faction,
}

impl Mob {
    pub fn set_facing(mut mob_query: Query<(&MobInputs, &mut Facing)>) {
        for (mob_inputs, mut facing) in &mut mob_query {
			if mob_inputs.movement.x == 0.0 && mob_inputs.attack.is_none() {continue;}

            let input_left = mob_inputs.movement.x < 0.0;
			let attack_left = mob_inputs.attack.map(|dir| dir.x < 0.0).unwrap_or(false);

            *facing = if input_left || attack_left
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
        for (mob, mut velocity, mob_inputs) in &mut mob_query {
            let (input_direction, input_magnitude) = if let Some(mob_inputs) = mob_inputs {
                (
                    mob_inputs.movement.normalize_or_zero(),
                    mob_inputs.movement.length().min(1.0),
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
    pub friction: Friction,
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
            health: Health::full(100.0),
            z_ramp_by_y: ZRampByY,
            velocity: Velocity::default(),
            rigid_body: RigidBody::default(),
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

#[derive(Debug, Component, Reflect, Default)]
pub struct MobInputs {
    pub movement: Vec2,
    pub attack: Option<Vec2>,
}

impl MobInputs {
    pub fn animate_attack(
        mob_query: Query<(&MobInputs, &Children)>,
        mut animation_query: Query<&mut AttackAnimation>,
    ) {
        for (mob_inputs, children) in &mob_query {
            if let Some(attack_direction) = mob_inputs.attack {
				let attack_direction = vec2(attack_direction.x.abs(), attack_direction.y);
                for &child in children {
                    if let Ok(mut anim) = animation_query.get_mut(child) {
                        anim.t = 0.0;
						anim.direction = attack_direction;
                    }
                }
            }
        }
    }
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
            Offset {
                pos: self.offset,
                ..default()
            },
            WalkAnimation {
                air_time: 0.18,
                height: 3.0,
                sound: Some(handle.audio[&AudioKey::GnollWalk].clone()),
                ..default()
            },
            AttackAnimation { ..default() },
        ));
        #[cfg(feature = "debug_mode")]
        body.insert(Name::new("Body"));

        body.id()
    }
}

#[derive(Component)]
pub struct DeadBody;

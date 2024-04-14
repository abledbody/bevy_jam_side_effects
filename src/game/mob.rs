pub mod animation;
pub mod enemy;
pub mod player;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::common::asset::Handles;
use crate::common::asset::ImageKey;
use crate::common::UpdateSet;
use crate::game::combat::Faction;
use crate::game::combat::COLLISION_GROUP;
use crate::game::mob::animation::AttackAnimation;
use crate::game::mob::animation::DeathAnimation;
use crate::game::mob::animation::FlinchAnimation;
use crate::game::mob::animation::WalkAnimation;
use crate::util::animation::facing::Facing;
use crate::util::animation::offset::Offset;
use crate::util::math::MoveTowards;
use crate::util::y_sort::YSort;

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Health>();

        app.register_type::<Mob>().add_systems(
            Update,
            (
                Mob::set_facing.in_set(UpdateSet::ApplyIntents),
                Mob::apply_movement.in_set(UpdateSet::ApplyIntents),
            ),
        );

        app.register_type::<MobInputs>();

        app.register_type::<Body>();

        app.add_plugins((
            animation::AnimationPlugin,
            enemy::EnemyPlugin,
            player::PlayerPlugin,
        ));
    }
}

#[derive(Component, Reflect)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn full(max: f32) -> Self {
        Self { current: max, max }
    }
}

#[derive(Component, Reflect)]
pub struct Mob {
    pub speed: f32,
    pub acceleration: f32,
    pub brake_deceleration: f32,
    pub idle_threshold: f32,
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

            *facing = if inputs.attack.map(|dir| dir.x < 0.0).unwrap_or_else(|| {
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
                    .unwrap_or(inputs.movement.x < 0.0)
            }) {
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

            let acceleration = if input_direction.dot(velocity.linvel) <= 0.0 {
                mob.brake_deceleration
            } else {
                mob.acceleration
            };

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

#[derive(Bundle)]
pub struct MobBundle {
    pub mob: Mob,
    pub mob_inputs: MobInputs,
    pub facing: Facing,
    pub health: Health,
    pub velocity: Velocity,
    pub z_ramp_by_y: YSort,
    pub rigid_body: RigidBody,
    pub locked_axes: LockedAxes,
    pub friction: Friction,
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
            z_ramp_by_y: YSort,
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

impl MobBundle {
    pub fn with_faction(mut self, faction: Faction) -> Self {
        let hurtbox_groups = faction.hurtbox_groups();
        self.collision_groups.memberships |= hurtbox_groups.memberships;
        self.collision_groups.filters |= hurtbox_groups.filters;
        self.mob.faction = faction;
        self
    }
}

#[derive(Component, Reflect, Default)]
pub struct MobInputs {
    pub movement: Vec2,
    pub attack: Option<Vec2>,
}

#[derive(Component, Reflect)]
pub struct Body;

pub struct BodyTemplate {
    texture: ImageKey,
    offset: Transform,
    walk_sound: Option<Handle<AudioSource>>,
    is_corpse: bool,
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
                sound: self.walk_sound,
                ..default()
            },
            AttackAnimation::default(),
            FlinchAnimation::default(),
            Body,
        ));
        if self.is_corpse {
            body.insert(DeathAnimation::default());
        }
        #[cfg(feature = "dev")]
        body.insert(Name::new("Body"));

        body.id()
    }
}

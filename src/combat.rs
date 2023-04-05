use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{DeathAnimation, Lifetime, WalkAnimation},
    asset::{AudioKey, Handles},
    mob::{
        enemy::Alarm,
        player::{Gold, PlayerControl},
        DeadBody,
        Health,
        Mob,
        MobInputs,
    },
};

pub const COLLISION_GROUP: Group = Group::GROUP_1;
pub const HITBOX_GROUP: Group = Group::GROUP_2;
pub const PLAYER_HURTBOX_GROUP: Group = Group::GROUP_3;
pub const ENEMY_HURTBOX_GROUP: Group = Group::GROUP_4;

#[derive(Copy, Clone, Debug, Reflect)]
pub enum Faction {
    Player,
    Enemy,
}

impl Faction {
    pub fn hitbox_groups(&self) -> CollisionGroups {
        CollisionGroups {
            memberships: HITBOX_GROUP,
            filters: match self {
                Faction::Player => ENEMY_HURTBOX_GROUP,
                Faction::Enemy => PLAYER_HURTBOX_GROUP,
            },
        }
    }

    pub fn hurtbox_groups(&self) -> CollisionGroups {
        CollisionGroups {
            memberships: match self {
                Faction::Player => PLAYER_HURTBOX_GROUP,
                Faction::Enemy => ENEMY_HURTBOX_GROUP,
            },
            filters: HITBOX_GROUP,
        }
    }
}

pub struct HitboxTemplate {
    pub position: Vec3,
    pub direction: Vec2,
    pub radius: f32,
    pub damage: f32,
    pub knockback: f32,
    pub faction: Faction,
}

impl HitboxTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        let mut hitbox = commands.spawn((
            TransformBundle {
                local: Transform {
                    translation: self.position,
                    ..default()
                },
                ..default()
            },
            Collider::ball(self.radius),
            Sensor,
            self.faction.hitbox_groups(),
            ActiveEvents::COLLISION_EVENTS,
            HitBox {
                damage: self.damage,
                knockback: self.knockback * self.direction,
                hit: false,
                hit_sound: Some(handle.audio[&AudioKey::PlayerAttack4].clone()),
                miss_sound: Some(handle.audio[&AudioKey::PlayerAttackMiss].clone()),
            },
        ));
        #[cfg(feature = "debug_mode")]
        hitbox.insert(Name::new("Hitbox"));

        hitbox.id()
    }
}

pub struct HitEvent {
    knockback: Vec2,
    hitbox: Entity,
    target: Entity,
}

impl HitEvent {
    pub fn detect(
        mut collision_events: EventReader<CollisionEvent>,
        mut hit_events: EventWriter<HitEvent>,
        hit_query: Query<&HitBox>,
    ) {
        for &event in collision_events.iter() {
            let CollisionEvent::Started(entity1, entity2, _) = event else {
                continue
            };

            let mut handle_collision = |hitbox: Entity, target: Entity| {
                let Ok(hit_effects) = hit_query.get(hitbox) else { return };
                hit_events.send(HitEvent {
                    knockback: hit_effects.knockback,
                    hitbox,
                    target,
                });
            };

            handle_collision(entity1, entity2);
            handle_collision(entity2, entity1);
        }
    }
}

#[derive(Component, Reflect)]
pub struct HitBox {
    damage: f32,
    knockback: Vec2,
    hit: bool,
    hit_sound: Option<Handle<AudioSource>>,
    miss_sound: Option<Handle<AudioSource>>,
}

impl HitBox {
    pub fn apply(
        mut hit_events: EventReader<HitEvent>,
        mut death_events: EventWriter<DeathEvent>,
        mut hitbox_query: Query<&mut HitBox>,
        mut health_query: Query<&mut Health>,
        mut velocity_query: Query<&mut Velocity>,
        audio: Res<Audio>,
    ) {
        for &HitEvent {
            knockback,
            hitbox,
            target,
        } in hit_events.iter()
        {
            let Ok(mut hitbox) = hitbox_query.get_mut(hitbox) else { return };
            hitbox.hit = true;

            if let Some(sound) = &hitbox.hit_sound {
                audio.play(sound.clone());
            }

            // Damage
            if let Ok(mut health) = health_query.get_mut(target) {
                if 0.0 < health.current && health.current <= hitbox.damage {
                    death_events.send(DeathEvent(target));
                }
                health.current -= hitbox.damage;
            }

            // Knockback
            if let Ok(mut velocity) = velocity_query.get_mut(target) {
                let scale = 40.0;
                velocity.linvel = knockback * scale;
            }
        }
    }

    pub fn cleanup(
        mut commands: Commands,
        hit_effects_query: Query<(Entity, &HitBox)>,
        audio: Res<Audio>,
    ) {
        for (entity, hitbox) in &hit_effects_query {
            if !hitbox.hit {
                if let Some(sound) = &hitbox.miss_sound {
                    audio.play(sound.clone());
                }
            }

            commands.entity(entity).despawn_recursive();
        }
    }

    pub fn spawn_from_inputs(
        mut commands: Commands,
        mob_query: Query<(&Mob, &Transform, &MobInputs)>,
        handle: Res<Handles>,
    ) {
        for (mob, transform, inputs) in &mob_query {
            if let Some(dir) = inputs.attack {
                // Make the hitbox offset slightly ovular
                let ovular_dir = Quat::from_axis_angle(Vec3::X, 0.5 * PI * 0.3) * dir.extend(0.0);
                HitboxTemplate {
                    position: transform.translation + 15.0 * ovular_dir,
                    direction: dir,
                    radius: 7.0,
                    damage: 8.0,
                    knockback: 5.0,
                    faction: mob.faction,
                }
                .spawn(&mut commands, &handle);
            }
        }
    }
}

pub struct DeathEvent(pub Entity);

#[derive(Component, Reflect)]
pub struct DeathEffects {
    pub reward_gold: f32,
    pub increase_alarm: f32,
    // TODO: Animation, sound effect
}

impl Default for DeathEffects {
    fn default() -> Self {
        Self {
            reward_gold: 10.0,
            increase_alarm: 5.0,
        }
    }
}

impl DeathEffects {
    pub fn apply(
        mut commands: Commands,
        mut death_events: EventReader<DeathEvent>,
        death_effects_query: Query<&DeathEffects>,
        mut alarm: ResMut<Alarm>,
        mut player_query: Query<&mut Gold, With<PlayerControl>>,
        children_query: Query<&Children>,
        animation_query: Query<(), With<WalkAnimation>>, // And you can use animation_query.contains(child)
    ) {
        for &DeathEvent(entity) in death_events.iter() {
            // Turn into a dead body
            commands
                .entity(entity)
                .insert((DeadBody, Lifetime(10.0)))
                .remove::<MobInputs>();

            if let Ok(children) = children_query.get(entity) {
                for &child in children {
                    if animation_query.contains(child) {
                        commands.entity(child).insert(DeathAnimation::default());
                    }
                }
            }

            let Ok(death_effects) = death_effects_query.get(entity) else {
                continue
            };

            // Reward gold
            for mut player_gold in &mut player_query {
                player_gold.0 += death_effects.reward_gold;
            }

            // Increase alarm
            alarm.increase(death_effects.increase_alarm);
        }
    }
}

use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{DeathAnimation, FlinchAnimation, WalkAnimation},
    asset::{AudioKey, Handles},
    mob::{enemy::Alarm, Health, Mob, MobInputs},
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
            HitEffects {
                damage: self.damage,
                knockback: self.knockback * self.direction,
                success_sound: Some(handle.audio[&AudioKey::GnollAttackHit].clone()),
                failure_sound: Some(handle.audio[&AudioKey::GnollAttackMiss].clone()),
                ..default()
            },
        ));
        #[cfg(feature = "debug_mode")]
        hitbox.insert(Name::new("Hitbox"));

        hitbox.id()
    }
}

pub struct HitEvent {
    pub hitbox: Entity,
    pub hurtbox: Entity,
}

impl HitEvent {
    pub fn detect(
        mut collision_events: EventReader<CollisionEvent>,
        mut hit_events: EventWriter<HitEvent>,
        hit_query: Query<&HitEffects>,
    ) {
        for &event in collision_events.iter() {
            let CollisionEvent::Started(entity1, entity2, _) = event else {
                continue
            };

            let mut handle_collision = |hitbox: Entity, target: Entity| {
                if !hit_query.contains(hitbox) {
                    return;
                }
                hit_events.send(HitEvent {
                    hitbox,
                    hurtbox: target,
                });
            };

            handle_collision(entity1, entity2);
            handle_collision(entity2, entity1);
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct HitEffects {
    pub damage: f32,
    pub knockback: Vec2,
    pub success: bool,
    pub success_sound: Option<Handle<AudioSource>>,
    pub failure_sound: Option<Handle<AudioSource>>,
}

impl HitEffects {
    pub fn apply(
        mut hit_events: EventReader<HitEvent>,
        mut death_events: EventWriter<DeathEvent>,
        mut hit_effects: Query<&mut HitEffects>,
        mut health_query: Query<&mut Health>,
        mut velocity_query: Query<&mut Velocity>,
        child_query: Query<&Children>,
        mut flinch_query: Query<&mut FlinchAnimation>,
        audio: Res<Audio>,
    ) {
        for &HitEvent { hitbox, hurtbox } in hit_events.iter() {
            let Ok(mut hit) = hit_effects.get_mut(hitbox) else { return };

            if let Some(sound) = &hit.success_sound {
                audio.play(sound.clone()).with_volume(0.4);
            }
            hit.success = true;

            // Damage
            if let Ok(mut health) = health_query.get_mut(hurtbox) {
                if 0.0 < health.current && health.current <= hit.damage {
                    death_events.send(DeathEvent(hurtbox));
                }
                health.current -= hit.damage;
            }

            // Knockback
            if let Ok(mut velocity) = velocity_query.get_mut(hurtbox) {
                let scale = 40.0;
                velocity.linvel = hit.knockback * scale;
            }

            // Flinch
            if let Ok(children) = child_query.get(hurtbox) {
                for child in children.iter() {
                    if let Ok(mut flinch) = flinch_query.get_mut(*child) {
                        flinch.trigger(hit.knockback.normalize_or_zero());
                    }
                }
            }
        }
    }

    pub fn cleanup(
        mut commands: Commands,
        hit_effects_query: Query<(Entity, &HitEffects)>,
        audio: Res<Audio>,
    ) {
        for (entity, effects) in &hit_effects_query {
            if !effects.success {
                if let Some(sound) = &effects.failure_sound {
                    audio.play(sound.clone()).with_volume(0.4);
                }
            }

            commands.entity(entity).despawn_recursive();
        }
    }

    pub fn spawn_from_inputs(
        mut commands: Commands,
        mob_query: Query<(&Mob, &GlobalTransform, &MobInputs)>,
        handle: Res<Handles>,
    ) {
        for (mob, transform, inputs) in &mob_query {
            let Some(direction) = inputs.attack else {
                continue
            };

            // Make the hitbox offset slightly ovular
            let ovular_dir = Quat::from_rotation_x(0.5 * PI * 0.3) * direction.extend(0.0);
            let radius = 12.0;
            let distance = radius;

            HitboxTemplate {
                position: transform.translation() + distance * ovular_dir,
                direction,
                radius,
                damage: 8.0,
                knockback: 6.0,
                faction: mob.faction,
            }
            .spawn(&mut commands, &handle);
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct HurtEffects {
    pub increase_alarm: f32,
    pub sound: Option<Handle<AudioSource>>,
}

impl HurtEffects {
    pub fn apply(
        mut hit_events: EventReader<HitEvent>,
        hurt_effects_query: Query<&HurtEffects>,
        mut alarm: ResMut<Alarm>,
        audio: Res<Audio>,
    ) {
        for &HitEvent { hurtbox, .. } in hit_events.iter() {
            let Ok(hurt) = hurt_effects_query.get(hurtbox) else { continue };

            // Increase alarm
            alarm.increase(hurt.increase_alarm);

            // Play sound
            if let Some(sound) = &hurt.sound {
                audio.play(sound.clone()).with_volume(0.4);
            }
        }
    }
}

pub struct DeathEvent(pub Entity);

#[derive(Component, Reflect, Default)]
pub struct DeathEffects {
    pub increase_alarm: f32,
}

impl DeathEffects {
    pub fn apply(
        mut commands: Commands,
        mut death_events: EventReader<DeathEvent>,
        death_effects_query: Query<&DeathEffects>,
        mut hurt_effects_query: Query<&mut HurtEffects>,
        mut mob_query: Query<&mut Mob>,
        mut alarm: ResMut<Alarm>,
        children_query: Query<&Children>,
        animation_query: Query<(), With<WalkAnimation>>, // And you can use animation_query.contains(child)
    ) {
        for &DeathEvent(entity) in death_events.iter() {
            // Turn into a dead body
            commands
                .entity(entity)
                .insert(ColliderMassProperties::Mass(25.0))
                .remove::<MobInputs>();
            if let Ok(mut hurt_effects) = hurt_effects_query.get_mut(entity) {
                hurt_effects.increase_alarm = 0.0;
            }
            if let Ok(mut mob) = mob_query.get_mut(entity) {
                mob.brake_deceleration = 700.0;
            }

            // Play death animation
            if let Ok(children) = children_query.get(entity) {
                for &child in children {
                    if animation_query.contains(child) {
                        commands.entity(child).insert(DeathAnimation::default());
                    }
                }
            }

            let Ok(death) = death_effects_query.get(entity) else {
                continue
            };

            // Increase alarm
            alarm.increase(death.increase_alarm);
        }
    }
}

use bevy::{
    math::{vec2, Vec3Swizzles},
    prelude::*,
};
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{DeathAnimation, Lifetime, Offset, VirtualParent, WalkAnimation},
    asset::{AudioKey, Handles},
    mob::{
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
    pub offset: Vec2,
    pub radius: f32,
    pub damage: f32,
    pub knockback: f32,
    pub faction: Faction,
    pub parent: Entity,
}

impl HitboxTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        let mut hitbox = commands.spawn((
            Offset {
                pos: self.offset,
                ..default()
            },
            TransformBundle::default(),
            Collider::ball(self.radius),
            Sensor,
            self.faction.hitbox_groups(),
            ActiveEvents::COLLISION_EVENTS,
            HitEffects {
                damage: self.damage,
                knockback: self.knockback,
                sound: Some(handle.audio[&AudioKey::PlayerAttack2].clone()),
            },
            VirtualParent(self.parent),
        ));
        #[cfg(feature = "debug_mode")]
        hitbox.insert(Name::new("Hitbox"));

        hitbox.id()
    }
}

pub struct HitEvent {
    actor: Entity,
    hitbox: Entity,
    target: Entity,
}

impl HitEvent {
    pub fn detect(
        mut collision_events: EventReader<CollisionEvent>,
        mut hit_events: EventWriter<HitEvent>,
        hit_query: Query<&VirtualParent, With<HitEffects>>,
    ) {
        for &event in collision_events.iter() {
            let CollisionEvent::Started(entity1, entity2, _) = event else {
                continue
            };

            let mut handle_collision = |hitbox: Entity, target: Entity| {
                let Ok(&VirtualParent(actor)) = hit_query.get(hitbox) else { return };
                hit_events.send(HitEvent {
                    actor,
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
pub struct HitEffects {
    damage: f32,
    knockback: f32,
    sound: Option<Handle<AudioSource>>,
}

impl HitEffects {
    pub fn apply(
        mut hit_events: EventReader<HitEvent>,
        mut death_events: EventWriter<DeathEvent>,
        hit_effects_query: Query<&HitEffects>,
        mut health_query: Query<&mut Health>,
        mut velocity_query: Query<&mut Velocity>,
        transform_query: Query<&GlobalTransform>,
        audio: Res<Audio>,
    ) {
        for &HitEvent {
            actor,
            hitbox,
            target,
        } in hit_events.iter()
        {
            let Ok(effect) = hit_effects_query.get(hitbox) else { return };

            if let Some(sound) = &effect.sound {
                audio.play(sound.clone());
            }

            // Damage
            if let Ok(mut health) = health_query.get_mut(target) {
                if 0.0 < health.current && health.current <= effect.damage {
                    death_events.send(DeathEvent(target));
                }
                health.current -= effect.damage;
            }

            // Knockback
            if let Ok(mut velocity) = velocity_query.get_mut(target) {
                let Ok(actor_transform) = transform_query.get(actor) else {
                    return
                };
                let Ok(target_transform) = transform_query.get(target) else {
                    return
                };

                let scale = 40.0;
                let direction = (target_transform.translation().xy()
                    - actor_transform.translation().xy())
                .normalize_or_zero();
                velocity.linvel = effect.knockback * scale * direction;
            }
        }
    }

    pub fn cleanup(mut commands: Commands, hit_effects_query: Query<Entity, With<HitEffects>>) {
        for entity in &hit_effects_query {
            commands.entity(entity).despawn_recursive();
        }
    }

    pub fn spawn_from_inputs(
        mut commands: Commands,
        mob_query: Query<(Entity, &Mob, &MobInputs)>,
        handle: Res<Handles>,
    ) {
        for (entity, mob, inputs) in &mob_query {
            if inputs.attack {
                HitboxTemplate {
                    offset: vec2(15.0, 0.0),
                    radius: 9.0,
                    damage: 8.0,
                    knockback: 5.0,
                    faction: mob.faction,
                    parent: entity,
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
    // TODO: Animation, sound effect
}

impl Default for DeathEffects {
    fn default() -> Self {
        Self { reward_gold: 10.0 }
    }
}

impl DeathEffects {
    pub fn apply(
        mut commands: Commands,
        mut death_events: EventReader<DeathEvent>,
        death_effects_query: Query<&DeathEffects>,
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
        }
    }
}

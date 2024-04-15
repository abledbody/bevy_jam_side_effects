use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::common::UpdateSet;
use crate::game::actor::body::DeathAnimation;
use crate::game::actor::body::FlinchAnimation;
use crate::game::actor::body::WalkAnimation;
use crate::game::actor::health::Health;
use crate::game::actor::intent::ActorIntent;
use crate::game::actor::Actor;
use crate::game::alarm::Alarm;
use crate::util::DespawnSet;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CombatAssets>()
            .init_collection::<CombatAssets>();

        app.add_systems(
            Update,
            (
                spawn_attack_hitboxes.in_set(UpdateSet::ApplyIntents),
                clean_up_hitboxes.in_set(UpdateSet::Update),
            ),
        );

        app.add_event::<HitEvent>()
            .add_systems(Update, detect_hit_events.in_set(UpdateSet::Start));

        app.register_type::<HitEffects>()
            .add_systems(Update, apply_hit_effects.in_set(UpdateSet::HandleEvents));

        app.register_type::<HurtEffects>()
            .add_systems(Update, apply_hurt_effects.in_set(UpdateSet::HandleEvents));

        app.add_event::<DeathEvent>();

        app.register_type::<DeathEffects>()
            .add_systems(Update, apply_death_effects.in_set(UpdateSet::HandleEvents));
    }
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct CombatAssets {
    #[asset(path = "sound/sfx/gnoll_attack_miss.wav")]
    attack_miss: Handle<AudioSource>,
    #[asset(path = "sound/sfx/gnoll_attack_hit.wav")]
    attack_hit: Handle<AudioSource>,
}

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
    pub fn spawn(self, commands: &mut Commands, combat_assets: &CombatAssets) -> Entity {
        commands
            .spawn((
                Name::new("Hitbox"),
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
                    success_sound: Some(combat_assets.attack_hit.clone()),
                    failure_sound: Some(combat_assets.attack_miss.clone()),
                    ..default()
                },
            ))
            .id()
    }
}

pub fn spawn_attack_hitboxes(
    mut commands: Commands,
    combat_assets: Res<CombatAssets>,
    actor_query: Query<(&Actor, &GlobalTransform, &ActorIntent)>,
) {
    for (actor, gt, intent) in &actor_query {
        let Some(direction) = intent.attack else {
            continue;
        };

        // Make the hitbox offset slightly ovular
        let ovular_dir = Quat::from_rotation_x(0.5 * PI * 0.3) * direction.extend(0.0);
        let radius = 12.0;
        let distance = radius;

        HitboxTemplate {
            position: gt.translation() + distance * ovular_dir,
            direction,
            radius,
            damage: 8.0,
            knockback: 6.0,
            faction: actor.faction,
        }
        .spawn(&mut commands, &combat_assets);
    }
}

#[derive(Event)]
pub struct HitEvent {
    pub hitbox: Entity,
    pub hurtbox: Entity,
}

fn detect_hit_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut hit_events: EventWriter<HitEvent>,
    hit_query: Query<&HitEffects>,
) {
    for &event in collision_events.read() {
        let CollisionEvent::Started(entity1, entity2, _) = event else {
            continue;
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

#[derive(Component, Reflect, Default)]
pub struct HitEffects {
    pub damage: f32,
    pub knockback: Vec2,
    pub success: bool,
    pub success_sound: Option<Handle<AudioSource>>,
    pub failure_sound: Option<Handle<AudioSource>>,
}

fn apply_hit_effects(
    mut hit_events: EventReader<HitEvent>,
    mut death_events: EventWriter<DeathEvent>,
    mut hitbox_query: Query<&mut HitEffects>,
    mut hurtbox_query: Query<(
        Option<&mut Health>,
        Option<&mut Velocity>,
        Option<&Children>,
    )>,
    mut body_query: Query<&mut FlinchAnimation>,
    audio: Res<Audio>,
) {
    for &HitEvent { hitbox, hurtbox } in hit_events.read() {
        let Ok(mut hit) = hitbox_query.get_mut(hitbox) else {
            continue;
        };

        if let Some(sound) = &hit.success_sound {
            audio.play(sound.clone()).with_volume(0.4);
        }
        hit.success = true;

        let Ok((health, velocity, children)) = hurtbox_query.get_mut(hurtbox) else {
            continue;
        };

        // Damage
        if let Some(mut health) = health {
            if 0.0 < health.current && health.current <= hit.damage {
                death_events.send(DeathEvent(hurtbox));
            }
            health.current -= hit.damage;
        }

        // Knockback
        if let Some(mut velocity) = velocity {
            let scale = 40.0;
            velocity.linvel = hit.knockback * scale;
        }

        // Flinch
        for &child in children.into_iter().flatten() {
            if let Ok(mut flinch) = body_query.get_mut(child) {
                flinch.trigger(hit.knockback.normalize_or_zero());
            }
        }
    }
}

fn clean_up_hitboxes(
    mut despawn: ResMut<DespawnSet>,
    hitbox_query: Query<(Entity, &HitEffects)>,
    audio: Res<Audio>,
) {
    for (entity, effects) in &hitbox_query {
        if !effects.success {
            if let Some(sound) = &effects.failure_sound {
                audio.play(sound.clone()).with_volume(0.4);
            }
        }

        despawn.recursive(entity);
    }
}

#[derive(Component, Reflect, Default)]
pub struct HurtEffects {
    pub increase_alarm: f32,
    pub sound: Option<Handle<AudioSource>>,
}

fn apply_hurt_effects(
    mut hit_events: EventReader<HitEvent>,
    hurt_effects_query: Query<&HurtEffects>,
    mut alarm: ResMut<Alarm>,
    audio: Res<Audio>,
) {
    for &HitEvent { hurtbox, .. } in hit_events.read() {
        let Ok(hurt) = hurt_effects_query.get(hurtbox) else {
            continue;
        };

        // Increase alarm
        alarm.increase(hurt.increase_alarm);

        // Play sound
        if let Some(sound) = &hurt.sound {
            audio.play(sound.clone()).with_volume(0.4);
        }
    }
}

#[derive(Event)]
pub struct DeathEvent(pub Entity);

#[derive(Component, Reflect, Default)]
pub struct DeathEffects {
    pub increase_alarm: f32,
}

fn apply_death_effects(
    mut commands: Commands,
    mut death_events: EventReader<DeathEvent>,
    death_effects_query: Query<&DeathEffects>,
    mut hurt_effects_query: Query<&mut HurtEffects>,
    mut actor_query: Query<&mut Actor>,
    mut alarm: ResMut<Alarm>,
    children_query: Query<&Children>,
    animation_query: Query<(), With<WalkAnimation>>, // And you can use animation_query.contains(child)
) {
    for &DeathEvent(entity) in death_events.read() {
        // Turn into a dead body
        commands
            .entity(entity)
            .insert(ColliderMassProperties::Mass(25.0))
            .remove::<ActorIntent>();
        if let Ok(mut hurt_effects) = hurt_effects_query.get_mut(entity) {
            hurt_effects.increase_alarm = 0.0;
        }
        if let Ok(mut actor) = actor_query.get_mut(entity) {
            actor.brake_deceleration = 700.0;
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
            continue;
        };

        // Increase alarm
        alarm.increase(death.increase_alarm);
    }
}

use bevy::{
    math::{vec3, Vec3Swizzles},
    prelude::*,
};
use bevy_rapier2d::prelude::{ActiveEvents, Collider, CollisionEvent, Sensor};
use rand::{seq::SliceRandom, thread_rng, Rng};

use super::MobInputs;
use crate::{
    asset::{Handles, ImageKey},
    camera::CAMERA_SCALE,
    combat::{DeathEffects, Faction, HitEvent, HurtEffects},
    hud::{HealthBarTemplate, NametagTemplate},
    mob::{player::PlayerControl, BodyTemplate, Health, Mob, MobBundle},
    vfx::DropShadowTemplate,
};

const CASUAL_NAMES: [&str; 51] = [
    "Alex", "Amy", "Abby", "Ashley", "Becca", "Ben", "Cindy", "Chloe", "Chris", "Danny", "Diane",
    "Eli", "Emma", "Greg", "Heather", "Henry", "Ian", "Ike", "Ivy", "Jack", "Jake", "Jenny",
    "Jessica", "Joe", "John", "Jordan", "Kate", "Kim", "Kyle", "Liam", "Lily", "Lisa", "Lucy",
    "Mary", "Megan", "Mike", "Ned", "Nick", "Pete", "Rick", "Rose", "Roy", "Ryan", "Sam", "Sarah",
    "Steve", "Ted", "Tina", "Tom", "Wanda", "Will",
];
// Max length = 8
const FANTASY_FIRST_NAMES: [&str; 26] = [
    "Ambrosia",
    "Anastasia",
    "Augustus",
    "Benedict",
    "Claudius",
    "Cornelia",
    "Delphine",
    "Eurydice",
    "Flavius",
    "Gideon",
    "Gloria",
    "Hyperion",
    "Leonardo",
    "Lucius",
    "Lucretia",
    "Marcella",
    "Octavia",
    "Pandora",
    "Penelope",
    "Tatiana",
    "Tiberius",
    "Theodore",
    "Ulysses",
    "Victoria",
    "Vivian",
    "Wolfgang",
];
// Max length = 8
const FANTASY_LAST_NAMES_P1: [&str; 19] = [
    "Battle", "Beast", "Blood", "Bone", "Brave", "Brute", "Death", "Dread", "Dusk", "Fierce",
    "Gloom", "Grim", "Night", "Noble", "Proud", "Rough", "Scraggle", "War", "Wild",
];
// Max length = 5
const FANTASY_LAST_NAMES_P2: [&str; 9] = [
    "borne", "claw", "heart", "hide", "fang", "jaw", "maw", "snarl", "tooth",
];

fn random_casual_name(mut rng: impl Rng) -> String {
    CASUAL_NAMES.choose(&mut rng).unwrap().to_string()
}

fn random_fantasy_name(mut rng: impl Rng) -> String {
    format!(
        "{} {}{}",
        FANTASY_FIRST_NAMES.choose(&mut rng).unwrap(),
        FANTASY_LAST_NAMES_P1.choose(&mut rng).unwrap(),
        FANTASY_LAST_NAMES_P2.choose(&mut rng).unwrap()
    )
}

fn random_name(mut rng: impl Rng) -> String {
    if rng.gen_ratio(80, 100) {
        random_fantasy_name(rng)
    } else {
        random_casual_name(rng)
    }
}

pub struct EnemyTemplate {
    pub position: Vec2,
    pub name: String,
    pub variant: ImageKey,
    pub health: f32,
    pub reward_gold: f32,
    pub hurt_increase_alarm: f32,
    pub death_increase_alarm: f32,
    pub follow_beyond_detect_radius: f32,
    pub detect_radius: f32,
    pub attack_radius: f32,
}

impl Default for EnemyTemplate {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            name: "Unnamed".to_string(),
            variant: ImageKey::RedGnoll,
            health: 20.0,
            reward_gold: 10.0,
            hurt_increase_alarm: 0.5,
            death_increase_alarm: 5.0,
            follow_beyond_detect_radius: 100.0,
            detect_radius: 1000.0,
            attack_radius: 20.0,
        }
    }
}

impl EnemyTemplate {
    pub fn with_random_casual_name(mut self) -> Self {
        self.name = random_casual_name(thread_rng());
        self
    }

    pub fn with_random_fantasy_name(mut self) -> Self {
        self.name = random_fantasy_name(thread_rng());
        self
    }

    pub fn with_random_name(mut self) -> Self {
        self.name = random_name(thread_rng());
        self
    }

    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        const FACTION: Faction = Faction::Enemy;

        // Children
        let body = BodyTemplate {
            texture: ImageKey::RedGnoll,
            offset: Transform::from_xyz(2.0, 11.0, 0.0),
        }
        .spawn(commands, handle);
        let drop_shadow = DropShadowTemplate::default().spawn(commands, handle);
        let nametag = NametagTemplate {
            offset: Transform::from_xyz(0.0, 26.0, 0.0).with_scale(vec3(
                CAMERA_SCALE,
                CAMERA_SCALE,
                1.0,
            )),
            name: self.name,
        }
        .spawn(commands, handle);
        let health_bar = HealthBarTemplate {
            offset: Transform::from_xyz(0.0, -6.0, 0.0),
        }
        .spawn(commands);
        let detector = DetectorTemplate {
            radius: self.detect_radius,
        }
        .spawn(commands);

        // Parent
        let mut enemy = commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(self.position.extend(0.0)),
                ..default()
            },
            MobBundle {
                health: Health::full(self.health),
                mob: Mob::enemy(),
                ..default()
            }
            .with_faction(FACTION),
            EnemyAi {
                attack_radius: self.attack_radius,
                follow_radius: self.detect_radius + self.follow_beyond_detect_radius,
                ..default()
            },
            HurtEffects {
                increase_alarm: self.hurt_increase_alarm,
                ..default()
            },
            DeathEffects {
                reward_gold: self.reward_gold,
                increase_alarm: self.death_increase_alarm,
            },
        ));
        #[cfg(feature = "debug_mode")]
        enemy.insert(Name::new("Enemy"));

        enemy.add_child(body);
        enemy.add_child(drop_shadow);
        enemy.add_child(nametag);
        enemy.add_child(health_bar);
        enemy.add_child(detector);

        enemy.id()
    }
}

#[derive(Resource, Reflect)]
pub struct Alarm {
    pub current: f32,
    pub max: f32,
}

impl Alarm {
    pub fn empty(max: f32) -> Self {
        Self { current: 0.0, max }
    }

    pub fn increase(&mut self, value: f32) {
        self.current = (self.current + value).min(self.max);
    }

    pub fn scale_difficulty(
        alarm: Res<Alarm>,
        mut enemy_query: Query<(&mut EnemyAi, &Children)>,
        mut detector_query: Query<&mut Transform, With<Detector>>,
    ) {
        let t = alarm.current / alarm.max;
        for (mut enemy, children) in &mut enemy_query {
            for &child in children {
                let Ok(mut transform) = detector_query.get_mut(child) else { continue };

                transform.scale = Vec2::splat(t).extend(1.0);
            }

            // TODO: Adjust enemy.follow_radius
        }
    }
}

#[derive(Component, Reflect)]
pub struct EnemyAi {
    follow_radius: f32,
    attack_radius: f32,
    attack_cooldown: f32,
    attack_cooldown_t: f32,
    target: Option<Entity>,
}

impl Default for EnemyAi {
    fn default() -> Self {
        Self {
            follow_radius: 200.0,
            attack_radius: 30.0,
            attack_cooldown: 0.5,
            attack_cooldown_t: default(),
            target: None,
        }
    }
}

impl EnemyAi {
    pub fn think(
        mut enemy_query: Query<(&mut EnemyAi, &mut MobInputs, &GlobalTransform)>,
        mut detect_events: EventReader<DetectEvent>,
        mut hit_events: EventReader<HitEvent>,
        parent_query: Query<&Parent>,
        player_query: Query<Entity, With<PlayerControl>>,
        transform_query: Query<&GlobalTransform, Without<EnemyAi>>,
        time: Res<Time>,
    ) {
        let Ok(player) = player_query.get_single() else { return };

        // Detect target
        for &DetectEvent { sensor, target } in detect_events.iter() {
            if let Ok((mut enemy, ..)) = parent_query
                .get(sensor)
                .and_then(|parent| enemy_query.get_mut(parent.get()))
            {
                enemy.target = Some(target);
            }
        }
        for &HitEvent { hitbox, hurtbox } in hit_events.iter() {
            if let Ok((mut enemy, ..)) = enemy_query.get_mut(hurtbox) {
                // Assume the hitbox originated from the player.
                enemy.target = Some(player);
            }
        }

        let dt = time.delta_seconds();
        for (mut enemy, mut inputs, mob_gt) in &mut enemy_query {
            let Some(target) = enemy.target else { continue };
            let Ok(target_gt) = transform_query.get(target) else { continue };

            inputs.attack = None;
            enemy.attack_cooldown_t += dt;

            let delta = target_gt.translation().xy() - mob_gt.translation().xy();
            let distance = delta.length();
            let dir = delta.normalize();

            if distance > enemy.follow_radius {
                enemy.target = None;
            } else if distance > enemy.attack_radius {
                inputs.movement = dir;
            } else if enemy.attack_cooldown_t >= enemy.attack_cooldown {
                inputs.attack = Some(dir);
                enemy.attack_cooldown_t = 0.0;
            }
        }
    }
}

pub struct DetectEvent {
    sensor: Entity,
    target: Entity,
}

impl DetectEvent {
    pub fn detect(
        mut collision_events: EventReader<CollisionEvent>,
        mut detect_events: EventWriter<DetectEvent>,
        detector_query: Query<(), With<Detector>>,
    ) {
        for &event in collision_events.iter() {
            let CollisionEvent::Started(entity1, entity2, _) = event else {
                continue
            };

            let mut handle_collision = |sensor: Entity, target: Entity| {
                if detector_query.contains(sensor) {
                    detect_events.send(DetectEvent { sensor, target });
                }
            };

            handle_collision(entity1, entity2);
            handle_collision(entity2, entity1);
        }
    }
}

#[derive(Component, Reflect)]
pub struct Detector;

pub struct DetectorTemplate {
    radius: f32,
}

impl DetectorTemplate {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let mut detector = commands.spawn((
            TransformBundle::default(),
            Collider::ball(self.radius),
            Sensor,
            Faction::Enemy.hitbox_groups(),
            ActiveEvents::COLLISION_EVENTS,
            Detector,
        ));
        #[cfg(feature = "debug_mode")]
        detector.insert(Name::new("DetectionSensor"));

        detector.id()
    }
}

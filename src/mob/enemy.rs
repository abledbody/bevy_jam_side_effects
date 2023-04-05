use bevy::{
    math::{vec2, Vec3Swizzles},
    prelude::*,
};
use bevy_rapier2d::prelude::{ActiveEvents, Collider, CollisionEvent, Sensor};
use rand::{seq::SliceRandom, thread_rng, Rng};

use super::MobInputs;
use crate::{
    asset::{Handles, ImageKey},
    combat::{DeathEffects, Faction},
    hud::{HealthBarTemplate, NametagTemplate},
    mob::{BodyTemplate, Health, MobBundle},
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

// TODO: Generate dark fantasy names occasionally
fn random_name() -> String {
    let mut rng = thread_rng();
    if rng.gen_ratio(80, 100) {
        format!(
            "{} {}{}",
            FANTASY_FIRST_NAMES.choose(&mut rng).unwrap(),
            FANTASY_LAST_NAMES_P1.choose(&mut rng).unwrap(),
            FANTASY_LAST_NAMES_P2.choose(&mut rng).unwrap()
        )
    } else {
        CASUAL_NAMES.choose(&mut rng).unwrap().to_string()
    }
}

pub struct EnemyTemplate {
    pub position: Vec2,
    pub name: String,
    pub variant: ImageKey,
    pub health: f32,
    pub reward_gold: f32,
    pub increase_alarm: f32,
    pub detection_radius: f32,
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
            increase_alarm: 5.0,
            detection_radius: 100.0,
            attack_radius: 20.0,
        }
    }
}

impl EnemyTemplate {
    pub fn with_random_name(mut self) -> Self {
        self.name = random_name();
        self
    }

    pub fn spawn(
        self,
        commands: &mut Commands,
        handle: &Handles,
        attach: impl Into<Option<Entity>>,
    ) -> Entity {
        const FACTION: Faction = Faction::Enemy;

        // Children
        let body = BodyTemplate {
            texture: ImageKey::RedGnoll,
            offset: vec2(2.0, 11.0),
        }
        .spawn(commands, handle);
        let drop_shadow = DropShadowTemplate::default().spawn(commands, handle);
        let nametag = NametagTemplate {
            offset: vec2(0.0, 26.0),
            name: self.name,
        }
        .spawn(commands, handle);
        let health_bar = HealthBarTemplate {
            offset: vec2(0.0, -6.0),
        }
        .spawn(commands);
        let detection_radius = DetectionRadiusTemplate {
            radius: self.detection_radius,
        }
        .spawn(commands);

        // Parent entity
        let mut enemy = if let Some(e) = attach.into() {
            commands.entity(e)
        } else {
            commands.spawn_empty()
        };
        enemy.insert((
            SpatialBundle {
                transform: Transform::from_translation(self.position.extend(0.0)),
                ..default()
            },
            MobBundle {
                health: Health::full(self.health),
                ..default()
            }
            .with_faction(FACTION),
            EnemyAi {
                attack_radius: self.attack_radius,
                ..default()
            },
            DeathEffects {
                reward_gold: self.reward_gold,
                ..default()
            },
        ));
        #[cfg(feature = "debug_mode")]
        enemy.insert(Name::new("Enemy"));

        enemy.add_child(body);
        enemy.add_child(drop_shadow);
        enemy.add_child(nametag);
        enemy.add_child(health_bar);
        enemy.add_child(detection_radius);

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
}

#[derive(Component, Reflect)]
pub struct EnemyAi {
    follow_distance: f32,
    attack_radius: f32,
    attack_cooldown: f32,
    t: f32,
    target: Option<Entity>,
}

impl Default for EnemyAi {
    fn default() -> Self {
        Self {
            follow_distance: 200.0,
            attack_radius: 30.0,
            attack_cooldown: 0.5,
            t: Default::default(),
            target: None,
        }
    }
}

impl EnemyAi {
    pub fn think(
        mut ai_query: Query<(&mut EnemyAi, &mut MobInputs, &GlobalTransform), With<EnemyAi>>,
        mut detection_events: EventReader<DetectionEvent>,
        children_query: Query<&Parent>,
        transform_query: Query<&GlobalTransform, Without<EnemyAi>>,
        time: Res<Time>,
    ) {
        for DetectionEvent { sensor, target } in detection_events.iter() {
            if let Ok(parent) = children_query.get(*sensor) {
                if let Ok((mut ai, _, _)) = ai_query.get_mut(parent.get()) {
                    ai.target = Some(*target);
                }
            }
        }
        for (mut ai, mut mob_inputs, mob_gt) in &mut ai_query {
            if let Some(target) = ai.target {
                if let Ok(target_gt) = transform_query.get(target) {
                    mob_inputs.attack = None;
                    ai.t += time.delta_seconds();
                    let dir = target_gt.translation() - mob_gt.translation();
                    let dist = dir.length();
                    let dir = dir.xy().normalize();
                    if dist > ai.follow_distance {
                        ai.target = None;
                    } else if dist > ai.attack_radius {
                        mob_inputs.movement = dir;
                    } else if ai.t >= ai.attack_cooldown {
                        mob_inputs.attack = Some(dir);
                        ai.t = 0.0;
                    }
                }
            }
        }
    }
}

pub struct DetectionEvent {
    sensor: Entity,
    target: Entity,
}

impl DetectionEvent {
    pub fn detect(
        mut collision_events: EventReader<CollisionEvent>,
        mut detect_events: EventWriter<DetectionEvent>,
        detect_query: Query<Entity, With<Detector>>,
    ) {
        for &event in collision_events.iter() {
            if let CollisionEvent::Started(entity1, entity2, _) = event {
                let mut handle_collision = |sensor: Entity, target: Entity| {
                    if detect_query.contains(sensor) {
                        detect_events.send(DetectionEvent { sensor, target });
                    }
                };

                handle_collision(entity1, entity2);
                handle_collision(entity2, entity1);
            }
        }
    }
}

pub struct DetectionRadiusTemplate {
    radius: f32,
}

#[derive(Component)]
pub struct Detector;

impl DetectionRadiusTemplate {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let mut entity = commands.spawn((
            TransformBundle::default(),
            Collider::ball(self.radius),
            Sensor,
            Faction::Enemy.hitbox_groups(),
            ActiveEvents::COLLISION_EVENTS,
            Detector,
        ));
        #[cfg(feature = "debug_mode")]
        entity.insert(Name::new("Detection Radius"));

        entity.id()
    }
}

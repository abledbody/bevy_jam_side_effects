use bevy::{
    math::{vec3, Vec3Swizzles},
    prelude::*,
};
use bevy_rapier2d::prelude::*;
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
    pub transform: Transform,
    pub name: String,
    pub variant: ImageKey,
    pub health: f32,
    pub reward_gold: f32,
    pub hurt_increase_alarm: f32,
    pub death_increase_alarm: f32,
}

impl Default for EnemyTemplate {
    fn default() -> Self {
        Self {
            transform: default(),
            name: "Unnamed".to_string(),
            variant: ImageKey::GnollRed,
            health: 20.0,
            reward_gold: 10.0,
            hurt_increase_alarm: 0.005,
            death_increase_alarm: 0.05,
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
            texture: ImageKey::GnollRed,
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
        let detector = DetectorTemplate.spawn(commands);

        // Parent
        let mut enemy = commands.spawn((
            SpatialBundle {
                transform: self.transform,
                ..default()
            },
            MobBundle {
                health: Health::full(self.health),
                mob: Mob::enemy(),
                ..default()
            }
            .with_faction(FACTION),
            EnemyAi::default(),
            DifficultyCurve::default(),
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

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct Alarm(pub f32);

impl Alarm {
    pub fn increase(&mut self, value: f32) {
        self.0 = (self.0 + value).min(1.0);
    }
}

#[derive(Reflect, FromReflect)]
pub struct Curve {
    pub y0: f32,
    pub y1: f32,
}

impl Curve {
    pub fn new(y0: f32, y1: f32) -> Self {
        Self { y0, y1 }
    }

    pub fn at(&self, t: f32) -> f32 {
        self.y1 * t + self.y0 * (1.0 - t)
    }
}

#[derive(Component, Reflect)]
pub struct DifficultyCurve {
    speed: Curve,
    detect_radius: Curve,
    follow_radius: Curve,
    attack_radius: Curve,
    attack_cooldown: Curve,
}

impl Default for DifficultyCurve {
    fn default() -> Self {
        Self {
            speed: Curve::new(60.0, 100.0),
            detect_radius: Curve::new(0.0, 1000.0),
            follow_radius: Curve::new(100.0, 1100.0),
            attack_radius: Curve::new(20.0, 40.0),
            attack_cooldown: Curve::new(0.8, 0.4),
        }
    }
}

impl DifficultyCurve {
    pub fn apply(
        alarm: Res<Alarm>,
        mut curve_query: Query<(&DifficultyCurve, &mut EnemyAi, &mut Mob, &Children)>,
        mut detector_query: Query<&mut Transform, With<Detector>>,
    ) {
        for (curve, mut enemy, mut mob, children) in &mut curve_query {
            mob.speed = curve.speed.at(alarm.0);
            let detect_radius = curve.detect_radius.at(alarm.0);
            enemy.follow_radius = curve.follow_radius.at(alarm.0);
            enemy.attack_radius = curve.attack_radius.at(alarm.0);
            enemy.attack_cooldown = curve.attack_cooldown.at(alarm.0);

            for &child in children {
                let Ok(mut transform) = detector_query.get_mut(child) else { continue };
                transform.scale = Vec2::splat(detect_radius).extend(1.0);
            }
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
            follow_radius: 100.0,
            attack_radius: 20.0,
            attack_cooldown: 0.8,
            attack_cooldown_t: 0.0,
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
        for &HitEvent { hurtbox, .. } in hit_events.iter() {
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
            inputs.movement = Vec2::ZERO;
            enemy.attack_cooldown_t += dt;

            let delta = target_gt.translation().xy() - mob_gt.translation().xy();
            let distance = delta.length();
            let dir = delta.normalize();

            if distance > enemy.follow_radius {
                enemy.target = None;
                continue;
            }

            inputs.movement = dir;
            if distance <= enemy.attack_radius && enemy.attack_cooldown_t >= enemy.attack_cooldown {
                inputs.attack = Some(dir);
                enemy.attack_cooldown_t = 0.0;
            }
        }
    }
}

pub struct DetectEvent {
    pub sensor: Entity,
    pub target: Entity,
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

pub struct DetectorTemplate;

impl DetectorTemplate {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let mut detector = commands.spawn((
            TransformBundle::default(),
            Collider::ball(1.0),
            ColliderMassProperties::Mass(0.0),
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

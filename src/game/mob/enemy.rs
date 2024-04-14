use bevy::math::vec2;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;

use super::MobInputs;
use crate::common::asset::AudioKey;
use crate::common::asset::Handles;
use crate::common::asset::ImageKey;
use crate::common::UpdateSet;
use crate::game::combat::DeathEffects;
use crate::game::combat::Faction;
use crate::game::combat::HitEvent;
use crate::game::combat::HurtEffects;
use crate::game::mob::player::PlayerControl;
use crate::game::mob::BodyTemplate;
use crate::game::mob::Health;
use crate::game::mob::Mob;
use crate::game::mob::MobBundle;
use crate::util::ui::health_bar::HealthBarTemplate;
use crate::util::ui::nametag::NametagTemplate;
use crate::util::vfx::DetectPopupTemplate;
use crate::util::vfx::DropShadowTemplate;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Alarm>().init_resource::<Alarm>();

        app.register_type::<DifficultyCurve>()
            .add_systems(Update, DifficultyCurve::apply.in_set(UpdateSet::Start));

        app.register_type::<EnemyAi>()
            .add_systems(Update, EnemyAi::think.in_set(UpdateSet::RecordIntents));

        app.add_event::<DetectEvent>()
            .add_systems(Update, DetectEvent::detect.in_set(UpdateSet::Start));
    }
}

const CASUAL_NAMES: [&str; 52] = [
    "Alex", "Amy", "Abby", "Ashley", "Becca", "Ben", "Cindy", "Chloe", "Chris", "Danny", "Diane",
    "Eli", "Emma", "Gnoll", "Greg", "Heather", "Henry", "Ian", "Ike", "Ivy", "Jack", "Jake",
    "Jenny", "Jessica", "Joe", "John", "Jordan", "Kate", "Kim", "Kyle", "Liam", "Lily", "Lisa",
    "Lucy", "Mary", "Megan", "Mike", "Ned", "Nick", "Pete", "Rick", "Rose", "Roy", "Ryan", "Sam",
    "Sarah", "Steve", "Ted", "Tina", "Tom", "Wanda", "Will",
];
// Max length = 8
const FANTASY_FIRST_NAMES: [&str; 31] = [
    "Alastair",
    "Anastasia",
    "Augustus",
    "Benedict",
    "Beatrice",
    "Bonnabelle",
    "Claudius",
    "Cornelia",
    "Delphine",
    "Dominic",
    "Eurydice",
    "Evelyn",
    "Flavius",
    "Gideon",
    "Gloria",
    "Leonardo",
    "Lucretia",
    "Marcella",
    "Octavia",
    "Pandora",
    "Penelope",
    "Phineas",
    "Professor",
    "Tatiana",
    "Tiberius",
    "Thaddeus",
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
    pub health: f32,
    pub is_corpse: bool,
    pub hurt_increase_alarm: f32,
    pub death_increase_alarm: f32,
}

impl Default for EnemyTemplate {
    fn default() -> Self {
        Self {
            transform: default(),
            name: "Unnamed".to_string(),
            health: 20.0,
            is_corpse: false,
            hurt_increase_alarm: 0.0025,
            death_increase_alarm: 0.025,
        }
    }
}

impl EnemyTemplate {
    pub fn with_random_name(mut self) -> Self {
        self.name = random_name(thread_rng());
        self
    }

    pub fn dead(mut self) -> Self {
        self.is_corpse = true;
        self.health = 0.0;
        self.hurt_increase_alarm = 0.0;
        self.death_increase_alarm = 0.0;
        self
    }

    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        const FACTION: Faction = Faction::Enemy;

        // Children
        let body = BodyTemplate {
            texture: ImageKey::GnollRed,
            offset: Transform::from_xyz(2.0, 11.0, 0.0),
            walk_sound: None,
            is_corpse: self.is_corpse,
        }
        .spawn(commands, handle);
        let drop_shadow = DropShadowTemplate::default().spawn(commands, handle);
        let nametag = NametagTemplate {
            offset: Transform::from_xyz(0.0, 26.0, 0.0),
            name: self.name,
        }
        .spawn(commands, handle);
        let health_bar = HealthBarTemplate {
            offset: Transform::from_xyz(0.0, -6.0, 0.0),
        }
        .spawn(commands);
        let detector = DetectorTemplate.spawn(commands);

        // Parent
        let mut mob = Mob::enemy();
        if self.is_corpse {
            mob.brake_deceleration = 700.0;
        }
        let mut enemy = commands.spawn((
            SpatialBundle {
                transform: self.transform,
                ..default()
            },
            MobBundle {
                health: Health::full(self.health),
                mob,
                ..default()
            }
            .with_faction(FACTION),
            ColliderMassProperties::Mass(if self.is_corpse { 25.0 } else { 1.0 }),
            EnemyAi::default(),
            DifficultyCurve::default(),
            HurtEffects {
                increase_alarm: self.hurt_increase_alarm,
                ..default()
            },
            DeathEffects {
                increase_alarm: self.death_increase_alarm,
            },
        ));
        if self.is_corpse {
            enemy.remove::<MobInputs>();
        }
        #[cfg(feature = "dev")]
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

#[derive(Reflect)]
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
            detect_radius: Curve::new(0.0, 500.0),
            follow_radius: Curve::new(50.0, 550.0),
            attack_radius: Curve::new(20.0, 25.0),
            attack_cooldown: Curve::new(1.0, 0.5),
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
                let Ok(mut transform) = detector_query.get_mut(child) else {
                    continue;
                };
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
            attack_cooldown: 1.0,
            attack_cooldown_t: 0.5,
            target: None,
        }
    }
}

impl EnemyAi {
    pub fn think(
        mut commands: Commands,
        mut enemy_query: Query<(&mut EnemyAi, &mut MobInputs, &GlobalTransform)>,
        mut detect_events: EventReader<DetectEvent>,
        mut hit_events: EventReader<HitEvent>,
        parent_query: Query<&Parent>,
        player_query: Query<Entity, With<PlayerControl>>,
        transform_query: Query<&GlobalTransform, Without<EnemyAi>>,
        handle: Res<Handles>,
        time: Res<Time>,
        audio: Res<Audio>,
    ) {
        let Ok(player) = player_query.get_single() else {
            let mut rng = thread_rng();
            for (mut enemy, mut inputs, _) in &mut enemy_query {
                if enemy.target.is_none() {
                    continue;
                }

                enemy.target = None;
                inputs.attack = None;
                inputs.movement =
                    vec2(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize_or_zero();
            }
            return;
        };

        // Detect target
        let mut handle_detection = |ai: &mut EnemyAi, enemy: Entity, target: Entity| {
            if ai.target.is_some() {
                return;
            }

            audio
                .play(handle.audio[&AudioKey::GnollDetect].clone())
                .with_volume(0.6);
            ai.target = Some(target);
            let popup = DetectPopupTemplate {
                offset: Transform::from_xyz(0.0, 38.0, 0.0),
            }
            .spawn(&mut commands, &handle);
            commands.entity(enemy).add_child(popup);
        };

        for &DetectEvent { sensor, target } in detect_events.read() {
            let Ok(parent) = parent_query.get(sensor) else {
                continue;
            };
            if let Ok((mut enemy, ..)) = enemy_query.get_mut(parent.get()) {
                handle_detection(&mut enemy, parent.get(), target);
            }
        }
        for &HitEvent { hurtbox, .. } in hit_events.read() {
            if let Ok((mut enemy, ..)) = enemy_query.get_mut(hurtbox) {
                // Assume the hitbox originated from the player
                handle_detection(&mut enemy, hurtbox, player);
            }
        }

        let dt = time.delta_seconds();
        for (mut enemy, mut inputs, mob_gt) in &mut enemy_query {
            let Some(target) = enemy.target else { continue };
            let Ok(target_gt) = transform_query.get(target) else {
                continue;
            };

            inputs.attack = None;
            inputs.movement = Vec2::ZERO;

            let target_delta = target_gt.translation().xy() - mob_gt.translation().xy();
            let target_distance = target_delta.length();

            // Give up on target
            if target_distance > enemy.follow_radius {
                enemy.target = None;
                continue;
            }

            // Move towards target
            let target_direction = target_delta.normalize();
            inputs.movement = target_direction;

            // Attack target
            if target_distance <= enemy.attack_radius {
                enemy.attack_cooldown_t -= dt;
                if enemy.attack_cooldown_t <= 0.0 {
                    inputs.attack = Some(target_direction);
                    enemy.attack_cooldown_t = enemy.attack_cooldown;
                }
            } else {
                enemy.attack_cooldown_t = enemy.attack_cooldown / 4.0;
            }
        }
    }
}

#[derive(Event)]
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
        for &event in collision_events.read() {
            let CollisionEvent::Started(entity1, entity2, _) = event else {
                continue;
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
        #[cfg(feature = "dev")]
        detector.insert(Name::new("Detector"));

        detector.id()
    }
}

use bevy::{math::vec2, prelude::*};
use rand::{seq::SliceRandom, thread_rng};

use crate::{
    asset::{Handles, ImageKey},
    combat::{DeathEffects, Faction},
    mob::{BodyTemplate, Health, MobBundle},
    vfx::{DropShadowTemplate, NametagTemplate},
};

const NORMAL_NAMES: [&str; 51] = [
    "Anthony", "Ashley", "Ben", "Brenda", "Charlie", "Carol", "Daniel", "Diane", "Eugene", "Emily",
    "Frank", "Flora", "George", "Gloria", "Henry", "Heather", "Isaac", "Isabelle", "James",
    "Jessica", "Kyle", "Kimberly", "Liam", "Lisa", "Michael", "Megan", "Nicholas", "Natalie",
    "Oliver", "Olivia", "Patrick", "Penelope", "Quincy", "Ryan", "Rebecca", "Steven", "Samantha",
    "Timothy", "Tina", "Ulysses", "Ursula", "Vincent", "Vivian", "Walter", "Willow", "Xander",
    "Xena", "Yahir", "Yael", "Zachary", "Zoe",
];

// TODO: Generate dark fantasy names occasionally
fn random_name() -> String {
    NORMAL_NAMES.choose(&mut thread_rng()).unwrap().to_string()
}

#[derive(Default, Component, Reflect)]
pub struct EnemyAi;

pub struct EnemyTemplate {
    pub position: Vec2,
    pub name: String,
    pub variant: ImageKey,
    pub health: f32,
    pub gold: f32,
}

impl Default for EnemyTemplate {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            name: "Unnamed".to_string(),
            variant: ImageKey::RedGnoll,
            health: 20.0,
            gold: 10.0,
        }
    }
}

impl EnemyTemplate {
    pub fn with_random_name(mut self) -> Self {
        self.name = random_name();
        self
    }

    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
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

        // Parent entity
        let mut enemy = commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(self.position.extend(0.0)),
                ..default()
            },
            MobBundle {
                health: Health(self.health),
                ..default()
            }
            .with_faction(FACTION),
            EnemyAi,
            DeathEffects {
                reward_gold: self.gold,
            },
        ));
        #[cfg(feature = "debug_mode")]
        enemy.insert(Name::new("Enemy"));

        enemy.add_child(body);
        enemy.add_child(drop_shadow);
        enemy.add_child(nametag);

        enemy.id()
    }
}

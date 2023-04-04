use bevy::{math::vec2, prelude::*};
use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::{
    asset::{Handles, ImageKey},
    combat::{DeathEffects, Faction},
    mob::{BodyTemplate, Health, MobBundle},
    vfx::{DropShadowTemplate, NametagTemplate},
};

const CASUAL_NAMES: [&str; 50] = [
    "Alex", "Amy", "Ashley", "Becca", "Benny", "Cindy", "Chris", "Danny", "Diane", "Eli", "Emma",
    "Finn", "Fiona", "Gloria", "Greg", "Gus", "Heather", "Henry", "Ian", "Ike", "Ivy", "Jack",
    "Jake", "Jessica", "John", "Joy", "Katie", "Kim", "Kyle", "Liam", "Lily", "Lisa", "Lucy",
    "Megan", "Mike", "Ned", "Nick", "Pete", "Quinn", "Rick", "Roy", "Ryan", "Sam", "Sasha",
    "Steve", "Ted", "Tina", "Tom", "Wanda", "Will",
];
// Max length = 8
const FANTASY_FIRST_NAMES: [&str; 11] = [
    "Augustus", "Benedict", "Claudius", "Cornelia", "Gideon", "Leonardo", "Lucius", "Octavia",
    "Penelope", "Tatiana", "Vivian",
];
// Max length = 8
const FANTASY_LAST_NAMES_P1: [&str; 18] = [
    "Battle", "Beast", "Blood", "Bone", "Brave", "Brute", "Death", "Dusk", "Fierce", "Gloom",
    "Grim", "Night", "Noble", "Proud", "Rough", "Scraggle", "War", "Wild",
];
// Max length = 5
const FANTASY_LAST_NAMES_P2: [&str; 8] = [
    "borne", "claw", "heart", "fang", "jaw", "maw", "snarl", "tooth",
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

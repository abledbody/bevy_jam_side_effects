use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct Loot {
    pub gold: f32,
}

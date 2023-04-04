use bevy::prelude::*;

use crate::{
    animation::Offset,
    asset::{Handles, ImageKey},
};

pub struct DropShadowTemplate {
    pub offset: Vec2,
}

impl Default for DropShadowTemplate {
    fn default() -> Self {
        Self { offset: Vec2::ZERO }
    }
}

impl DropShadowTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        let mut drop_shadow = commands.spawn((
            SpriteBundle {
                texture: handle.image[&ImageKey::DropShadow].clone(),
                transform: Transform::from_xyz(0.0, 0.0, -0.01),
                ..default()
            },
            Offset {
                pos: self.offset,
                ..default()
            },
        ));
        #[cfg(feature = "debug_mode")]
        drop_shadow.insert(Name::new("DropShadow"));

        drop_shadow.id()
    }
}

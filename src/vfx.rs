use bevy::prelude::*;

use crate::{
    animation::Offset,
    asset::{Handles, ImageKey},
};

pub struct DropShadowTemplate {
    pub offset: Transform,
}

impl Default for DropShadowTemplate {
    fn default() -> Self {
        Self {
            offset: Transform::from_xyz(0.0, 0.0, -0.01),
        }
    }
}

impl DropShadowTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        let mut drop_shadow = commands.spawn((
            SpriteBundle {
                texture: handle.image[&ImageKey::DropShadow].clone(),
                ..default()
            },
            Offset(self.offset),
        ));
        #[cfg(feature = "debug_mode")]
        drop_shadow.insert(Name::new("DropShadow"));

        drop_shadow.id()
    }
}

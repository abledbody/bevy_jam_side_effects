use bevy::prelude::*;

use crate::common::asset::Handles;
use crate::common::asset::ImageKey;
use crate::util::animation::lifetime::Lifetime;
use crate::util::animation::offset::Offset;

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
        commands
            .spawn((
                Name::new("DropShadow"),
                SpriteBundle {
                    texture: handle.image[&ImageKey::DropShadow].clone(),
                    ..default()
                },
                Offset(self.offset),
            ))
            .id()
    }
}

pub struct AlertPopupTemplate {
    pub offset: Transform,
}

impl Default for AlertPopupTemplate {
    fn default() -> Self {
        Self {
            offset: Transform::from_xyz(0.0, 0.0, 0.01),
        }
    }
}

impl AlertPopupTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        commands
            .spawn((
                Name::new("AlertPopup"),
                SpriteBundle {
                    texture: handle.image[&ImageKey::AlertPopup].clone(),
                    ..default()
                },
                Lifetime(1.0),
                Offset(self.offset),
            ))
            .id()
    }
}

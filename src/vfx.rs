use bevy::prelude::*;

use crate::{
    animation::{Lifetime, Offset},
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
        #[cfg(feature = "dev")]
        drop_shadow.insert(Name::new("DropShadow"));

        drop_shadow.id()
    }
}

pub struct DetectPopupTemplate {
    pub offset: Transform,
}

impl Default for DetectPopupTemplate {
    fn default() -> Self {
        Self {
            offset: Transform::from_xyz(0.0, 0.0, 0.01),
        }
    }
}

impl DetectPopupTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        let mut detect_popup = commands.spawn((
            SpriteBundle {
                texture: handle.image[&ImageKey::DetectPopup].clone(),
                ..default()
            },
            Lifetime(1.0),
            Offset(self.offset),
        ));
        #[cfg(feature = "dev")]
        detect_popup.insert(Name::new("DetectPopup"));

        detect_popup.id()
    }
}

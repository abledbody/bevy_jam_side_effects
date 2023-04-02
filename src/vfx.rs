use bevy::prelude::*;

use crate::{animation::Offset, asset::ImageKey};

pub fn drop_shadow(parent_z: f32, offset: Vec2) -> Entity {
    let mut drop_shadow = commands.spawn((
        SpriteBundle {
            texture: handle.image[&ImageKey::DropShadow].clone(),
            transform: Transform::from_xyz(0.0, 0.0, 50.0 - parent_z),
            ..default()
        },
        Offset(offset),
    ));
    #[cfg(feature = "debug_mode")]
    drop_shadow.insert(Name::new("DropShadow"));

    drop_shadow.id()
}

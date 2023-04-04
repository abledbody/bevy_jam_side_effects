use bevy::{
    math::{vec2, vec3},
    prelude::*,
};

use crate::{
    animation::Offset,
    asset::{FontKey, Handles, ImageKey},
    camera::CAMERA_SCALE,
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
            Offset(self.offset),
        ));
        #[cfg(feature = "debug_mode")]
        drop_shadow.insert(Name::new("DropShadow"));

        drop_shadow.id()
    }
}

pub struct NametagTemplate {
    pub offset: Vec2,
    pub name: String,
}

impl Default for NametagTemplate {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            name: "Unnamed".to_string(),
        }
    }
}

impl NametagTemplate {
    const BACKDROP_COLOR: Color = Color::rgba(0.2, 0.1, 0.2, 0.7);

    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        let style = TextStyle {
            font: handle.font[&FontKey::Regular].clone(),
            font_size: 14.0,
            color: Color::WHITE,
        };

        // Children
        let backdrop = commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: Self::BACKDROP_COLOR,
                    custom_size: Some(vec2(70.0, 14.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, -0.1),
                ..default()
            })
            .id();

        // Parent
        let mut nametag = commands.spawn((
            Text2dBundle {
                text: Text::from_section(self.name, style),
                transform: Transform::from_scale(vec3(CAMERA_SCALE, CAMERA_SCALE, 1.0))
                    .with_translation(vec3(0.0, 0.0, 500.0)),
                ..default()
            },
            Offset(self.offset),
        ));
        #[cfg(feature = "debug_mode")]
        nametag.insert(Name::new("Nametag"));

        nametag.add_child(backdrop);

        nametag.id()
    }
}

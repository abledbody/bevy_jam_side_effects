use bevy::prelude::*;

use crate::util::animation::offset::Offset;

pub struct BackdropTemplate {
    pub offset: Transform,
    pub size: Vec2,
}

impl BackdropTemplate {
    pub const COLOR: Color = Color::rgba(0.2, 0.1, 0.2, 0.6);

    pub fn spawn(self, commands: &mut Commands) -> Entity {
        commands
            .spawn((
                Name::new("Backdrop"),
                SpriteBundle {
                    sprite: Sprite {
                        color: Self::COLOR,
                        custom_size: Some(self.size),
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 0.0, -0.001),
                    ..default()
                },
                Offset(self.offset),
            ))
            .id()
    }
}

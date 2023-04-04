use bevy::{math::vec2, prelude::*};

use crate::{animation::Offset, mob::Health};

#[derive(Component, Reflect)]
pub struct HealthBar;

impl HealthBar {
    const BACKDROP_COLOR: Color = Color::rgba(0.2, 0.1, 0.2, 0.6);
    const HEALTH_COLOR: [Color; 4] = [
        Color::rgba(0.9, 0.2, 0.3, 0.3),
        Color::rgba(0.8, 0.5, 0.3, 0.3),
        Color::rgba(0.7, 0.7, 0.3, 0.3),
        Color::rgba(0.2, 0.9, 0.3, 0.3),
    ];

    pub fn update(
        mut health_bar_query: Query<(&mut Sprite, &Parent), With<HealthBar>>,
        health_query: Query<&Health>,
    ) {
        for (mut sprite, parent) in &mut health_bar_query {
            let Ok(health) = health_query.get(parent.get()) else {
                continue
            };

            // Hack but it works
            let t = (health.current / health.max).max(0.000001);
            let color_idx = (t * Self::HEALTH_COLOR.len() as f32).ceil() as usize - 1;

            sprite.color = Self::HEALTH_COLOR[color_idx];
            sprite.custom_size = Some(vec2(20.0 * health.current.max(0.0) / health.max, 2.5));
        }
    }
}

pub struct HealthBarTemplate {
    pub offset: Vec2,
}

impl HealthBarTemplate {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        // Children
        let mut backdrop = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: HealthBar::BACKDROP_COLOR,
                custom_size: Some(vec2(21.5, 4.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, -0.1),
            ..default()
        });
        #[cfg(feature = "debug_mode")]
        backdrop.insert(Name::new("Backdrop"));
        let backdrop = backdrop.id();

        // Parent
        let mut health_bar = commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(0.0, 0.0, 500.0),
                ..default()
            },
            Offset {
                pos: self.offset,
                ..default()
            },
            HealthBar,
        ));
        #[cfg(feature = "debug_mode")]
        health_bar.insert(Name::new("HealthBar"));

        health_bar.add_child(backdrop);

        health_bar.id()
    }
}

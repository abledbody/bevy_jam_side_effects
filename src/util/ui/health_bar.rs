use bevy::math::vec2;
use bevy::prelude::*;

use crate::common::UpdateSet;
use crate::game::actor::Health;
use crate::util::ui::backdrop::BackdropTemplate;

pub struct HealthBarPlugin;

impl Plugin for HealthBarPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<HealthBar>()
            .add_systems(Update, update_health_bars.in_set(UpdateSet::UpdateUi));
    }
}

#[derive(Component, Reflect)]
pub struct HealthBar;

impl HealthBar {
    const COLOR_RAMP: [Color; 4] = [
        Color::rgba(0.9, 0.2, 0.3, 0.3),
        Color::rgba(0.8, 0.5, 0.3, 0.3),
        Color::rgba(0.7, 0.7, 0.3, 0.3),
        Color::rgba(0.2, 0.9, 0.3, 0.3),
    ];
}

fn update_health_bars(
    mut health_bar_query: Query<(&mut Sprite, &Parent), With<HealthBar>>,
    parent_query: Query<&Parent, Without<HealthBar>>,
    health_query: Query<&Health>,
) {
    for (mut sprite, parent) in &mut health_bar_query {
        let Ok(parent) = parent_query.get(parent.get()) else {
            continue;
        };
        let Ok(health) = health_query.get(parent.get()) else {
            continue;
        };

        // Hack but it works
        let t = (health.current / health.max).max(0.000001);
        let color_idx = (t * HealthBar::COLOR_RAMP.len() as f32).ceil() as usize - 1;

        sprite.color = HealthBar::COLOR_RAMP[color_idx];
        sprite.custom_size = Some(vec2(20.0 * t, 2.5));
    }
}

pub struct HealthBarTemplate {
    pub offset: Transform,
}

impl HealthBarTemplate {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        // Children
        let mut health_bar = commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(0.0, 0.0, 0.001),
                ..default()
            },
            HealthBar,
        ));
        #[cfg(feature = "dev")]
        health_bar.insert(Name::new("HealthBar"));
        let health_bar = health_bar.id();

        // Parent
        let backdrop = BackdropTemplate {
            offset: self.offset,
            size: vec2(21.5, 4.0),
        }
        .spawn(commands);

        commands.entity(backdrop).add_child(health_bar);

        backdrop
    }
}

use bevy::{
    math::{vec2, vec3},
    prelude::*,
};

use crate::{
    animation::Offset,
    asset::{FontKey, Handles},
    camera::CAMERA_SCALE,
    mob::{enemy::Alarm, Health},
};

struct BackdropTemplate {
    size: Vec2,
}

impl BackdropTemplate {
    const COLOR: Color = Color::rgba(0.2, 0.1, 0.2, 0.6);

    fn spawn(self, commands: &mut Commands) -> Entity {
        let mut backdrop = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Self::COLOR,
                custom_size: Some(self.size),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, -0.001),
            ..default()
        });
        #[cfg(feature = "debug_mode")]
        backdrop.insert(Name::new("Backdrop"));

        backdrop.id()
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
    const TEXT_COLOR: Color = Color::rgba(0.9, 0.9, 0.85, 0.8);

    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        let style = TextStyle {
            font: handle.font[&FontKey::Bold].clone(),
            font_size: 14.0,
            color: Self::TEXT_COLOR,
        };

        // Children
        let backdrop = BackdropTemplate {
            size: vec2(110.0, 14.0),
        }
        .spawn(commands);

        // Parent
        let mut nametag = commands.spawn((
            Text2dBundle {
                text: Text::from_section(self.name, style),
                transform: Transform::from_scale(vec3(CAMERA_SCALE, CAMERA_SCALE, 1.0)),
                ..default()
            },
            Offset {
                pos: self.offset,
                ..default()
            },
        ));
        #[cfg(feature = "debug_mode")]
        nametag.insert(Name::new("Nametag"));

        nametag.add_child(backdrop);

        nametag.id()
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
            let color_idx = (t * Self::COLOR_RAMP.len() as f32).ceil() as usize - 1;

            sprite.color = Self::COLOR_RAMP[color_idx];
            sprite.custom_size = Some(vec2(20.0 * t, 2.5));
        }
    }
}

pub struct HealthBarTemplate {
    pub offset: Vec2,
}

impl HealthBarTemplate {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        // Children
        let backdrop = BackdropTemplate {
            size: vec2(21.5, 4.0),
        }
        .spawn(commands);

        // Parent
        let mut health_bar = commands.spawn((
            SpriteBundle::default(),
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

#[derive(Component, Reflect)]
pub struct AlarmMeter;

impl AlarmMeter {
    const COLOR_RAMP: [Color; 6] = [
        Color::rgba(0.5, 0.8, 0.3, 0.3),
        Color::rgba(0.7, 0.7, 0.3, 0.3),
        Color::rgba(0.8, 0.5, 0.3, 0.3),
        Color::rgba(0.8, 0.3, 0.2, 0.4),
        Color::rgba(0.9, 0.2, 0.2, 0.5),
        Color::rgba(1.0, 0.1, 0.1, 0.6),
    ];

    pub fn update(
        mut alarm_meter_query: Query<(&mut BackgroundColor, &mut Style), With<AlarmMeter>>,
        alarm: Res<Alarm>,
    ) {
        for (mut color, mut style) in &mut alarm_meter_query {
            // Hack but it works
            let t = (alarm.current / alarm.max).max(0.000001);
            let color_idx = (t * Self::COLOR_RAMP.len() as f32).ceil() as usize - 1;

            color.0 = Self::COLOR_RAMP[color_idx];
            style.size.width = Val::Percent(100.0 * t);
        }
    }
}

pub struct AlarmMeterTemplate;

impl AlarmMeterTemplate {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let mut alarm_meter = commands.spawn((NodeBundle::default(), AlarmMeter));
        #[cfg(feature = "debug_mode")]
        alarm_meter.insert(Name::new("AlarmMeter"));
        let alarm_meter = alarm_meter.id();

        let mut backdrop = commands.spawn(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Px(25.0)),
                padding: UiRect::all(Val::Px(7.0)),
                size: Size::new(Val::Percent(100.0), Val::Px(50.0)),
                ..default()
            },
            background_color: BackgroundColor(BackdropTemplate::COLOR),
            z_index: ZIndex::Global(100),
            ..default()
        });
        #[cfg(feature = "debug_mode")]
        backdrop.insert(Name::new("AlarmMeterBackdrop"));

        backdrop.add_child(alarm_meter);

        backdrop.id()
    }
}

use bevy::{math::vec2, prelude::*};
use rand::{thread_rng, Rng};

use crate::{
    animation::Offset,
    asset::{FontKey, Handles, ImageKey},
    camera::GameCamera,
    mob::{enemy::Alarm, Health},
};

struct BackdropTemplate {
    offset: Transform,
    size: Vec2,
}

impl BackdropTemplate {
    const COLOR: Color = Color::rgba(0.2, 0.1, 0.2, 0.6);

    fn spawn(self, commands: &mut Commands) -> Entity {
        let mut backdrop = commands.spawn((
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
        ));
        #[cfg(feature = "debug_mode")]
        backdrop.insert(Name::new("Backdrop"));

        backdrop.id()
    }
}

#[derive(Component, Reflect)]
pub struct FontSizeHack(pub f32);

impl FontSizeHack {
    pub fn scale(
        camera_query: Query<(&OrthographicProjection, &Camera), With<GameCamera>>,
        mut text_query: Query<(&mut Text, &mut Transform, &FontSizeHack)>,
    ) {
        let Ok((camera_proj, camera)) = camera_query.get_single() else { return };
        let Some(viewport_size) = camera.logical_viewport_size() else { return };

        let units_per_pixel = camera_proj.area.width() / viewport_size.x;
        let scale = Vec2::splat(units_per_pixel).extend(1.0);
        let max_font_size = 800.0;

        for (mut text, mut transform, font_size_hack) in &mut text_query {
            let font_size = font_size_hack.0 / units_per_pixel;
            let capped_font_size = font_size.min(max_font_size);
            transform.scale = scale * font_size / capped_font_size;
            debug!("Setting font size {capped_font_size}");
            for section in &mut text.sections {
                section.style.font_size = capped_font_size;
            }
        }
    }
}

pub struct NametagTemplate {
    pub offset: Transform,
    pub name: String,
}

impl Default for NametagTemplate {
    fn default() -> Self {
        Self {
            offset: default(),
            name: "Unnamed".to_string(),
        }
    }
}

impl NametagTemplate {
    const TEXT_COLOR: Color = Color::rgba(0.9, 0.9, 0.85, 0.8);

    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        let style = TextStyle {
            font: handle.font[&FontKey::Bold].clone(),
            font_size: 4.0,
            color: Self::TEXT_COLOR,
        };

        // Children
        let mut nametag = commands.spawn((
            Text2dBundle {
                text: Text::from_section(self.name, style.clone()),
                transform: Transform::from_xyz(0.0, 0.2, 0.001),
                ..default()
            },
            FontSizeHack(style.font_size),
        ));
        #[cfg(feature = "debug_mode")]
        nametag.insert(Name::new("Nametag"));
        let nametag = nametag.id();

        // Parent
        let backdrop = BackdropTemplate {
            size: vec2(32.0, 4.0),
            offset: self.offset,
        }
        .spawn(commands);

        commands.entity(backdrop).add_child(nametag);

        backdrop
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
        parent_query: Query<&Parent, Without<HealthBar>>,
        health_query: Query<&Health>,
    ) {
        for (mut sprite, parent) in &mut health_bar_query {
            let Ok(parent) = parent_query.get(parent.get()) else { continue };
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
        #[cfg(feature = "debug_mode")]
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

#[derive(Component, Reflect, Default)]
pub struct AlarmMeter {
    pub old_alarm: f32,
    pub shake: f32,
}

impl AlarmMeter {
    const COLOR_RAMP: [Color; 5] = [
        Color::rgba(0.7, 0.7, 0.3, 0.3),
        Color::rgba(0.8, 0.5, 0.3, 0.4),
        Color::rgba(0.8, 0.3, 0.2, 0.5),
        Color::rgba(0.9, 0.2, 0.2, 0.6),
        Color::rgba(1.0, 0.1, 0.1, 0.7),
    ];

    pub fn update(
        mut alarm_meter_query: Query<(&mut AlarmMeter, &mut BackgroundColor, &mut Style, &Parent)>,
        parent_query: Query<&Parent, Without<AlarmMeter>>,
        mut backdrop_query: Query<&mut Style, Without<AlarmMeter>>,
        alarm: Res<Alarm>,
        time: Res<Time>,
    ) {
        let dt = time.delta_seconds();
        for (mut meter, mut color, mut style, parent) in &mut alarm_meter_query {
            // Hack but it works
            let t = alarm.0.max(0.000001);
            let color_idx = (t * Self::COLOR_RAMP.len() as f32).ceil() as usize - 1;

            color.0 = Self::COLOR_RAMP[color_idx];
            style.size.width = Val::Percent(100.0 * t);

            if let Ok(mut backdrop) = parent_query
                .get(parent.get())
                .and_then(|parent| backdrop_query.get_mut(parent.get()))
            {
                let mut rng = thread_rng();
                let dx = rng.gen_range(-1.0..1.0) * meter.shake;
                let dy = rng.gen_range(-1.0..1.0) * meter.shake;
                backdrop.position.left = Val::Percent(dx);
                backdrop.position.top = Val::Percent(dy);
            };

            let shake_decay = 0.05f32;
            let shake_scale = 50.0;
            meter.shake *= shake_decay.powf(dt);
            meter.shake += shake_scale * (alarm.0 - meter.old_alarm);
            meter.old_alarm = alarm.0;
        }
    }
}

pub struct AlarmMeterTemplate;

impl AlarmMeterTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        let mut alarm_meter = commands.spawn((NodeBundle::default(), AlarmMeter::default()));
        #[cfg(feature = "debug_mode")]
        alarm_meter.insert(Name::new("AlarmMeter"));
        let alarm_meter = alarm_meter.id();

        let mut backdrop = commands.spawn(NodeBundle {
            style: Style {
                //margin: UiRect::all(Val::Percent(1.0)),
                padding: UiRect::all(Val::Percent(0.35)),
                size: Size::new(Val::Percent(100.0), Val::Percent(80.0)),
                ..default()
            },
            background_color: BackgroundColor(BackdropTemplate::COLOR),
            ..default()
        });
        #[cfg(feature = "debug_mode")]
        backdrop.insert(Name::new("AlarmMeterBackdrop"));
        backdrop.add_child(alarm_meter);
        let backdrop = backdrop.id();

        let mut icon = commands.spawn(ImageBundle {
            style: Style {
                margin: UiRect::left(Val::Percent(1.0)),
                size: Size::new(Val::Auto, Val::Percent(100.0)),
                aspect_ratio: Some(15.0 / 16.0),
                flex_shrink: 0.0,
                ..default()
            },
            image: UiImage::new(handle.image[&ImageKey::AlarmMeterIcon].clone()),
            ..default()
        });
        #[cfg(feature = "debug_mode")]
        icon.insert(Name::new("AlarmMeterIcon"));
        let icon = icon.id();

        let mut container = commands.spawn(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Percent(1.0)),
                size: Size::new(Val::Percent(100.0), Val::Percent(10.0)),
                align_items: AlignItems::Center,
                ..default()
            },
            z_index: ZIndex::Global(100),
            ..default()
        });
        #[cfg(feature = "debug_mode")]
        container.insert(Name::new("AlarmMeterContainer"));

        container.add_child(backdrop);
        container.add_child(icon);

        container.id()
    }
}

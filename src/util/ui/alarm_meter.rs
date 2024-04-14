use bevy::prelude::*;
use rand::thread_rng;
use rand::Rng;

use crate::common::asset::Handles;
use crate::common::asset::ImageKey;
use crate::common::UpdateSet;
use crate::game::mob::enemy::Alarm;
use crate::util::ui::backdrop::BackdropTemplate;

pub struct AlarmMeterPlugin;

impl Plugin for AlarmMeterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<AlarmMeter>()
            .add_systems(Update, AlarmMeter::update.in_set(UpdateSet::UpdateUi));
    }
}

#[derive(Component, Reflect, Default)]
pub struct AlarmMeter {
    pub old_alarm: f32,
    pub shake: f32,
}

impl AlarmMeter {
    const COLOR_RAMP: [Color; 6] = [
        Color::rgba(0.8, 0.2, 0.2, 0.3),
        Color::rgba(0.8, 0.2, 0.2, 0.4),
        Color::rgba(0.9, 0.2, 0.2, 0.45),
        Color::rgba(0.9, 0.15, 0.2, 0.5),
        Color::rgba(0.95, 0.15, 0.2, 0.6),
        Color::rgba(1.0, 0.1, 0.1, 0.8),
    ];

    pub fn update(
        mut alarm_meter_query: Query<(&mut AlarmMeter, &mut BackgroundColor, &mut Style, &Parent)>,
        backdrop_query: Query<&Parent, Without<AlarmMeter>>,
        mut alarm_icon_query: Query<&mut UiImage>,
        mut container_query: Query<(&mut Style, &Children), Without<AlarmMeter>>,
        handle: Res<Handles>,
        alarm: Res<Alarm>,
        time: Res<Time>,
    ) {
        let dt = time.delta_seconds();
        for (mut meter, mut color, mut style, backdrop) in &mut alarm_meter_query {
            // Hack but it works
            let x = alarm.0.clamp(0.000001, 1.0);
            let color_idx = (x * Self::COLOR_RAMP.len() as f32).ceil() as usize - 1;

            // Update color and size
            color.0 = Self::COLOR_RAMP[color_idx];
            style.width = Val::Percent(100.0 * x);

            // Calculate shake
            let shake_decay = 0.05f32;
            let shake_scale = 60.0;
            meter.shake *= shake_decay.powf(dt);
            meter.shake += shake_scale * (alarm.0 - meter.old_alarm);
            meter.old_alarm = alarm.0;

            let Ok(container) = backdrop_query.get(backdrop.get()) else {
                continue;
            };
            let Ok((mut container, children)) = container_query.get_mut(container.get()) else {
                continue;
            };

            // Apply shake
            if meter.shake > 0.01 {
                let mut rng = thread_rng();
                let dx = rng.gen_range(-1.0..1.0) * meter.shake;
                let dy = rng.gen_range(-1.0..1.0) * meter.shake;
                container.left = Val::Percent(dx);
                container.top = Val::Percent(dy);
            }

            // Apply alarm flashing
            let t = time.elapsed_seconds();
            for &child in children {
                let Ok(mut image) = alarm_icon_query.get_mut(child) else {
                    continue;
                };
                let shake_flash = meter.shake > 0.05;
                let max_flash = alarm.0 >= 1.0 && t.fract() < 0.25;
                let flash = shake_flash || max_flash;
                image.texture = handle.image[&if flash {
                    ImageKey::AlarmMeterIconFlash
                } else {
                    ImageKey::AlarmMeterIcon
                }]
                    .clone();
            }
        }
    }
}

pub struct AlarmMeterTemplate;

impl AlarmMeterTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        let mut alarm_meter = commands.spawn((NodeBundle::default(), AlarmMeter::default()));
        #[cfg(feature = "dev")]
        alarm_meter.insert(Name::new("AlarmMeter"));
        let alarm_meter = alarm_meter.id();

        let mut backdrop = commands.spawn(NodeBundle {
            style: Style {
                //margin: UiRect::all(Val::Percent(1.0)),
                padding: UiRect::all(Val::Percent(0.35)),
                width: Val::Percent(100.0),
                height: Val::Percent(70.0),
                ..default()
            },
            background_color: BackgroundColor(BackdropTemplate::COLOR),
            ..default()
        });
        #[cfg(feature = "dev")]
        backdrop.insert(Name::new("AlarmMeterBackdrop"));
        backdrop.add_child(alarm_meter);
        let backdrop = backdrop.id();

        let mut icon = commands.spawn(ImageBundle {
            style: Style {
                margin: UiRect::left(Val::Percent(1.0)),
                width: Val::Auto,
                height: Val::Percent(100.0),
                ..default()
            },
            image: UiImage::new(handle.image[&ImageKey::AlarmMeterIcon].clone()),
            ..default()
        });
        #[cfg(feature = "dev")]
        icon.insert(Name::new("AlarmMeterIcon"));
        let icon = icon.id();

        let mut container = commands.spawn(NodeBundle {
            style: Style {
                margin: UiRect::all(Val::Percent(1.0)),
                width: Val::Percent(100.0),
                height: Val::Percent(12.0),
                align_items: AlignItems::Center,
                ..default()
            },
            z_index: ZIndex::Global(100),
            ..default()
        });
        #[cfg(feature = "dev")]
        container.insert(Name::new("AlarmMeterContainer"));

        container.add_child(backdrop);
        container.add_child(icon);

        container.id()
    }
}

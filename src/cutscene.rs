use bevy::prelude::*;

use crate::asset::{AudioKey, FontKey, Handles};
const NUM_LINES: usize = 3;
const TEXT_LINES: [&str; NUM_LINES] = [
    "You are Sai.",
    "You have decided to defect.",
    "TOTALLY FITS THE THEME",
];
const LINE_VOLUMES: [f32; NUM_LINES] = [1.0, 1.0, 0.3];

#[derive(Component, Reflect)]
pub struct StartText {
    pub phase: usize,
    pub section: usize,
    pub hue: f32,
    pub sounds: [Handle<AudioSource>; NUM_LINES],
}

impl StartText {
    pub fn update(mut start_text_query: Query<(&mut Text, &mut StartText)>, time: Res<Time>) {
        let dt = time.delta_seconds();

        for (mut text, mut start_text) in &mut start_text_query {
            start_text.hue += dt;
            while start_text.hue >= 1.0 {
                start_text.hue -= 1.0;
            }

            if let Some(section) = text.sections.get_mut(1) {
                section.style.color = Color::hsl(start_text.hue * 360.0, 1.0, 0.5);
            }
        }
    }

    pub fn advance(
        mut commands: Commands,
        mut start_text_query: Query<(Entity, &mut Text, &mut StartText)>,
        audio: Res<Audio>,
    ) {
        for (entity, mut text, mut start_text) in &mut start_text_query {
            start_text.phase += 1;
            let phase_index = start_text.phase - 1;

            if phase_index < NUM_LINES {
                if phase_index == NUM_LINES - 1 {
                    start_text.section += 1;
                }

                text.sections[start_text.section].value = format!(
                    "{}\n\n\n\n{}",
                    text.sections[start_text.section].value, TEXT_LINES[phase_index]
                );

                audio.play_with_settings(
                    start_text.sounds[phase_index].clone(),
                    PlaybackSettings::default().with_volume(LINE_VOLUMES[phase_index]),
                );
            } else {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

pub struct StartTextTemplate;

impl StartTextTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        let text_style = TextStyle {
            font_size: 10.0,
            font: handle.font[&FontKey::Pixel].clone(),
            ..default()
        };

        let mut entity = commands.spawn((
            TextBundle {
                text: Text::from_sections(vec![
                    TextSection::new("", text_style.clone()),
                    TextSection::new("", text_style),
                ])
                .with_alignment(TextAlignment::Center),
                style: Style {
                    margin: UiRect {
                        left: Val::Auto,
                        right: Val::Auto,
                        top: Val::Percent(10.0),
                        bottom: Val::Px(40.0),
                    },
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
            StartText {
                phase: 0,
                section: 0,
                hue: 0.0,
                sounds: [
                    handle.audio[&AudioKey::Pop2].clone(),
                    handle.audio[&AudioKey::Pop1].clone(),
                    handle.audio[&AudioKey::Jackpot].clone(),
                ],
            },
        ));
        #[cfg(feature = "debug_mode")]
        entity.insert(Name::new("StartText"));

        entity.id()
    }
}

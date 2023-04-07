use bevy::prelude::*;

use crate::asset::{AudioKey, FontKey, Handles};

const NUM_LINES: usize = 3;
const TEXT_LINES: [&str; NUM_LINES] = [
    "You are Sai.",
    "You have decided to defect.",
    "!!! THEME TOTALLY FOLLOWED !!!",
];
const LINE_VOLUMES: [f32; NUM_LINES] = [1.0, 1.0, 0.3];

#[derive(Component, Reflect)]
pub struct StartText {
    pub phase: usize,
    pub sounds: [Handle<AudioSource>; NUM_LINES],
}

impl StartText {
    pub fn update(
        mut commands: Commands,
        mut start_text_query: Query<(Entity, &mut Text, &mut StartText)>,
        audio: Res<Audio>,
    ) {
        for (entity, mut text, mut start_text) in &mut start_text_query {
            start_text.phase += 1;
            let phase_index = start_text.phase - 1;

            if phase_index < NUM_LINES {
                text.sections[0].value = TEXT_LINES[..start_text.phase].join("\n\n\n\n");

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
            font_size: 18.0,
            font: handle.font[&FontKey::Pixel].clone(),
            ..default()
        };

        let mut entity = commands.spawn((
            TextBundle {
                text: Text::from_section("", text_style).with_alignment(TextAlignment::Center),
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
            StartText {
                phase: 0,
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

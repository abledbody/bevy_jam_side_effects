use bevy::prelude::*;

use crate::{
    asset::{AudioKey, FontKey, Handles},
    mob::player::PlayerControl,
};

const NUM_LINES: usize = 3;
const TEXT_LINES: [&str; NUM_LINES] = ["You are Sai.", "You have chosen to Defect.", "GOOD LUCK!"];
const LINE_VOLUMES: [f32; NUM_LINES] = [1.0, 1.0, 0.3];

#[derive(Component, Reflect)]
pub struct Cutscene {
    pub phase: usize,
    pub section: usize,
    pub hue: f32,
    pub sounds: [Handle<AudioSource>; NUM_LINES],
}

impl Cutscene {
    pub fn update(
        mut cutscene_query: Query<(&mut Text, &mut Cutscene)>,
        mut player_query: Query<&mut PlayerControl>,
        time: Res<Time>,
    ) {
        let Ok(mut player) = player_query.get_single_mut() else { return };

        let dt = time.delta_seconds();

        for (mut text, mut cutscene) in &mut cutscene_query {
            player.deny_input = true;
            cutscene.hue = (cutscene.hue + dt).fract();
            if let Some(section) = text.sections.get_mut(1) {
                section.style.color = Color::hsl(cutscene.hue * 360.0, 1.0, 0.5);
            }
        }
    }

    pub fn advance(
        mut commands: Commands,
        mut cutscene_query: Query<(Entity, &mut Text, &mut Cutscene)>,
        mut player_query: Query<&mut PlayerControl>,
        audio: Res<Audio>,
    ) {
        let Ok(mut player) = player_query.get_single_mut() else { return };

        for (entity, mut text, mut cutscene) in &mut cutscene_query {
            if cutscene.phase >= NUM_LINES {
                player.deny_input = false;
                commands.entity(entity).despawn_recursive();
                return;
            }

            if cutscene.phase == NUM_LINES - 1 {
                cutscene.section += 1;
            }

            text.sections[cutscene.section].value = format!(
                "{}\n\n\n\n{}",
                text.sections[cutscene.section].value, TEXT_LINES[cutscene.phase]
            );

            audio.play_with_settings(
                cutscene.sounds[cutscene.phase].clone(),
                PlaybackSettings::default().with_volume(LINE_VOLUMES[cutscene.phase]),
            );

            cutscene.phase += 1;
        }
    }
}

pub struct CutsceneTemplate;

impl CutsceneTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        let text_style = TextStyle {
            font_size: 18.0,
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
                        bottom: Val::Percent(60.0),
                    },
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
            Cutscene {
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
        entity.insert(Name::new("Cutscene"));

        entity.id()
    }
}

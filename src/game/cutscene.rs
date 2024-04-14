use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use leafwing_input_manager::common_conditions::action_just_pressed;

use crate::common::asset::AudioKey;
use crate::common::asset::FontKey;
use crate::common::asset::Handles;
use crate::common::GameAction;
use crate::common::UpdateSet;
use crate::game::actor::enemy::Alarm;
use crate::game::actor::player::PlayerControl;
use crate::game::actor::player::Playthrough;
use crate::game::actor::ActorIntent;
use crate::game::actor::Health;
use crate::game::map::Victory;
use crate::util::DespawnSet;

pub struct CutscenePlugin;

impl Plugin for CutscenePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Cutscene>().add_systems(
            Update,
            (
                Cutscene::update.in_set(UpdateSet::Update),
                Cutscene::advance
                    .run_if(action_just_pressed(GameAction::Confirm))
                    .in_set(UpdateSet::HandleActions),
            ),
        );

        app.register_type::<Message>().add_systems(
            Update,
            (Message::show_death_message, Message::show_victory_message),
        );
    }
}

const NUM_LINES: usize = 3;
const TEXT_LINES: [&str; NUM_LINES] = ["You are Sai.", "You have chosen to Defect.", "GOOD LUCK!"];
const LINE_VOLUMES: [f64; NUM_LINES] = [1.0, 1.0, 0.3];

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
        if let Ok(mut player) = player_query.get_single_mut() {
            player.deny_input = !cutscene_query.is_empty();
        };

        let dt = time.delta_seconds();
        for (mut text, mut cutscene) in &mut cutscene_query {
            cutscene.hue = (cutscene.hue + dt).fract();
            if let Some(section) = text.sections.get_mut(1) {
                section.style.color = Color::hsl(cutscene.hue * 360.0, 1.0, 0.5);
            }
        }
    }

    pub fn advance(
        mut despawn: ResMut<DespawnSet>,
        mut cutscene_query: Query<(Entity, &mut Text, &mut Cutscene)>,
        audio: Res<Audio>,
    ) {
        for (entity, mut text, mut cutscene) in &mut cutscene_query {
            if cutscene.phase >= NUM_LINES {
                despawn.recursive(entity);
                continue;
            }

            if cutscene.phase == NUM_LINES - 1 {
                cutscene.section += 1;
            }

            text.sections[cutscene.section].value = format!(
                "{}\n\n\n\n{}",
                text.sections[cutscene.section].value, TEXT_LINES[cutscene.phase]
            );

            audio
                .play(cutscene.sounds[cutscene.phase].clone())
                .with_volume(LINE_VOLUMES[cutscene.phase]);

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
        #[cfg(feature = "dev")]
        entity.insert(Name::new("Cutscene"));

        entity.id()
    }
}

#[derive(Component, Reflect)]
pub struct Message;

pub struct MessageTemplate {
    title: String,
    body: String,
}

impl MessageTemplate {
    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        let title_style = TextStyle {
            font: handle.font[&FontKey::Pixel].clone(),
            font_size: 24.0,
            color: Color::WHITE,
        };
        let body_style = TextStyle {
            font: handle.font[&FontKey::Pixel].clone(),
            font_size: 16.0,
            color: Color::WHITE,
        };

        let mut message = commands.spawn((
            TextBundle {
                style: Style {
                    margin: UiRect::all(Val::Auto),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                text: Text::from_sections([
                    TextSection::new(self.title + "\n\n\n\n\n", title_style),
                    TextSection::new(self.body, body_style),
                ])
                .with_alignment(TextAlignment::Center),
                ..default()
            },
            Message,
        ));
        #[cfg(feature = "dev")]
        message.insert(Name::new("Message"));

        message.id()
    }
}

impl Message {
    pub fn show_death_message(
        mut commands: Commands,
        handle: Res<Handles>,
        message_query: Query<(), With<Message>>,
        player_query: Query<(), (With<PlayerControl>, Without<ActorIntent>)>,
    ) {
        if !message_query.is_empty() || player_query.is_empty() {
            return;
        }

        MessageTemplate {
            title: "You died.".to_string(),
            body: "(press R to restart)".to_string(),
        }
        .spawn(&mut commands, &handle);
    }

    pub fn show_victory_message(
        mut commands: Commands,
        handle: Res<Handles>,
        message_query: Query<(), With<Message>>,
        health_query: Query<&Health, With<PlayerControl>>,
        playthrough: Res<Playthrough>,
        victory: Res<Victory>,
        alarm: Res<Alarm>,
        time: Res<Time>,
    ) {
        if !victory.0 || !message_query.is_empty() {
            return;
        }
        let Ok(health) = health_query.get_single() else {
            return;
        };

        let alarm_scale = 100_000.0;
        let alarm_t = 1.0 - alarm.0;
        let alarm_score = (alarm_scale * alarm_t).round() as i32;

        let health_scale = 10_000.0;
        let health_t = health.current / health.max;
        let health_score = (health_scale * health_t).round() as i32;

        let time_scale = 50_000.0 * 60.0;
        let time_t = time.elapsed_seconds() - playthrough.start_time;
        let time_score = (time_scale / time_t).round() as i32;

        let score = alarm_score + health_score + time_score;

        MessageTemplate {
            title: "You escaped!".to_string(),
            body: format!("Alarm score: {alarm_score}\n\n\n\n\nHealth score: {health_score}\n\n\n\n\nTime score: {time_score}\n\n\n\n\nTotal score: {score}\n\n\n\n\n(press R to play again)"),
        }
        .spawn(&mut commands, &handle);
    }
}

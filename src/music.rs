use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::{
    asset::{AudioKey, Handles},
    map::Victory,
    mob::player::Playthrough,
};

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct Music {
    pub key: Option<AudioKey>,
    pub track: Option<Handle<AudioInstance>>,
}

impl Music {
    pub fn update(
        mut music: ResMut<Music>,
        mut audio_instances: ResMut<Assets<AudioInstance>>,
        handle: Res<Handles>,
        playthrough: Res<Playthrough>,
        victory: Res<Victory>,
        audio: Res<Audio>,
    ) {
        let key = if victory.0 {
            Some(AudioKey::VictoryTrack)
        } else if playthrough.defected {
            Some(AudioKey::MainTrack)
        } else {
            None
        };
        if music.key == key {
            return;
        }

        music.key = key;

        // Stop current track
        if let Some(track) = &music.track {
            if let Some(instance) = audio_instances.get_mut(track) {
                instance.stop(AudioTween::default());
            }
        }

        // Start new track
        music.track = key.map(|key| {
            audio
                .play(handle.audio[&key].clone())
                .with_volume(0.4)
                .looped()
                .handle()
        });
    }
}

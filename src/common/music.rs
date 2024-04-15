use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::game::actor::player::Playthrough;
use crate::game::level::victory::Victory;

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MusicAssets>()
            .init_collection::<MusicAssets>();

        app.register_type::<Music>()
            .init_resource::<Music>()
            .add_systems(Update, update_music);
    }
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct MusicAssets {
    #[asset(path = "sound/music/game.wav")]
    main: Handle<AudioSource>,
    #[asset(path = "sound/music/victory.wav")]
    victory: Handle<AudioSource>,
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct Music {
    pub current: Option<Handle<AudioSource>>,
    pub track: Option<Handle<AudioInstance>>,
}

fn update_music(
    mut music: ResMut<Music>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    music_assets: Res<MusicAssets>,
    playthrough: Res<Playthrough>,
    victory: Res<Victory>,
    audio: Res<Audio>,
) {
    let next = if victory.0 {
        Some(music_assets.victory.clone())
    } else if playthrough.defected {
        Some(music_assets.main.clone())
    } else {
        None
    };
    if music.current == next {
        return;
    }

    music.current = next;

    // Stop current track
    if let Some(track) = &music.track {
        if let Some(instance) = audio_instances.get_mut(track) {
            instance.stop(AudioTween::default());
        }
    }

    // Start next track
    music.track = music
        .current
        .clone()
        .map(|source| audio.play(source).with_volume(0.4).looped().handle());
}

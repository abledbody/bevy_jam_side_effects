use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_ldtk::LdtkAsset;

#[derive(Reflect, FromReflect, Eq, PartialEq, Hash)]
pub enum ImageKey {
    RedGnoll,
    GreenGnoll,
    BlueGnoll,
    DropShadow,
}

#[derive(Reflect, FromReflect, Eq, PartialEq, Hash)]
pub enum AudioKey {
    GnollWalk,
	PlayerAttack1,
	PlayerAttack2,
	PlayerHit,
}

#[derive(Reflect, FromReflect, Eq, PartialEq, Hash)]
pub enum LevelKey {
    TestLevel,
}

const IMAGE_MAP: [(ImageKey, &str); 4] = [
    (ImageKey::RedGnoll, "sprites/character/RedGnoll.png"),
    (ImageKey::GreenGnoll, "sprites/character/GreenGnoll.png"),
    (ImageKey::BlueGnoll, "sprites/character/BlueGnoll.png"),
    (ImageKey::DropShadow, "sprites/vfx/DropShadow.png"),
];

const AUDIO_MAP: [(AudioKey, &str); 4] = [
	(AudioKey::GnollWalk, "sfx/walk.wav"),
	(AudioKey::PlayerAttack1, "sfx/player_attack_1.wav"),
	(AudioKey::PlayerAttack2, "sfx/player_attack_2.wav"),
	(AudioKey::PlayerHit, "sfx/player_hit.wav"),
];

const LEVEL_MAP: [(LevelKey, &str); 1] = [(LevelKey::TestLevel, "maps/test_map.ldtk")];

#[derive(Resource, Reflect, Default)]
pub struct Handles {
    pub image: HashMap<ImageKey, Handle<Image>>,
    pub audio: HashMap<AudioKey, Handle<AudioSource>>,
    pub levels: HashMap<LevelKey, Handle<LdtkAsset>>,
}

impl Handles {
    pub fn load(asset: Res<AssetServer>, mut handle: ResMut<Self>) {
        handle.image = IMAGE_MAP
            .into_iter()
            .map(|(key, path)| (key, asset.load(path)))
            .collect();

        handle.audio = AUDIO_MAP
            .into_iter()
            .map(|(key, path)| (key, asset.load(path)))
            .collect();

        handle.levels = LEVEL_MAP
            .into_iter()
            .map(|(key, path)| (key, asset.load(path)))
            .collect();
    }
}

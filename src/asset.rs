use bevy::{prelude::*, utils::HashMap};

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
}

const IMAGE_MAP: [(ImageKey, &str); 4] = [
    (ImageKey::RedGnoll, "sprites/character/RedGnoll.png"),
    (ImageKey::GreenGnoll, "sprites/character/GreenGnoll.png"),
    (ImageKey::BlueGnoll, "sprites/character/BlueGnoll.png"),
    (ImageKey::DropShadow, "sprites/vfx/DropShadow.png"),
];

const AUDIO_MAP: [(AudioKey, &'static str); 1] = [(AudioKey::GnollWalk, "sfx/walk.wav")];

#[derive(Resource, Reflect, Default)]
pub struct Handles {
    pub image: HashMap<ImageKey, Handle<Image>>,
    pub audio: HashMap<AudioKey, Handle<AudioSource>>,
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
    }
}

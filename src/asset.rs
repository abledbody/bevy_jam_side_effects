use bevy::{prelude::*, utils::HashMap};

#[derive(Reflect, FromReflect, Eq, PartialEq, Hash)]
pub enum ImageKey {
    RedGnoll,
    GreenGnoll,
    BlueGnoll,
}

const IMAGE_MAP: [(ImageKey, &'static str); 3] = [
    (ImageKey::RedGnoll, "sprites/character/RedGnoll.png"),
    (ImageKey::GreenGnoll, "sprites/character/GreenGnoll.png"),
    (ImageKey::BlueGnoll, "sprites/character/BlueGnoll.png"),
];

#[derive(Resource, Reflect, Default)]
pub struct Handles {
    pub image: HashMap<ImageKey, Handle<Image>>,
}

impl Handles {
    pub fn load(asset: Res<AssetServer>, mut handle: ResMut<Self>) {
        handle.image = IMAGE_MAP
            .into_iter()
            .map(|(key, path)| (key, asset.load(path)))
            .collect();
    }
}

use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Reflect, FromReflect, Eq, PartialEq, Hash)]
pub enum ImageKey {
    Gnoll,
}

const IMAGE_MAP: [(ImageKey, &'static str); 1] = [(ImageKey::Gnoll, "sprites/character/Gnoll.png")];

#[derive(Resource, Reflect, Default)]
pub struct Handles {
    pub image: HashMap<ImageKey, Handle<Image>>,
}

impl Handles {
    pub fn load(mut commands: Commands, asset: Res<AssetServer>) {
        commands.insert_resource(Self {
            image: IMAGE_MAP
                .into_iter()
                .map(|(key, path)| (key, asset.load(path)))
                .collect(),
        });
    }
}

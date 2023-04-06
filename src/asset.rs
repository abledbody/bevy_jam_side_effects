use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_ldtk::LdtkAsset;

#[derive(Reflect, FromReflect, Eq, PartialEq, Hash)]
pub enum FontKey {
    Regular,
    Bold,
    Pixel,
}

const FONT_MAP: [(FontKey, &str); 3] = [
    (FontKey::Regular, "font/OpenSans-Regular.ttf"),
    (FontKey::Bold, "font/OpenSans-Bold.ttf"),
    (FontKey::Pixel, "font/Jaywalk.ttf"),
];

#[derive(Reflect, FromReflect, Eq, PartialEq, Hash)]
pub enum ImageKey {
    GnollRed,
    GnollGreen,
    GnollBlue,
    PlateUnpressed,
    PlatePressed,
    GateOpen,
    GateClosed,
    DropShadow,
}

const IMAGE_MAP: [(ImageKey, &str); 8] = [
    (ImageKey::GnollRed, "sprites/character/gnoll_red.png"),
    (ImageKey::GnollGreen, "sprites/character/gnoll_green.png"),
    (ImageKey::GnollBlue, "sprites/character/gnoll_blue.png"),
    (
        ImageKey::PlateUnpressed,
        "sprites/interactive/plate_unpressed.png",
    ),
    (
        ImageKey::PlatePressed,
        "sprites/interactive/plate_pressed.png",
    ),
    (ImageKey::GateOpen, "sprites/interactive/gate_open.png"),
    (ImageKey::GateClosed, "sprites/interactive/gate_closed.png"),
    (ImageKey::DropShadow, "sprites/vfx/drop_shadow.png"),
];

#[derive(Reflect, FromReflect, Eq, PartialEq, Hash)]
pub enum AudioKey {
    GnollWalk,
    PlayerAttack1,
    PlayerAttack2,
    PlayerAttack3,
    PlayerAttack4,
    PlayerHit,
    PlayerAttackMiss,
}

const AUDIO_MAP: [(AudioKey, &str); 7] = [
    (AudioKey::GnollWalk, "sfx/walk.wav"),
    (AudioKey::PlayerAttack1, "sfx/player_attack_1.wav"),
    (AudioKey::PlayerAttack2, "sfx/player_attack_2.wav"),
    (AudioKey::PlayerAttack3, "sfx/player_attack_3.wav"),
    (AudioKey::PlayerAttack4, "sfx/player_attack_4.wav"),
    (AudioKey::PlayerHit, "sfx/player_hit.wav"),
    (AudioKey::PlayerAttackMiss, "sfx/player_attack_miss.wav"),
];

#[derive(Reflect, FromReflect, Eq, PartialEq, Hash)]
pub enum LevelKey {
    GameMap,
}

const LEVEL_MAP: [(LevelKey, &str); 1] = [(LevelKey::GameMap, "maps/game_map.ldtk")];

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct Handles {
    pub font: HashMap<FontKey, Handle<Font>>,
    pub image: HashMap<ImageKey, Handle<Image>>,
    pub audio: HashMap<AudioKey, Handle<AudioSource>>,
    pub levels: HashMap<LevelKey, Handle<LdtkAsset>>,
}

impl Handles {
    pub fn load(asset: Res<AssetServer>, mut handle: ResMut<Self>) {
        handle.font = FONT_MAP
            .into_iter()
            .map(|(key, path)| (key, asset.load(path)))
            .collect();

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

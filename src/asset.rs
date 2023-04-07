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
    DetectPopup,
}

const IMAGE_MAP: [(ImageKey, &str); 9] = [
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
    (ImageKey::DetectPopup, "sprites/vfx/detect_popup.png"),
];

#[derive(Reflect, FromReflect, Eq, PartialEq, Hash)]
pub enum AudioKey {
    GnollWalk,
    GnollAttack1,
    GnollAttack2,
    GnollAttack3,
    GnollAttack4,
    GnollAttackHit,
    GnollAttackMiss,
    GnollHurt,
    GnollDetect,
    PlateTriggerGate,
    Pop1,
    Pop2,
    Jackpot,
}

const AUDIO_MAP: [(AudioKey, &str); 13] = [
    (AudioKey::GnollWalk, "sfx/walk.wav"),
    (AudioKey::GnollAttack1, "sfx/player_attack_1.wav"),
    (AudioKey::GnollAttack2, "sfx/player_attack_2.wav"),
    (AudioKey::GnollAttack3, "sfx/player_attack_3.wav"),
    (AudioKey::GnollAttack4, "sfx/player_attack_4.wav"),
    (AudioKey::GnollAttackHit, "sfx/player_hit.wav"),
    (AudioKey::GnollAttackMiss, "sfx/player_attack_miss.wav"),
    (AudioKey::GnollHurt, "sfx/gnoll_hurt.wav"),
    (AudioKey::GnollDetect, "sfx/alert.wav"),
    (AudioKey::PlateTriggerGate, "sfx/button_gate.wav"),
    (AudioKey::Pop1, "sfx/pop_1.wav"),
    (AudioKey::Pop2, "sfx/pop_2.wav"),
    (AudioKey::Jackpot, "sfx/jackpot.wav"),
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

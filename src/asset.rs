use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_ldtk::LdtkAsset;
use bevy_kira_audio::prelude::*;

#[derive(Reflect, FromReflect, Copy, Clone, Eq, PartialEq, Hash)]
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

#[derive(Reflect, FromReflect, Copy, Clone, Eq, PartialEq, Hash)]
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
    AlarmMeterIcon,
    AlarmMeterIconFlash,
}

const IMAGE_MAP: [(ImageKey, &str); 11] = [
    (ImageKey::GnollRed, "image/character/gnoll_red.png"),
    (ImageKey::GnollGreen, "image/character/gnoll_green.png"),
    (ImageKey::GnollBlue, "image/character/gnoll_blue.png"),
    (
        ImageKey::PlateUnpressed,
        "image/interactive/plate_unpressed.png",
    ),
    (
        ImageKey::PlatePressed,
        "image/interactive/plate_pressed.png",
    ),
    (ImageKey::GateOpen, "image/interactive/gate_open.png"),
    (ImageKey::GateClosed, "image/interactive/gate_closed.png"),
    (ImageKey::DropShadow, "image/vfx/drop_shadow.png"),
    (ImageKey::DetectPopup, "image/vfx/detect_popup.png"),
    (ImageKey::AlarmMeterIcon, "image/ui/alarm.png"),
    (ImageKey::AlarmMeterIconFlash, "image/ui/alarm_flash.png"),
];

#[derive(Reflect, FromReflect, Copy, Clone, Eq, PartialEq, Hash)]
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
    MainTrack,
    VictoryTrack,
}

const AUDIO_MAP: [(AudioKey, &str); 15] = [
    (AudioKey::GnollWalk, "sound/sfx/walk.wav"),
    (AudioKey::GnollAttack1, "sound/sfx/player_attack_1.wav"),
    (AudioKey::GnollAttack2, "sound/sfx/player_attack_2.wav"),
    (AudioKey::GnollAttack3, "sound/sfx/player_attack_3.wav"),
    (AudioKey::GnollAttack4, "sound/sfx/player_attack_4.wav"),
    (AudioKey::GnollAttackHit, "sound/sfx/player_hit.wav"),
    (
        AudioKey::GnollAttackMiss,
        "sound/sfx/player_attack_miss.wav",
    ),
    (AudioKey::GnollHurt, "sound/sfx/gnoll_hurt.wav"),
    (AudioKey::GnollDetect, "sound/sfx/alert.wav"),
    (AudioKey::PlateTriggerGate, "sound/sfx/button_gate.wav"),
    (AudioKey::Pop1, "sound/sfx/pop_1.wav"),
    (AudioKey::Pop2, "sound/sfx/pop_2.wav"),
    (AudioKey::Jackpot, "sound/sfx/jackpot.mp3"),
    (AudioKey::MainTrack, "sound/music/game.mp3"),
    (AudioKey::VictoryTrack, "sound/music/victory.mp3"),
];

#[derive(Reflect, FromReflect, Copy, Clone, Eq, PartialEq, Hash)]
pub enum MapKey {
    Game,
}

const MAP_MAP: [(MapKey, &str); 1] = [(MapKey::Game, "map/game_map.ldtk")];

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct Handles {
    pub font: HashMap<FontKey, Handle<Font>>,
    pub image: HashMap<ImageKey, Handle<Image>>,
    pub audio: HashMap<AudioKey, Handle<AudioSource>>,
    pub map: HashMap<MapKey, Handle<LdtkAsset>>,
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

        handle.map = MAP_MAP
            .into_iter()
            .map(|(key, path)| (key, asset.load(path)))
            .collect();
    }
}

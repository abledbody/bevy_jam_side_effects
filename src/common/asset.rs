// TODO: Use bevy_asset_loader instead

use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_ldtk::assets::LdtkProject;
use bevy_kira_audio::prelude::*;

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Handles>()
            .init_resource::<Handles>()
            .add_systems(PreStartup, load_handles);
    }
}

#[derive(Reflect, Copy, Clone, Eq, PartialEq, Hash)]
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

#[derive(Reflect, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AudioKey {
    GnollWalk,
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

const AUDIO_MAP: [(AudioKey, &str); 11] = [
    (AudioKey::GnollWalk, "sound/sfx/walk.wav"),
    (AudioKey::GnollAttackHit, "sound/sfx/gnoll_attack_hit.wav"),
    (AudioKey::GnollAttackMiss, "sound/sfx/gnoll_attack_miss.wav"),
    (AudioKey::GnollHurt, "sound/sfx/gnoll_hurt.wav"),
    (AudioKey::GnollDetect, "sound/sfx/alert.wav"),
    (AudioKey::PlateTriggerGate, "sound/sfx/button_gate.wav"),
    (AudioKey::Pop1, "sound/sfx/pop_1.wav"),
    (AudioKey::Pop2, "sound/sfx/pop_2.wav"),
    (AudioKey::Jackpot, "sound/sfx/jackpot.wav"),
    (AudioKey::MainTrack, "sound/music/game.wav"),
    (AudioKey::VictoryTrack, "sound/music/victory.wav"),
];

#[derive(Reflect, Copy, Clone, Eq, PartialEq, Hash)]
pub enum MapKey {
    Game,
}

const MAP_MAP: [(MapKey, &str); 1] = [(MapKey::Game, "map/game_map.ldtk")];

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct Handles {
    pub image: HashMap<ImageKey, Handle<Image>>,
    pub audio: HashMap<AudioKey, Handle<AudioSource>>,
    pub map: HashMap<MapKey, Handle<LdtkProject>>,
}

fn load_handles(asset: Res<AssetServer>, mut handle: ResMut<Handles>) {
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

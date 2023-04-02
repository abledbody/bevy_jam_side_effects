use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::asset::Handles;
use crate::asset::ImageKey;
use crate::debug::DebugPlugin;
use crate::mob::Mob;
use crate::mob::MobInputs;
use crate::player::Player;

const TITLE: &'static str = "My Title";
const CLEAR_COLOR: Color = Color::DARK_GRAY;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Resources
        app.insert_resource(ClearColor(CLEAR_COLOR))
            .init_resource::<Handles>();

        // Plugins
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: TITLE.to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default());
        #[cfg(feature = "debug_mode")]
        app.add_plugin(DebugPlugin::default());

        // Startup systems
        app.add_startup_systems((
            spawn_camera,
            Handles::load,
            spawn_player.after(Handles::load),
        ));

        // UI systems
        app.add_system(bevy::window::close_on_esc)
            .add_system(Player::record_controls)
            .add_system(Mob::apply_input);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_player(mut commands: Commands, handle: Res<Handles>) {
    commands.spawn((
        SpriteBundle {
            texture: handle.image[&ImageKey::GreenGnoll].clone(),
            ..default()
        },
        Mob::default(),
        MobInputs::default(),
        Player,
        Velocity::default(),
        RigidBody::default(),
        LockedAxes::ROTATION_LOCKED,
    ));
}

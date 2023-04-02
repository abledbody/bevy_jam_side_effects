use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::asset::Handles;
use crate::asset::ImageKey;
use crate::debug::DebugPlugin;
use crate::mob::Health;
use crate::mob::Mob;
use crate::mob::MobInputs;
use crate::player::Player;

const TITLE: &'static str = "My Title";
const CLEAR_COLOR: Color = Color::DARK_GRAY;
pub const TIME_STEP: f32 = 1.0 / 60.0;
const TIME_STEP_DURATION: Duration = Duration::from_nanos((TIME_STEP * 1_000_000_000.0) as u64);

type Physics = RapierPhysicsPlugin<NoUserData>;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // Resources
        app.insert_resource(FixedTime::new(TIME_STEP_DURATION))
            .insert_resource(ClearColor(CLEAR_COLOR))
            .insert_resource(RapierConfiguration {
                gravity: Vec2::ZERO,
                timestep_mode: TimestepMode::Fixed {
                    dt: TIME_STEP,
                    substeps: 1,
                },
                ..default()
            })
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
        .add_plugin(Physics::default().with_default_system_setup(false));
        #[cfg(feature = "debug_mode")]
        app.add_plugin(DebugPlugin::default());

        // Startup systems
        app.add_startup_systems((
            spawn_camera,
            Handles::load,
            spawn_player.after(Handles::load),
        ));

        // Game logic systems (fixed timestep)
        app.edit_schedule(CoreSchedule::FixedUpdate, |schedule| {
            schedule.add_systems(
                (Player::record_controls, Mob::apply_input)
                    .chain()
                    .before(PhysicsSet::SyncBackend),
            );

            // Physics
            for set in [
                PhysicsSet::SyncBackend,
                PhysicsSet::SyncBackendFlush,
                PhysicsSet::StepSimulation,
                PhysicsSet::Writeback,
            ] {
                schedule.add_systems(Physics::get_systems(set.clone()).in_base_set(set));
            }
        });

        // UI systems
        app.add_system(bevy::window::close_on_esc);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_player(mut commands: Commands, handle: Res<Handles>) {
    let texture = ImageKey::GreenGnoll;
    let health = 100.0;

    commands.spawn((
        SpriteBundle {
            texture: handle.image[&texture].clone(),
            ..default()
        },
        Mob::player(),
        MobInputs::default(),
        Player,
        Health(health),
        (
            Velocity::default(),
            RigidBody::default(),
            LockedAxes::ROTATION_LOCKED,
        ),
    ));
}

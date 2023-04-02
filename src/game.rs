use std::{f32::consts::TAU, time::Duration};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    animation::{self, Facing, Offset, WalkAnimation},
    asset::Handles,
    mob::{enemy::Enemy, player::Player, Mob},
};

// TODO: Come up with a title.
const TITLE: &str = "My Title";
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
        app.add_plugin(crate::debug::DebugPlugin::default());

        // Startup systems
        app.add_startup_systems((spawn_camera, Handles::load));
        app.add_startup_systems((Player::spawn, spawn_enemies).after(Handles::load));

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

        // Visual systems
        app.add_systems((
            Mob::set_facing,
            Facing::update_sprites.after(Mob::set_facing),
            Offset::apply.after(Mob::set_facing),
            WalkAnimation::update,
            animation::sum_animations,
        ));

        // UI systems
        app.add_system(bevy::window::close_on_esc);
    }
}

fn spawn_camera(mut commands: Commands) {
    let projection = OrthographicProjection {
        // TODO: Scale to screen resolution
        scale: 1.0 / 4.0,
        ..default()
    };
    commands.spawn(Camera2dBundle {
        projection,
        ..default()
    });
}

fn spawn_enemies(mut commands: Commands, handle: Res<Handles>) {
    let distance = 80.0;
    let count = 12;
    for i in 0..count {
        let angle = i as f32 / count as f32 * TAU;
        let position = (distance * Vec2::from_angle(angle)).extend(400.0);
        Enemy::spawn(&mut commands, &handle, position);
    }
}

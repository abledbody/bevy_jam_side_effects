use bevy::{
    math::Vec3Swizzles,
    prelude::*,
    render::camera::{OrthographicProjection, ScalingMode},
};

use crate::mob::player::PlayerControl;

pub struct GameCameraTemplate;

impl GameCameraTemplate {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let projection = OrthographicProjection {
            scaling_mode: ScalingMode::AutoMax {
                max_width: 480.0,
                max_height: 270.0,
            },
            ..default()
        };

        let mut camera = commands.spawn((
            Camera2dBundle {
                projection,
                ..default()
            },
            GameCamera {
                rate: 5.0,
                ..default()
            },
        ));
        #[cfg(feature = "debug_mode")]
        camera.insert(Name::new("GameCamera"));

        camera.id()
    }
}

#[derive(Component, Reflect, Default)]
pub struct GameCamera {
    pub rate: f32,
}

impl GameCamera {
    pub fn cut_to_new_target(
        mut camera_query: Query<&mut Transform, With<GameCamera>>,
        target_query: Query<&GlobalTransform, Added<PlayerControl>>,
    ) {
        let Ok(mut camera_transform) = camera_query.get_single_mut() else { return };
        let Ok(target_transform) = target_query.get_single() else { return };

        let target_pos = target_transform.translation().xy();
        camera_transform.translation.x = target_pos.x;
        camera_transform.translation.y = target_pos.y;
    }

    pub fn follow_target(
        mut camera_query: Query<(&GameCamera, &mut Transform)>,
        target_query: Query<&GlobalTransform, With<PlayerControl>>,
        time: Res<Time>,
    ) {
        let Ok((camera, mut camera_transform)) = camera_query.get_single_mut() else { return };
        let Ok(target_transform) = target_query.get_single() else { return };

        let dt = time.delta_seconds();

        let camera_pos = camera_transform.translation.xy();
        let target_pos = target_transform.translation().xy();

        camera_transform.translation += camera_pos
            .smooth_approach(target_pos, camera.rate, dt)
            .extend(0.0);
    }
}

pub trait SmoothApproach {
    fn smooth_approach(self, target: Self, rate: f32, dt: f32) -> Self;
}

impl SmoothApproach for Vec2 {
    fn smooth_approach(self, target: Self, rate: f32, dt: f32) -> Self {
        (target - self) * dt * rate
    }
}

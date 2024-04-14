use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

use crate::common::PostTransformSet;
use crate::common::UpdateSet;
use crate::game::actor::player::PlayerControl;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa::Off);

        app.register_type::<CameraRoot>()
            .init_resource::<CameraRoot>();

        app.register_type::<GameCamera>().add_systems(
            PostUpdate,
            (snap_camera_to_new_target, camera_follow_target).in_set(PostTransformSet::Blend),
        );

        app.register_type::<AbsoluteScale>()
            .add_systems(Update, apply_absolute_scale.in_set(UpdateSet::End));
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct CameraRoot {
    pub primary: Entity,
}

impl FromWorld for CameraRoot {
    fn from_world(world: &mut World) -> Self {
        Self {
            primary: world
                .spawn((
                    Name::new("PrimaryCamera"),
                    Camera2dBundle {
                        projection: OrthographicProjection {
                            near: -1000.0,
                            scaling_mode: ScalingMode::AutoMax {
                                max_width: 480.0,
                                max_height: 270.0,
                            },
                            ..default()
                        },
                        ..default()
                    },
                    GameCamera { rate: 5.0 },
                ))
                .id(),
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct GameCamera {
    pub rate: f32,
}

fn snap_camera_to_new_target(
    mut camera_query: Query<&mut Transform, With<GameCamera>>,
    target_query: Query<&GlobalTransform, Added<PlayerControl>>,
) {
    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };
    let Ok(target_transform) = target_query.get_single() else {
        return;
    };

    let target_pos = target_transform.translation().xy();
    camera_transform.translation.x = target_pos.x;
    camera_transform.translation.y = target_pos.y;
}

fn camera_follow_target(
    mut camera_query: Query<(&GameCamera, &mut Transform)>,
    target_query: Query<&GlobalTransform, With<PlayerControl>>,
    time: Res<Time>,
) {
    let Ok((camera, mut camera_transform)) = camera_query.get_single_mut() else {
        return;
    };
    let Ok(target_transform) = target_query.get_single() else {
        return;
    };

    let dt = time.delta_seconds();

    let camera_pos = camera_transform.translation.xy();
    let target_pos = target_transform.translation().xy();

    camera_transform.translation = camera_pos
        .smooth_approach(target_pos, camera.rate, dt)
        .extend(camera_transform.translation.z);
}

pub trait SmoothApproach {
    fn smooth_approach(self, target: Self, rate: f32, dt: f32) -> Self;
}

impl SmoothApproach for f32 {
    fn smooth_approach(self, target: Self, rate: f32, dt: f32) -> Self {
        (self - target) / ((rate * dt) + 1.0) + target
    }
}

impl SmoothApproach for Vec2 {
    fn smooth_approach(self, target: Self, rate: f32, dt: f32) -> Self {
        Vec2::new(
            self.x.smooth_approach(target.x, rate, dt),
            self.y.smooth_approach(target.y, rate, dt),
        )
    }
}

// Camera zoom-independent scale
// (workaround for https://github.com/bevyengine/bevy/issues/1890)
#[derive(Component, Reflect)]
pub struct AbsoluteScale(pub Vec3);

impl Default for AbsoluteScale {
    fn default() -> Self {
        Self(Vec3::ONE)
    }
}

fn apply_absolute_scale(
    camera_root: Res<CameraRoot>,
    camera_query: Query<(&OrthographicProjection, &Camera)>,
    mut scale_query: Query<(&mut Transform, &AbsoluteScale)>,
) {
    let Ok((camera_proj, camera)) = camera_query.get(camera_root.primary) else {
        return;
    };
    let Some(viewport_size) = camera.logical_viewport_size() else {
        return;
    };

    let units_per_pixel = camera_proj.area.width() / viewport_size.x;
    let camera_scale_inverse = Vec2::splat(units_per_pixel).extend(1.0);
    for (mut transform, scale) in &mut scale_query {
        transform.scale = camera_scale_inverse * scale.0;
    }
}

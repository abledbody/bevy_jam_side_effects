use std::marker::PhantomData;

use bevy::{
    math::{vec2, vec3, Vec3Swizzles},
    prelude::*,
    render::camera::OrthographicProjection,
};
use bevy_ecs_ldtk::{LdtkLevel, LevelSelection};

use crate::mob::player::PlayerControl;

pub const CAMERA_SCALE: f32 = 1.0 / 4.0;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(CameraFollow::<PlayerControl>::apply)
            .add_systems(
                (
                    CameraFollow::<PlayerControl>::follow,
                    camera_fit_inside_current_level,
                )
                    .chain(),
            );
    }
}

pub struct GameCameraTemplate<C: Component> {
    pub target: Entity,
    _c: PhantomData<C>,
}

impl<C: Component> Default for GameCameraTemplate<C> {
    fn default() -> Self {
        Self {
            target: Entity::PLACEHOLDER,
            _c: PhantomData,
        }
    }
}

impl<C: Component> GameCameraTemplate<C> {
    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let projection = OrthographicProjection {
            // TODO: Scale to screen resolution
            scale: CAMERA_SCALE,
            ..default()
        };

        let mut camera = commands.spawn((
            Camera2dBundle {
                projection,
                ..default()
            },
            CameraFollow::<C> {
                target: self.target,
                rate: 5.0,
                ..default()
            },
        ));
        #[cfg(feature = "debug_mode")]
        camera.insert(Name::new("GameCamera"));

        camera.id()
    }
}

#[derive(Component, Reflect)]
pub struct CameraFollow<C: Component> {
    pub target: Entity,
    pub rate: f32,
    #[reflect(ignore)]
    _c: PhantomData<C>,
}

impl<C: Component> Default for CameraFollow<C> {
    fn default() -> Self {
        Self {
            target: Entity::PLACEHOLDER,
            rate: 0.0,
            _c: PhantomData,
        }
    }
}

impl<C: Component> CameraFollow<C> {
    fn follow(
        mut follow_query: Query<&mut CameraFollow<C>>,
        target_query: Query<Entity, Added<C>>,
    ) {
        for mut follow in &mut follow_query {
            if let Ok(target) = target_query.get_single() {
                follow.target = target;
            }
        }
    }

    fn apply(
        mut camera_query: Query<(&CameraFollow<C>, &mut Transform)>,
        transform_query: Query<&Transform, Without<CameraFollow<C>>>,
        time: Res<Time>,
    ) {
        for (follow, mut transform) in &mut camera_query {
            if let Ok(&target) = transform_query.get(follow.target) {
                transform.translation.x = transform.translation.x.smooth_approach(
                    target.translation.x,
                    follow.rate,
                    time.delta_seconds(),
                );
                transform.translation.y = transform.translation.y.smooth_approach(
                    target.translation.y,
                    follow.rate,
                    time.delta_seconds(),
                );
            }
        }
    }
}

pub trait SmoothApproach {
    fn smooth_approach(self, target: Self, rate: f32, dt: f32) -> Self;
}

impl SmoothApproach for f32 {
    fn smooth_approach(self, target: Self, rate: f32, dt: f32) -> Self {
        (self - target) / ((rate * dt) + 1.0) + target
    }
}

fn camera_fit_inside_current_level(
    mut camera_query: Query<
        (&Camera, &mut Transform, &GlobalTransform),
        With<CameraFollow<PlayerControl>>,
    >,
    level_query: Query<
        (&GlobalTransform, &Handle<LdtkLevel>),
        Without<CameraFollow<PlayerControl>>,
    >,
    level_selection: Res<LevelSelection>,
    ldtk_levels: Res<Assets<LdtkLevel>>,
) {
    let (camera, mut cam_transform, cam_gt) = camera_query.single_mut();

    for (level_transform, level_handle) in &level_query {
        if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
            let level = &ldtk_level.level;
            let level_translation = level_transform.translation();
            if level_selection.is_match(&0, level) {
                let cam_bottom_left = camera
                    .ndc_to_world(&cam_gt, -vec3(1.0, 1.0, 0.0))
                    .unwrap()
                    .xy();
                let cam_top_right = camera
                    .ndc_to_world(&cam_gt, vec3(1.0, 1.0, 0.0))
                    .unwrap()
                    .xy();
                // The level's origin is at the bottom left corner
                let level_extents = vec2(level.px_wid as f32, level.px_hei as f32);
                let level_bottom_left = level_translation.xy();
                let level_top_right = level_translation.xy() + level_extents;

                // Vector pointing from camera bounds to level bounds
                // If x or y are positive the screen is out of bounds and must shift
                // If x or y are negative, ignore them as they are in bounds
                let shift_up_right = (level_bottom_left - cam_bottom_left)
                    .max(Vec2::ZERO)
                    .extend(0.0);
                // Same as above in reverse: use negative values ignore positive ones
                let shift_down_left = (level_top_right - cam_top_right)
                    .min(Vec2::ZERO)
                    .extend(0.0);

                cam_transform.translation += shift_up_right + shift_down_left;
            }
        }
    }
}

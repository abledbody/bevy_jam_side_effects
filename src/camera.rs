use std::marker::PhantomData;

use bevy::{
    math::{vec2, Vec3Swizzles},
    prelude::*,
    render::camera::{OrthographicProjection, ScalingMode},
};
use bevy_ecs_ldtk::{LdtkLevel, LevelSelection};

use crate::mob::player::PlayerControl;

pub const CAMERA_SCALE: f32 = 1.0 / 4.0;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(CameraFollow::<PlayerControl>::apply)
            .add_system(CameraFollow::<PlayerControl>::follow);
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
        mut camera_query: Query<(
            &CameraFollow<C>,
            &mut Transform,
            &GlobalTransform,
            &mut OrthographicProjection,
        )>,
        transform_query: Query<
            &GlobalTransform,
            (Without<CameraFollow<C>>, Without<Handle<LdtkLevel>>),
        >,
        level_query: Query<(&GlobalTransform, &Handle<LdtkLevel>), Without<CameraFollow<C>>>,
        level_selection: Res<LevelSelection>,
        ldtk_levels: Res<Assets<LdtkLevel>>,
        time: Res<Time>,
    ) {
        for (follow, mut cam_transform, cam_gt, mut ortho) in &mut camera_query {
            let mut delta = Vec3::ZERO;
            if let Ok(&target) = transform_query.get(follow.target) {
                // Store the delta from this translation so we account for it when locking the camera
                delta = cam_gt
                    .translation()
                    .xy()
                    .smooth_approach(target.translation().xy(), follow.rate, time.delta_seconds())
                    .extend(0.0);
                cam_transform.translation += delta;
            };
            for (level_transform, level_handle) in &level_query {
                if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
                    let level = &ldtk_level.level;
                    let level_translation = level_transform.translation();
                    if level_selection.is_match(&0, level) {
                        let cam_gt = (cam_gt.translation() + delta).xy();
                        let cam_bottom_left = cam_gt + ortho.area.min;
                        let cam_top_right = cam_gt + ortho.area.max;
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
                        ortho.scaling_mode = ScalingMode::AutoMax {
                            max_width: level_extents.x * 3.0,
                            max_height: level_extents.y * 3.0,
                        }
                    }
                }
            }
        }
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

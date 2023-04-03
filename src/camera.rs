use std::marker::PhantomData;

use bevy::prelude::*;

use crate::mob::player::PlayerControl;

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
            scale: 1.0 / 4.0,
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
                transform.translation.x = transform.translation.x.smooth_approach(target.translation.x, follow.rate, time.delta_seconds());
                transform.translation.y = transform.translation.y.smooth_approach(target.translation.y, follow.rate, time.delta_seconds());
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
use std::marker::PhantomData;

use bevy::prelude::*;

use crate::mob::player::Player;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(CameraFollow::<Player>::apply)
            .add_system(CameraFollow::<Player>::follow);
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
    #[reflect(ignore)]
    _c: PhantomData<C>,
}

impl<C: Component> Default for CameraFollow<C> {
    fn default() -> Self {
        Self {
            target: Entity::PLACEHOLDER,
            _c: PhantomData,
        }
    }
}

impl<C: Component> CameraFollow<C> {
    fn follow(mut follow: Query<&mut CameraFollow<C>>, target: Query<Entity, Added<C>>) {
        for mut follow in &mut follow {
            for target in &target {
                follow.target = target;
            }
        }
    }

    fn apply(
        mut camera_query: Query<(&CameraFollow<C>, &mut Transform)>,
        transform_query: Query<&Transform, Without<CameraFollow<C>>>,
    ) {
        for (follow, mut transform) in &mut camera_query {
            if let Ok(&target) = transform_query.get(follow.target) {
                transform.translation.x = target.translation.x;
                transform.translation.y = target.translation.y;
            }
        }
    }
}

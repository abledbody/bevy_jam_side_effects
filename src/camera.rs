use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(CameraFollow::apply);
    }
}

pub struct GameCameraTemplate {
    pub target: Entity,
}

impl Default for GameCameraTemplate {
    fn default() -> Self {
        Self {
            target: Entity::PLACEHOLDER,
        }
    }
}

impl GameCameraTemplate {
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
            CameraFollow(self.target),
        ));
        #[cfg(feature = "debug_mode")]
        camera.insert(Name::new("GameCamera"));

        camera.id()
    }
}

#[derive(Component, Reflect)]
pub struct CameraFollow(pub Entity);

impl CameraFollow {
    fn apply(
        mut camera_query: Query<(&CameraFollow, &mut Transform)>,
        transform_query: Query<&Transform, Without<CameraFollow>>,
    ) {
        for (follow, mut transform) in &mut camera_query {
            if let Ok(&target) = transform_query.get(follow.0) {
                transform.translation.x = target.translation.x;
                transform.translation.y = target.translation.y;
            }
        }
    }
}

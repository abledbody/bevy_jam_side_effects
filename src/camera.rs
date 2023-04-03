use bevy::prelude::*;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(GameCamera::spawn)
            .add_system(GameCamera::follow);
    }
}

#[derive(Component)]
pub struct CameraFollow;

#[derive(Resource)]
pub struct CameraTarget(pub Entity);

pub struct GameCamera;

impl GameCamera {
    pub fn spawn(mut commands: Commands) {
        let projection = OrthographicProjection {
            // TODO: Scale to screen resolution
            scale: 1.0 / 4.0,
            ..default()
        };
        commands
            .spawn(Camera2dBundle {
                projection,
                ..default()
            })
            .insert(CameraFollow);
    }

    fn follow(
        mut camera: Query<&mut Transform, With<CameraFollow>>,
        transforms: Query<&Transform, Without<CameraFollow>>,
        target: Res<CameraTarget>,
    ) {
        for mut transform in &mut camera {
            if let Ok(followed) = transforms.get(target.0) {
                *transform = *followed;
            }
        }
    }
}

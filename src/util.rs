use bevy::{prelude::*, utils::HashSet};

pub const Z_MAX: f32 = 10.0;
pub const Z_SCALE: f32 = 0.001;

#[derive(Component, Reflect, Default, Clone, Debug)]
pub struct ZRampByY;

impl ZRampByY {
    pub fn apply(mut transform_query: Query<(&mut Transform, &GlobalTransform), With<ZRampByY>>) {
        for (mut transform, gt) in &mut transform_query {
            transform.translation.z = Z_MAX - Z_SCALE * gt.translation().y;
        }
    }
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct DespawnSet(pub HashSet<Entity>);

impl DespawnSet {
    pub fn apply(mut commands: Commands, mut despawn: ResMut<Self>) {
        for entity in despawn.0.drain() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

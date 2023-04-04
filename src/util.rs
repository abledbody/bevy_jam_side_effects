use bevy::{prelude::*, utils::HashSet};

pub const Z_MAX: f32 = 10.0;
pub const Z_SCALE: f32 = 0.001;

#[derive(Default, Clone, Debug, Component, Reflect)]
pub struct ZRampByY;

impl ZRampByY {
    pub fn apply(mut transform_query: Query<&mut Transform, With<ZRampByY>>) {
        for mut transform in &mut transform_query {
            transform.translation.z = Z_MAX - Z_SCALE * transform.translation.y;
        }
    }
}

#[derive(Resource, Reflect, Default)]
pub struct DespawnSet(pub HashSet<Entity>);

impl DespawnSet {
    pub fn apply(mut commands: Commands, mut despawn: ResMut<Self>) {
        for entity in despawn.0.drain() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

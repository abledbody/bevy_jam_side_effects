use bevy::prelude::*;

pub const Z_MAX: f32 = 10.0;
pub const Z_SCALE: f32 = 0.001;

#[derive(Component, Reflect, Default, Clone, Debug)]
pub struct YSort;

impl YSort {
    pub fn apply(mut transform_query: Query<(&mut Transform, &GlobalTransform), With<YSort>>) {
        for (mut transform, gt) in &mut transform_query {
            transform.translation.z = Z_MAX - Z_SCALE * gt.translation().y;
        }
    }
}

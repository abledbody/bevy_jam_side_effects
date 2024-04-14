use bevy::prelude::*;

use crate::common::PostTransformSet;

pub const Z_MAX: f32 = 10.0;
pub const Z_SCALE: f32 = 0.001;

pub struct YSortPlugin;

impl Plugin for YSortPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<YSort>()
            .add_systems(PostUpdate, y_sort.in_set(PostTransformSet::Blend));
    }
}

#[derive(Component, Reflect, Default, Clone, Debug)]
pub struct YSort;

fn y_sort(mut transform_query: Query<(&mut Transform, &GlobalTransform), With<YSort>>) {
    for (mut transform, gt) in &mut transform_query {
        transform.translation.z = Z_MAX - Z_SCALE * gt.translation().y;
    }
}

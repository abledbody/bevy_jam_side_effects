use bevy::prelude::*;

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

// An alternative to bevy hierarchy. Workaround for bevy rapier. Pair this with Offset.
#[derive(Component, Reflect)]
pub struct VirtualParent(pub Entity);

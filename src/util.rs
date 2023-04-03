use bevy::prelude::*;

pub const MOB_Z: f32 = 500.0;

#[derive(Debug, Component, Reflect)]
pub struct ZRampByY(pub f32);

impl ZRampByY {
    pub fn apply(mut transform_query: Query<(&Self, &mut Transform)>) {
        let scale = 0.01;
        for (z, mut transform) in &mut transform_query {
            transform.translation.z = (z.0 - scale * transform.translation.y).max(0.0);
        }
    }
}

// An alternative to bevy hierarchy. Workaround for bevy rapier. Pair this with Offset.
#[derive(Component, Reflect)]
pub struct VirtualParent(pub Entity);

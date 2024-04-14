use bevy::prelude::*;

use crate::common::UpdateSet;

pub struct FollowPlugin;

impl Plugin for FollowPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Follow>()
            .add_systems(Update, follow.in_set(UpdateSet::Update));
    }
}

// An alternative to bevy hierarchy. Workaround for bevy rapier. Pair this with Offset.
#[derive(Component, Reflect)]
pub struct Follow(pub Entity);

fn follow(mut follow_query: Query<(&Follow, &mut Transform)>, gt_query: Query<&GlobalTransform>) {
    for (follow, mut transform) in &mut follow_query {
        let Ok(&follow_gt) = gt_query.get(follow.0) else {
            continue;
        };

        *transform = follow_gt.compute_transform();
    }
}

use bevy::prelude::*;
// An alternative to bevy hierarchy. Workaround for bevy rapier. Pair this with Offset.
#[derive(Component, Reflect)]
pub struct Follow(pub Entity);

impl Follow {
    pub fn apply(
        mut follow_query: Query<(&Follow, &mut Transform)>,
        transform_query: Query<&Transform, Without<Follow>>,
    ) {
        for (follow, mut transform) in &mut follow_query {
            let Ok(&parent_transform) = transform_query.get(follow.0) else {
                continue;
            };

            *transform = parent_transform;
        }
    }
}

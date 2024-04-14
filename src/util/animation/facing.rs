use bevy::prelude::*;

use crate::common::PostTransformSet;
use crate::util::animation::follow::Follow;

pub struct FacingPlugin;

impl Plugin for FacingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Facing>().add_systems(
            PostUpdate,
            apply_facing.in_set(PostTransformSet::ApplyFacing),
        );
    }
}

#[derive(Component, Reflect, Debug, Default)]
pub enum Facing {
    Left,
    #[default]
    Right,
}

impl Facing {
    pub fn left(&self) -> bool {
        matches!(self, Facing::Left)
    }

    pub fn right(&self) -> bool {
        matches!(self, Facing::Right)
    }
}

fn apply_facing(
    parent_query: Query<(Entity, &Parent)>,
    follow_query: Query<(Entity, &Follow)>,
    facing_query: Query<&Facing>,
    mut transform_query: Query<&mut Transform>,
    mut sprite_query: Query<&mut Sprite>,
) {
    for (child, parent) in &parent_query {
        let Ok(facing) = facing_query.get(parent.get()) else {
            continue;
        };

        if let Ok(mut sprite) = sprite_query.get_mut(child) {
            sprite.flip_x = facing.left();
        };

        if let Ok(mut transform) = transform_query.get_mut(child) {
            if facing.left() {
                transform.translation.x = -transform.translation.x;
                transform.rotation = -transform.rotation;
            }
        }
    }

    for (child, follow) in &follow_query {
        let Ok(facing) = facing_query.get(follow.0) else {
            continue;
        };
        if facing.right() {
            continue;
        }
        let parent_x = {
            let Ok(parent_transform) = transform_query.get(follow.0) else {
                continue;
            };
            parent_transform.translation.x
        };
        let Ok(mut child_transform) = transform_query.get_mut(child) else {
            continue;
        };

        // Reflect child's X about parent's X
        child_transform.translation.x = 2.0 * parent_x - child_transform.translation.x;
    }
}

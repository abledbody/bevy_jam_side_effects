use bevy::prelude::*;

use crate::util::animation::follow::Follow;

#[derive(Component, Reflect, Default)]
pub struct Offset(pub Transform);

impl Offset {
    pub fn apply(
        mut offset_query: Query<(Entity, &Offset, &mut Transform)>,
        virtual_parent_query: Query<(), With<Follow>>,
    ) {
        for (entity, offset, mut transform) in &mut offset_query {
            if virtual_parent_query.contains(entity) {
                *transform = *transform * offset.0;
            } else {
                *transform = offset.0;
            }
        }
    }
}

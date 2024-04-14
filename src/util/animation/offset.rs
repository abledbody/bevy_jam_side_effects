use bevy::prelude::*;

use crate::common::PostTransformSet;
use crate::util::animation::follow::Follow;

pub struct OffsetPlugin;

impl Plugin for OffsetPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Offset>()
            .add_systems(PostUpdate, Offset::apply.in_set(PostTransformSet::Save));
    }
}

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

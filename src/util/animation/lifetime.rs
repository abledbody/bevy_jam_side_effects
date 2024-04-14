use bevy::prelude::*;

use crate::util::DespawnSet;

#[derive(Component, Reflect)]
pub struct Lifetime(pub f32);

impl Lifetime {
    pub fn apply(
        mut lifetime_query: Query<(Entity, &mut Self)>,
        time: Res<Time>,
        mut despawn: ResMut<DespawnSet>,
    ) {
        let dt = time.delta_seconds();
        for (entity, mut lifetime) in &mut lifetime_query {
            lifetime.0 -= dt;
            if lifetime.0 <= 0.0 {
                despawn.recursive(entity);
            }
        }
    }
}

use bevy::prelude::*;

use crate::common::UpdateSet;
use crate::util::DespawnSet;

pub struct LifetimePlugin;

impl Plugin for LifetimePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Lifetime>()
            .add_systems(Update, Lifetime::apply.in_set(UpdateSet::Start));
    }
}

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

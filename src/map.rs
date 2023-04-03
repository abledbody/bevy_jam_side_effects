use bevy::prelude::{Bundle, Component, Plugin};
use bevy_ecs_ldtk::{prelude::LdtkIntCellAppExt, LdtkIntCell, LdtkPlugin, LevelSelection};
use bevy_rapier2d::prelude::{Collider, CollisionGroups, Group};

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(LdtkPlugin)
            .insert_resource(LevelSelection::Index(0))
            .register_ldtk_int_cell::<WallBundle>(2);
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Wall;

#[derive(Clone, Debug, Bundle, LdtkIntCell)]
pub struct WallBundle {
    wall: Wall,
    collider: Collider,
    collision_groups: CollisionGroups,
}

impl Default for WallBundle {
    fn default() -> Self {
        Self {
            wall: Default::default(),
            collider: Collider::cuboid(20.0, 20.0),
            collision_groups: CollisionGroups {
                memberships: Group::ALL,
                filters: Group::ALL,
            },
        }
    }
}

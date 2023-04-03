use bevy::prelude::{default, Bundle, Component, Plugin};
use bevy_ecs_ldtk::{
    prelude::LdtkIntCellAppExt,
    IntGridCell,
    LdtkIntCell,
    LdtkPlugin,
    LevelSelection,
};
use bevy_rapier2d::prelude::{Collider, CollisionGroups, Friction, Group, RigidBody};

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

#[derive(Clone, Debug, Bundle)]
pub struct WallCollider {
    collider: Collider,
    collision_groups: CollisionGroups,
    rigid_body: RigidBody,
    friction: Friction,
}

#[derive(Clone, Debug, Bundle, LdtkIntCell)]
pub struct WallBundle {
    wall: Wall,
    #[bundle]
    #[from_int_grid_cell]
    collider: WallCollider,
}

impl From<IntGridCell> for WallCollider {
    fn from(_value: IntGridCell) -> Self {
        Self {
            collider: Collider::cuboid(8.0, 8.0),
            collision_groups: CollisionGroups {
                memberships: Group::ALL,
                filters: Group::ALL,
            },
            rigid_body: RigidBody::Fixed,
            friction: Friction {
                coefficient: 1.0,
                ..default()
            },
        }
    }
}

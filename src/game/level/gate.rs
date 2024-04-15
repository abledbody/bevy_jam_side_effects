use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::game::combat::COLLISION_GROUP;

pub struct GatePlugin;

impl Plugin for GatePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GateAssets>()
            .init_collection::<GateAssets>();

        app.register_type::<Gate>();
    }
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct GateAssets {
    #[asset(path = "image/interactive/gate_open.png")]
    pub open: Handle<Image>,
    #[asset(path = "image/interactive/gate_closed.png")]
    pub closed: Handle<Image>,
}

#[derive(Component, Reflect)]
pub struct Gate {
    pub open: bool,
}

pub struct GateTemplate {
    pub transform: Transform,
    pub open: bool,
}

impl GateTemplate {
    pub fn spawn(self, commands: &mut Commands, gate_assets: &GateAssets) -> Entity {
        let (filters, texture) = if self.open {
            (Group::empty(), gate_assets.open.clone())
        } else {
            (COLLISION_GROUP, gate_assets.closed.clone())
        };

        commands
            .spawn((
                Name::new("Gate"),
                SpriteBundle {
                    transform: self.transform,
                    texture,
                    ..default()
                },
                Collider::ball(8.0),
                CollisionGroups {
                    memberships: COLLISION_GROUP,
                    filters,
                },
                Friction::new(0.0),
                RigidBody::Fixed,
                Gate { open: self.open },
            ))
            .id()
    }
}

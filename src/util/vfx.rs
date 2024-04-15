use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::util::animation::lifetime::Lifetime;
use crate::util::animation::offset::Offset;

pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<VfxAssets>()
            .init_collection::<VfxAssets>();
    }
}

#[derive(AssetCollection, Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct VfxAssets {
    #[asset(path = "image/vfx/drop_shadow.png")]
    drop_shadow: Handle<Image>,
    #[asset(path = "image/vfx/alert_popup.png")]
    alert_popup: Handle<Image>,
}

pub struct DropShadowTemplate {
    pub offset: Transform,
}

impl Default for DropShadowTemplate {
    fn default() -> Self {
        Self {
            offset: Transform::from_xyz(0.0, 0.0, -0.01),
        }
    }
}

impl DropShadowTemplate {
    pub fn spawn(self, commands: &mut Commands, vfx_assets: &VfxAssets) -> Entity {
        commands
            .spawn((
                Name::new("DropShadow"),
                SpriteBundle {
                    texture: vfx_assets.drop_shadow.clone(),
                    ..default()
                },
                Offset(self.offset),
            ))
            .id()
    }
}

pub struct AlertPopupTemplate {
    pub offset: Transform,
}

impl Default for AlertPopupTemplate {
    fn default() -> Self {
        Self {
            offset: Transform::from_xyz(0.0, 0.0, 0.01),
        }
    }
}

impl AlertPopupTemplate {
    pub fn spawn(self, commands: &mut Commands, vfx_assets: &VfxAssets) -> Entity {
        commands
            .spawn((
                Name::new("AlertPopup"),
                SpriteBundle {
                    texture: vfx_assets.alert_popup.clone(),
                    ..default()
                },
                Lifetime(1.0),
                Offset(self.offset),
            ))
            .id()
    }
}

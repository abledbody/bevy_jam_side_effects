pub mod backdrop;
pub mod font;
pub mod health_bar;
pub mod interaction;
pub mod nametag;

use bevy::prelude::*;
use bevy::ui::Val::*;
use bevy_mod_picking::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<UiRoot>().init_resource::<UiRoot>();

        app.add_plugins((
            font::FontPlugin,
            health_bar::HealthBarPlugin,
            interaction::InteractionPlugin,
        ));
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct UiRoot {
    pub body: Entity,
}

impl FromWorld for UiRoot {
    fn from_world(world: &mut World) -> Self {
        Self {
            body: world
                .spawn((
                    Name::new("Ui"),
                    NodeBundle {
                        style: Style {
                            width: Percent(100.0),
                            height: Percent(100.0),
                            ..default()
                        },
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .id(),
        }
    }
}

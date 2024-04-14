use bevy::math::vec2;
use bevy::prelude::*;

use crate::common::asset::FontKey;
use crate::common::asset::Handles;
use crate::util::ui::backdrop::BackdropTemplate;
use crate::util::ui::font_size_hack::FontSizeHack;

pub struct NametagTemplate {
    pub offset: Transform,
    pub name: String,
}

impl Default for NametagTemplate {
    fn default() -> Self {
        Self {
            offset: default(),
            name: "Unnamed".to_string(),
        }
    }
}

impl NametagTemplate {
    const TEXT_COLOR: Color = Color::rgba(0.9, 0.9, 0.85, 0.8);

    pub fn spawn(self, commands: &mut Commands, handle: &Handles) -> Entity {
        let style = TextStyle {
            font: handle.font[&FontKey::Bold].clone(),
            font_size: 4.0,
            color: Self::TEXT_COLOR,
        };

        // Children
        let mut nametag = commands.spawn((
            Text2dBundle {
                text: Text::from_section(self.name, style.clone()),
                transform: Transform::from_xyz(0.0, 0.2, 0.001),
                ..default()
            },
            FontSizeHack(style.font_size),
        ));
        #[cfg(feature = "dev")]
        nametag.insert(Name::new("Nametag"));
        let nametag = nametag.id();

        // Parent
        let backdrop = BackdropTemplate {
            size: vec2(32.0, 4.0),
            offset: self.offset,
        }
        .spawn(commands);

        commands.entity(backdrop).add_child(nametag);

        backdrop
    }
}

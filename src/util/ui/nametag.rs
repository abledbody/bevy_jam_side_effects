use bevy::math::vec2;
use bevy::prelude::*;

use crate::common::asset::Handles;
use crate::util::ui::backdrop::BackdropTemplate;
use crate::util::ui::font::FontSize;
use crate::util::ui::font::BOLD_FONT_HANDLE;

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

    pub fn spawn(self, commands: &mut Commands) -> Entity {
        let style = TextStyle {
            font: BOLD_FONT_HANDLE,
            font_size: 16.0,
            color: Self::TEXT_COLOR,
        };

        // Children
        let mut nametag = commands.spawn(Text2dBundle {
            text: Text::from_section(self.name, style.clone()),
            transform: Transform::from_xyz(0.0, 0.0, 0.001)
                .with_scale(Vec2::splat(0.25).extend(1.0)),
            ..default()
        });
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

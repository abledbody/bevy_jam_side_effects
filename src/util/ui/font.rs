use bevy::asset::load_internal_binary_asset;
use bevy::prelude::*;
use bevy::text::Text2dBounds;

use crate::common::camera::CameraRoot;
use crate::common::window::WindowRoot;
use crate::common::UpdateSet;

pub struct FontPlugin;

impl Plugin for FontPlugin {
    fn build(&self, app: &mut App) {
        load_internal_binary_asset!(
            app,
            FONT_HANDLE,
            "../../../assets/font/OpenSans-Regular.ttf",
            |bytes: &[u8], _path: String| Font::try_from_bytes(bytes.to_vec()).unwrap()
        );
        load_internal_binary_asset!(
            app,
            BOLD_FONT_HANDLE,
            "../../../assets/font/OpenSans-Bold.ttf",
            |bytes: &[u8], _path: String| Font::try_from_bytes(bytes.to_vec()).unwrap()
        );
        load_internal_binary_asset!(
            app,
            PIXEL_FONT_HANDLE,
            "../../../assets/font/Jaywalk.ttf",
            |bytes: &[u8], _path: String| Font::try_from_bytes(bytes.to_vec()).unwrap()
        );

        app.register_type::<FontSize>()
            .add_systems(Update, apply_font_size.in_set(UpdateSet::End));
    }
}

pub const FONT_HANDLE: Handle<Font> =
    Handle::weak_from_u128(303551798864246209986336759745415587961);
pub const BOLD_FONT_HANDLE: Handle<Font> =
    Handle::weak_from_u128(317423448069604009516378143395193332978);
pub const PIXEL_FONT_HANDLE: Handle<Font> =
    Handle::weak_from_u128(93153499609634570285243616548722721367);

#[derive(Component, Reflect)]
pub struct FontSize {
    pub size: Val,
    pub step: f32,
    pub minimum: f32,
}

impl FontSize {
    pub fn new(size: Val) -> Self {
        Self {
            size,
            step: 0.0,
            minimum: 0.0,
        }
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.step = step;
        self.minimum = self.minimum.max(step);
        self
    }

    pub fn with_minimum(mut self, minimum: f32) -> Self {
        self.minimum = minimum;
        self
    }
}

pub fn apply_font_size(
    window_root: Res<WindowRoot>,
    window_query: Query<&Window>,
    mut text_query: Query<(&FontSize, &Node, &mut Text)>,
) {
    let Ok(window) = window_query.get(window_root.primary) else {
        return;
    };
    let viewport_size = Vec2::new(window.resolution.width(), window.resolution.height());

    for (font_size, node, mut text) in &mut text_query {
        // Compute font size
        let Ok(size) = font_size.size.resolve(node.size().x, viewport_size) else {
            continue;
        };

        // Round to nearest multiple of step
        let resolved = if font_size.step > 0.0 {
            (size / font_size.step).floor() * font_size.step
        } else {
            size
        };
        // Clamp above minimum
        let size = resolved.max(font_size.minimum);

        for section in &mut text.sections {
            section.style.font_size = size;
        }
    }
}

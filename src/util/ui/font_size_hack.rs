use bevy::prelude::*;
use bevy::text::Text2dBounds;
use bevy::utils::HashMap;

use crate::common::camera::GameCamera;

pub struct FontSizeHackPlugin;

impl Plugin for FontSizeHackPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FontSizeHack>()
            .add_systems(Update, apply_font_size_hack);
    }
}

#[derive(Component, Reflect)]
pub struct FontSizeHack(pub f32);

fn apply_font_size_hack(
    camera_query: Query<(&OrthographicProjection, &Camera), With<GameCamera>>,
    mut text_query: Query<(&mut Text, &mut Transform, &FontSizeHack)>,
) {
    let Ok((camera_proj, camera)) = camera_query.get_single() else {
        return;
    };
    let Some(viewport_size) = camera.logical_viewport_size() else {
        return;
    };

    let units_per_pixel = camera_proj.area.width() / viewport_size.x;
    let scale = Vec2::splat(units_per_pixel).extend(1.0);
    let max_font_size = 800.0;

    for (mut text, mut transform, font_size_hack) in &mut text_query {
        let font_size = font_size_hack.0 / units_per_pixel;
        let capped_font_size = font_size.min(max_font_size);
        transform.scale = scale * font_size / capped_font_size;
        debug!("Setting font size {capped_font_size}");
        for section in &mut text.sections {
            section.style.font_size = capped_font_size;
        }
    }
}

pub mod facing;
pub mod follow;
pub mod lifetime;
pub mod offset;

use bevy::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            facing::FacingPlugin,
            follow::FollowPlugin,
            lifetime::LifetimePlugin,
            offset::OffsetPlugin,
        ));
    }
}

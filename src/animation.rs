use std::f32::consts::PI;

use bevy::prelude::*;

use crate::{game::TIME_STEP, mob::MobInputs};

#[derive(Component, Reflect, Debug, Default)]
pub enum Facing {
    Left,
    #[default]
    Right,
}

impl Facing {
    pub fn update_sprites(
        facing_query: Query<(&Facing, &Children)>,
        mut sprite_query: Query<&mut Sprite>,
    ) {
        for (facing, children) in &facing_query {
            for child in children {
                let Ok(mut sprite) = sprite_query.get_mut(*child) else {
                    continue
                };

                sprite.flip_x = facing.left();
            }
        }
    }

    pub fn sign(&self) -> f32 {
        match self {
            Facing::Left => -1.0,
            Facing::Right => 1.0,
        }
    }

    pub fn left(&self) -> bool {
        if let Facing::Left = self {
            true
        } else {
            false
        }
    }

    pub fn right(&self) -> bool {
        if let Facing::Right = self {
            true
        } else {
            false
        }
    }
}

#[derive(Component, Reflect)]
pub struct Offset(pub Vec2);

impl Offset {
    pub fn apply_to_sprites(
        facing_query: Query<(&Facing, &Children)>,
        mut offset_query: Query<(&Offset, &mut Transform), With<Sprite>>,
    ) {
        for (facing, children) in &facing_query {
            for child in children {
                let Ok((offset, mut transform)) = offset_query.get_mut(*child) else {
                    continue
                };

                transform.translation.x = offset.0.x * facing.sign();
                transform.translation.y = offset.0.y;
            }
        }
    }

    pub fn apply_to_non_sprites(
        facing_query: Query<(&Facing, &Children)>,
        mut offset_query: Query<(&Offset, &mut Transform), Without<Sprite>>,
    ) {
        for (facing, children) in &facing_query {
            for child in children {
                let Ok((offset, mut transform)) = offset_query.get_mut(*child) else {
                    continue
                };

                transform.translation.x = offset.0.x * facing.sign();
                transform.translation.y = offset.0.y;
            }
        }
    }
}

#[derive(Component, Reflect)]
pub struct Lifetime(pub f32);

impl Lifetime {
    pub fn apply(mut commands: Commands, mut lifetime_query: Query<(Entity, &mut Self)>) {
        for (entity, mut lifetime) in &mut lifetime_query {
            lifetime.0 -= TIME_STEP;
            if lifetime.0 <= 0.0 {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct WalkAnimation {
    pub air_time: f32,
    pub height: f32,
    pub t: f32,
    pub sound: Option<Handle<AudioSource>>,
}

impl WalkAnimation {
    pub fn update(
        mob_query: Query<(&MobInputs, &Children)>,
        mut animator_query: Query<&mut WalkAnimation>,
        time: Res<Time>,
        audio: Res<Audio>,
    ) {
        for (mob_inputs, children) in &mob_query {
            let moving = mob_inputs.movement.length() != 0.0;

            for child in children {
                let Ok(mut walk_animation) = animator_query.get_mut(*child) else {continue;};

                if walk_animation.t <= 0.0 && !moving {
                    continue;
                }

                if let Some(sound) = &walk_animation.sound {
                    if walk_animation.t <= 0.0 {
                        audio.play_with_settings(
                            sound.clone(),
                            PlaybackSettings {
                                volume: 0.3,
                                ..default()
                            },
                        );
                    }
                }

                walk_animation.t += time.delta_seconds() / walk_animation.air_time;

                // The rest of this manages the loop, or lack thereof.
                if walk_animation.t < 1.0 {
                    continue;
                }
                if moving {
                    walk_animation.t -= walk_animation.t.floor();

                    if let Some(sound) = &walk_animation.sound {
                        audio.play_with_settings(
                            sound.clone(),
                            PlaybackSettings {
                                volume: 0.3,
                                ..default()
                            },
                        );
                    }
                } else {
                    walk_animation.t = 0.0;
                }
            }
        }
    }
}

pub fn sum_animations(mut sprite_query: Query<(&mut Offset, &WalkAnimation)>) {
    for (mut offset, walk_animation) in &mut sprite_query {
        // PI is used here because we only want half a rotation.
        offset.0.y = walk_animation.height * (walk_animation.t * PI).sin();
    }
}

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use crate::{mob::MobInputs, util::DespawnSet};

#[derive(Component, Reflect)]
pub struct Lifetime(pub f32);

impl Lifetime {
    pub fn apply(
        mut lifetime_query: Query<(Entity, &mut Self)>,
        time: Res<Time>,
        mut despawn: ResMut<DespawnSet>,
    ) {
        let dt = time.delta_seconds();
        for (entity, mut lifetime) in &mut lifetime_query {
            lifetime.0 -= dt;
            if lifetime.0 <= 0.0 {
                despawn.0.insert(entity);
            }
        }
    }
}

// An alternative to bevy hierarchy. Workaround for bevy rapier. Pair this with Offset.
#[derive(Component, Reflect)]
pub struct VirtualParent(pub Entity);

impl VirtualParent {
    pub fn copy_transform(
        mut virtual_parent_query: Query<(&VirtualParent, &mut Transform)>,
        transform_query: Query<&Transform, Without<VirtualParent>>,
    ) {
        for (virtual_parent, mut transform) in &mut virtual_parent_query {
            let Ok(&parent_transform) = transform_query.get(virtual_parent.0) else {
                continue
            };

            *transform = parent_transform;
        }
    }
}

#[derive(Component, Reflect)]
pub struct Offset(pub Vec2);

impl Offset {
    pub fn apply(
        mut offset_query: Query<(Entity, &Offset, &mut Transform)>,
        virtual_parent_query: Query<(), With<VirtualParent>>,
    ) {
        for (entity, offset, mut transform) in &mut offset_query {
            if virtual_parent_query.contains(entity) {
                transform.translation.x += offset.0.x;
                transform.translation.y += offset.0.y;
            } else {
                transform.translation.x = offset.0.x;
                transform.translation.y = offset.0.y;
                // FIXME: This is a hack.
                // Reset the rotation here so it can be animated each frame
                transform.rotation = Quat::IDENTITY;
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
        mut animation_query: Query<&mut WalkAnimation>,
        time: Res<Time>,
        audio: Res<Audio>,
    ) {
        let dt = time.delta_seconds();

        for (mob_inputs, children) in &mob_query {
            let moving = mob_inputs.movement.length() != 0.0;

            for &child in children {
                let Ok(mut anim) = animation_query.get_mut(child) else {
                    continue
                };

                if anim.t <= 0.0 && !moving {
                    continue;
                }

                if let Some(sound) = &anim.sound {
                    if anim.t <= 0.0 {
                        audio.play_with_settings(
                            sound.clone(),
                            PlaybackSettings {
                                volume: 0.3,
                                ..default()
                            },
                        );
                    }
                }

                anim.t += dt / anim.air_time;

                // The rest of this manages the loop, or lack thereof.
                if anim.t < 1.0 {
                    continue;
                }
                if moving {
                    anim.t = anim.t.fract();

                    if let Some(sound) = &anim.sound {
                        audio.play_with_settings(
                            sound.clone(),
                            PlaybackSettings {
                                volume: 0.3,
                                ..default()
                            },
                        );
                    }
                } else {
                    anim.t = 0.0;
                }
            }
        }
    }

    pub fn apply(mut animation_query: Query<(&WalkAnimation, &mut Transform)>) {
        for (anim, mut transform) in &mut animation_query {
            // PI is used here because we only want half a rotation.
            transform.translation.y += anim.height * (anim.t * PI).sin();
        }
    }
}

#[derive(Component, Reflect)]
pub struct DeathAnimation {
    pub air_time: f32,
    pub height: f32,
    pub rotate_time: f32,
    pub air_t: f32,
    pub rot_t: f32,
}

impl Default for DeathAnimation {
    fn default() -> Self {
        Self {
            air_time: 0.25,
            height: 12.0,
            rotate_time: 0.3,
            air_t: 0.0,
            rot_t: 0.0,
        }
    }
}

impl DeathAnimation {
    pub fn update(mut animation_query: Query<&mut DeathAnimation>, time: Res<Time>) {
        let dt = time.delta_seconds();

        for mut anim in &mut animation_query {
            anim.air_t = (anim.air_t + dt / anim.air_time).min(1.0);
            anim.rot_t = (anim.rot_t + dt / anim.rotate_time).min(1.0);
        }
    }

    pub fn apply(mut animation_query: Query<(&DeathAnimation, &mut Transform)>) {
        for (anim, mut transform) in &mut animation_query {
            transform.translation.y += anim.height * (anim.air_t * PI).sin();
            transform.rotation *= Quat::from_rotation_z((anim.rot_t * TAU / 4.0).sin() * TAU / 4.0);
        }
    }
}

#[derive(Component, Reflect, Debug, Default)]
pub enum Facing {
    Left,
    #[default]
    Right,
}

impl Facing {
    pub fn apply(
        parent_query: Query<(Entity, &Parent)>,
        virtual_parent_query: Query<(Entity, &VirtualParent)>,
        facing_query: Query<&Facing>,
        mut transform_query: Query<&mut Transform>,
        mut sprite_query: Query<&mut Sprite>,
    ) {
        for (child, parent) in &parent_query {
            let Ok(facing) = facing_query.get(parent.get()) else {
                continue
            };

            if let Ok(mut sprite) = sprite_query.get_mut(child) {
                sprite.flip_x = facing.left();
            };

            if let Ok(mut transform) = transform_query.get_mut(child) {
                if facing.left() {
                    transform.translation.x = -transform.translation.x;
                    transform.rotation = -transform.rotation;
                }
            }
        }

        for (child, virtual_parent) in &virtual_parent_query {
            let Ok(facing) = facing_query.get(virtual_parent.0) else {
                continue
            };
            if facing.right() {
                continue;
            }
            let parent_x = {
                let Ok(parent_transform) = transform_query.get(virtual_parent.0) else {
                    continue
                };
                parent_transform.translation.x
            };
            let Ok(mut child_transform) = transform_query.get_mut(child) else {
                continue
            };

            // Reflect child's X about parent's X
            child_transform.translation.x = 2.0 * parent_x - child_transform.translation.x;
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

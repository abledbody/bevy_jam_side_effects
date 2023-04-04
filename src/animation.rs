use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

use crate::{
    mob::{DeadBody, MobInputs},
    util::{DespawnSet, VirtualParent},
};

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
/*
impl Offset {
    pub fn apply_to_sprites(
        mut parent_query: Query<
            (&Offset, &Parent, &mut Transform),
            (With<Sprite>, Without<VirtualParent>),
        >,
        mut virtual_parent_query: Query<
            (&Offset, &VirtualParent, &mut Transform),
            (With<Sprite>, Without<Parent>),
        >,
        facing_query: Query<(&Facing, &Transform), (Without<Parent>, Without<VirtualParent>)>,
    ) {
        for (offset, parent, mut transform) in &mut parent_query {
            let Ok((facing, _)) = facing_query.get(parent.get()) else {
                continue
            };

            transform.translation.x = offset.0.x * facing.sign();
            transform.translation.y = offset.0.y;
        }
        for (offset, virtual_parent, mut transform) in &mut virtual_parent_query {
            let Ok((facing, parent_transform)) = facing_query.get(virtual_parent.0) else {
                continue
            };

            transform.translation.x = parent_transform.translation.x + offset.0.x * facing.sign();
            transform.translation.y = parent_transform.translation.y + offset.0.y;
        }
    }

    pub fn apply_to_non_sprites(
        mut parent_query: Query<
            (&Offset, &Parent, &mut Transform),
            (Without<Sprite>, Without<VirtualParent>),
        >,
        mut virtual_parent_query: Query<
            (&Offset, &VirtualParent, &mut Transform),
            (Without<Sprite>, Without<Parent>),
        >,
        facing_query: Query<(&Facing, &Transform), (Without<Parent>, Without<VirtualParent>)>,
    ) {
        for (offset, parent, mut transform) in &mut parent_query {
            let Ok((facing, _)) = facing_query.get(parent.get()) else {
                continue
            };

            transform.translation.x = offset.0.x * facing.sign();
            transform.translation.y = offset.0.y;
        }
        for (offset, virtual_parent, mut transform) in &mut virtual_parent_query {
            let Ok((facing, parent_transform)) = facing_query.get(virtual_parent.0) else {
                continue
            };

            transform.translation.x = parent_transform.translation.x + offset.0.x * facing.sign();
            transform.translation.y = parent_transform.translation.y + offset.0.y;
        }
    }
}
*/

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

#[derive(Component, Reflect)]
pub struct DeathAnimation {
    pub air_time: f32,
    pub height: f32,
    pub rotate_time: f32,
    pub air_t: f32,
    pub rot_t: f32,
}

impl DeathAnimation {
    pub fn update(mut animator_query: Query<&mut DeathAnimation>, time: Res<Time>) {
        for mut death_animation in &mut animator_query {
            death_animation.air_t =
                (death_animation.air_t + time.delta_seconds() / death_animation.air_time).min(1.0);
            death_animation.rot_t = (death_animation.rot_t
                + time.delta_seconds() / death_animation.rotate_time)
                .min(1.0);
        }
    }

    pub fn template() -> DeathAnimation {
        DeathAnimation {
            air_time: 0.25,
            height: 12.0,
            rotate_time: 0.3,
            air_t: 0.0,
            rot_t: 0.0,
        }
    }
}

pub fn sum_animations(
    mut offset_query: Query<(
        &Offset,
        &mut Transform,
        Option<&Parent>,
        Option<&VirtualParent>,
        Option<&Facing>,
        Option<&WalkAnimation>,
        Option<&DeathAnimation>,
    )>,
    facing_query: Query<&Facing>,
    transform_query: Query<&Transform, Without<Offset>>,
) {
    for (
		offset,
		mut transform,
		parent,
		virtual_parent,
		facing,
		walk_animation,
		death_animation
	) in &mut offset_query
    {
        // If we have a facing, use it.
        // Otherwise use the parent's.
        // Otherwise use the virtual parent's.
        let facing_sign = facing
            .or_else(|| parent.and_then(|p| facing_query.get(p.get()).ok()))
            .or_else(|| virtual_parent.and_then(|p| facing_query.get(p.0).ok()))
            .map_or(1.0, |f| f.sign());

        let mut out_offset = offset.0.clone();
        let mut rot = 0.0;

        if let Some(walk_animation) = walk_animation {
            // PI is used here because we only want half a rotation.
            out_offset.y += walk_animation.height * (walk_animation.t * PI).sin();
        }
        if let Some(death_animation) = death_animation {
            out_offset.y += death_animation.height * (death_animation.air_t * PI).sin();
            rot += (death_animation.rot_t * TAU / 4.0 * facing_sign).sin() * TAU / 4.0;
        }

		out_offset.x *= facing_sign;
		rot *= facing_sign;

		if let Some(vp) = virtual_parent {
            if let Ok(parent_transform) = transform_query.get(vp.0) {
                out_offset.x += parent_transform.translation.x;
                out_offset.y += parent_transform.translation.y;
            }
        }

        transform.translation.x = out_offset.x;
        transform.translation.y = out_offset.y;
        transform.rotation = Quat::from_rotation_z(rot);
    }
}

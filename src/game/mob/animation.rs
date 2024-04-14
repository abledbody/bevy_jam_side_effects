use std::f32::consts::PI;
use std::f32::consts::TAU;

use bevy::math::vec2;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

use crate::common::PostTransformSet;
use crate::common::UpdateSet;
use crate::game::mob::player::PlayerControl;
use crate::game::mob::MobInputs;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<WalkAnimation>()
            .add_systems(
                Update,
                (
                    WalkAnimation::update.in_set(UpdateSet::Start),
                    WalkAnimation::trigger.in_set(UpdateSet::ApplyIntents),
                    WalkAnimation::play_step_sound,
                ),
            )
            .add_systems(
                PostUpdate,
                WalkAnimation::apply.in_set(PostTransformSet::Blend),
            );

        app.register_type::<AttackAnimation>()
            .add_systems(
                Update,
                (
                    AttackAnimation::update.in_set(UpdateSet::Start),
                    AttackAnimation::trigger.in_set(UpdateSet::ApplyIntents),
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                AttackAnimation::apply.in_set(PostTransformSet::Blend),
            );

        app.register_type::<FlinchAnimation>()
            .add_systems(Update, FlinchAnimation::update.in_set(UpdateSet::Start))
            .add_systems(
                PostUpdate,
                FlinchAnimation::apply.in_set(PostTransformSet::Blend),
            );

        app.register_type::<DeathAnimation>()
            .add_systems(Update, DeathAnimation::update.in_set(UpdateSet::Start))
            .add_systems(
                PostUpdate,
                DeathAnimation::apply.in_set(PostTransformSet::Blend),
            );
    }
}

#[derive(Component, Reflect)]
pub struct WalkAnimation {
    pub air_time: f32,
    pub height: f32,
    pub t: f32,
    pub start_frame: bool,
    pub sound: Option<Handle<AudioSource>>,
}

impl Default for WalkAnimation {
    fn default() -> Self {
        Self {
            air_time: 0.18,
            height: 3.0,
            t: 1.0,
            start_frame: false,
            sound: None,
        }
    }
}

impl WalkAnimation {
    pub fn trigger(
        mut animation_query: Query<(&mut WalkAnimation, &Parent)>,
        inputs_query: Query<&MobInputs>,
    ) {
        for (mut anim, parent) in &mut animation_query {
            if anim.t < 1.0 {
                anim.start_frame = false;
                continue;
            }

            let Ok(inputs) = inputs_query.get(parent.get()) else {
                continue;
            };
            if inputs.movement.length() == 0.0 {
                anim.t = 1.0;
                continue;
            }

            anim.start_frame = true;
            anim.t = anim.t.fract();
        }
    }

    pub fn play_step_sound(
        player_query: Query<&GlobalTransform, With<PlayerControl>>,
        animation_query: Query<(&WalkAnimation, &GlobalTransform), Without<PlayerControl>>,
        audio: Res<Audio>,
    ) {
        let Ok(player) = player_query.get_single() else {
            return;
        };
        let player_pos = player.translation().xy();

        for (anim, transform) in &animation_query {
            if !anim.start_frame {
                continue;
            }
            let Some(sound) = &anim.sound else { continue };

            let pos = transform.translation().xy();
            let dist_to_player = (player_pos - pos).length() as f64;
            let max_volume = 0.3;

            audio
                .play(sound.clone())
                .with_volume(max_volume / (0.2 * dist_to_player).max(1.0));
        }
    }

    pub fn update(mut animation_query: Query<&mut WalkAnimation>, time: Res<Time>) {
        let dt = time.delta_seconds();

        for mut anim in &mut animation_query {
            anim.t += dt / anim.air_time;
        }
    }

    pub fn apply(mut animation_query: Query<(&WalkAnimation, &mut Transform)>) {
        for (anim, mut transform) in &mut animation_query {
            // PI is used here because we only want half a rotation.
            transform.translation.y += anim.height * (anim.t.min(1.0) * PI).sin();
        }
    }
}

#[derive(Component, Reflect)]
pub struct AttackAnimation {
    pub duration: f32,
    pub distance: f32,
    pub direction: Vec2,
    pub x_sign: f32,
    pub t: f32,
}

impl AttackAnimation {
    pub fn trigger(
        mut animation_query: Query<(&mut AttackAnimation, &Parent)>,
        inputs_query: Query<&MobInputs>,
    ) {
        for (mut anim, parent) in &mut animation_query {
            let Ok(inputs) = inputs_query.get(parent.get()) else {
                continue;
            };
            let Some(attack) = inputs.attack else {
                continue;
            };

            anim.t = 0.0;
            anim.direction = vec2(attack.x.abs(), attack.y);
            anim.x_sign = attack.x.signum();
        }
    }

    pub fn update(mut animation_query: Query<&mut AttackAnimation>, time: Res<Time>) {
        let dt = time.delta_seconds();

        for mut anim in &mut animation_query {
            anim.t = (anim.t + dt / anim.duration).min(1.0);
        }
    }

    pub fn apply(mut animation_query: Query<(&AttackAnimation, &mut Transform)>) {
        for (anim, mut transform) in &mut animation_query {
            transform.translation += (anim.direction * anim.distance * (1.0 - anim.t)).extend(0.0);
        }
    }
}

impl Default for AttackAnimation {
    fn default() -> Self {
        Self {
            duration: 0.2,
            distance: 10.0,
            direction: Vec2::ZERO,
            x_sign: 0.0,
            t: 1.0,
        }
    }
}

#[derive(Component, Reflect)]
pub struct FlinchAnimation {
    pub duration: f32,
    pub distance: f32,
    pub rotation: f32,
    pub direction: Vec2,
    pub t: f32,
}

impl Default for FlinchAnimation {
    fn default() -> Self {
        Self {
            duration: 0.15,
            distance: 6.0,
            rotation: TAU / 16.0,
            direction: Vec2::ZERO,
            t: 1.0,
        }
    }
}

impl FlinchAnimation {
    pub fn trigger(&mut self, direction: Vec2) {
        self.t = 0.0;
        self.direction = direction;
    }

    pub fn update(mut animation_query: Query<&mut FlinchAnimation>, time: Res<Time>) {
        let dt = time.delta_seconds();

        for mut anim in &mut animation_query {
            anim.t = (anim.t + dt / anim.duration).min(1.0);
        }
    }

    pub fn apply(mut animation_query: Query<(&FlinchAnimation, &mut Transform)>) {
        for (anim, mut transform) in &mut animation_query {
            let agnostic_direction = vec2(-anim.direction.x.abs(), anim.direction.y);
            transform.translation +=
                (agnostic_direction * anim.distance * (1.0 - anim.t)).extend(0.0);
            transform.rotation *= Quat::from_rotation_z(
                -(anim.direction.x.signum()) * anim.rotation * (1.0 - anim.t),
            );
        }
    }
}

#[derive(Component, Reflect)]
pub struct DeathAnimation {
    pub height: f32,
    pub final_height: f32,
    pub air_time: f32,
    pub rotate_time: f32,
    pub air_t: f32,
    pub rot_t: f32,
}

impl Default for DeathAnimation {
    fn default() -> Self {
        Self {
            air_time: 0.25,
            height: 16.0,
            final_height: -8.0,
            rotate_time: 0.3,
            air_t: 1.0,
            rot_t: 1.0,
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
            transform.translation.y +=
                anim.height * (anim.air_t * PI).sin() + anim.final_height * anim.air_t;
            transform.rotation *= Quat::from_rotation_z((anim.rot_t * TAU / 4.0).sin() * TAU / 4.0);
        }
    }
}

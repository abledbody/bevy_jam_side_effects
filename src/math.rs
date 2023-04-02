use bevy::prelude::*;

pub trait MoveTowards {
    fn move_towards(self, target: Self, step: f32) -> Self;
}

impl MoveTowards for Vec2 {
    fn move_towards(self, target: Vec2, step: f32) -> Self {
        let distance = (target - self).length();
        let direction = (target - self).normalize_or_zero();
        if distance < step {
            target
        } else {
            self + direction * step
        }
    }
}

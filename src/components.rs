use std::ops::RangeInclusive;

use bevy::{
    core::Timer,
    math::{Vec2, Vec3},
    prelude::Component,
};

#[derive(Component)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Fire;

#[derive(Component)]
pub struct SpriteSize(pub Vec2);

impl From<(f32, f32)> for SpriteSize {
    fn from(size: (f32, f32)) -> Self {
        Self(Vec2::new(size.0, size.1))
    }
}

#[derive(Component)]
pub struct Movable {
    pub auto_despawn: bool,
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct FromPlayer;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct FromEnemy;

#[derive(Component)]
pub struct Explosion;

#[derive(Component)]
pub struct ExplosionToSpawn(pub Vec3);

#[derive(Component)]
pub struct ExplosionTimer(pub Timer);

impl Default for ExplosionTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, true))
    }
}

#[derive(Component)]
pub struct Animate {
    pub range: RangeInclusive<usize>,
    pub timer: Timer,
}

impl Default for Animate {
    fn default() -> Self {
        Self {
            range: 0..=0,
            timer: Timer::from_seconds(0.5, true),
        }
    }
}

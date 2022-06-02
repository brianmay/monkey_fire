use std::f32::consts::PI;

use crate::{
    components::{Animate, Enemy, Fire, FromEnemy, Movable, OnOutsideWindow, SpriteSize, Velocity},
    EnemyCount, GameTextures, WinSize, ENEMY_MAX, ENEMY_SIZE, FIRE_SIZE, SPRITE_SCALE, TIME_STEP,
};
use bevy::{core::FixedTimestep, ecs::schedule::ShouldRun, prelude::*};
use rand::{thread_rng, Rng};

use self::formation::{Formation, FormationMaker};

mod formation;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FormationMaker::default())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(1.0))
                    .with_system(enemy_spawn_system),
            )
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(enemy_fire_criteria)
                    .with_system(enemy_fire_system),
            )
            .add_system(enemy_movement_system);
    }
}

fn enemy_spawn_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    mut enemy_count: ResMut<EnemyCount>,
    mut formation_maker: ResMut<FormationMaker>,
    win_size: Res<WinSize>,
) {
    if enemy_count.0 < ENEMY_MAX {
        let formation = formation_maker.make(&win_size);
        let (x, y) = formation.start;

        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: game_textures.enemy.clone(),
                transform: Transform {
                    translation: Vec3::new(x, y, 10.0),
                    scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Enemy)
            .insert(formation)
            .insert(SpriteSize::from(ENEMY_SIZE))
            .insert(Animate {
                range: 0..=7,
                ..Default::default()
            });

        enemy_count.0 += 1;
    }
}

fn enemy_fire_criteria() -> ShouldRun {
    if thread_rng().gen_bool(1.0 / 60.0) {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn enemy_fire_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    enemy_query: Query<&Transform, With<Enemy>>,
) {
    for &tf in enemy_query.iter() {
        let (x, y) = (tf.translation.x, tf.translation.y);

        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: game_textures.fire.clone(),
                transform: Transform {
                    translation: Vec3::new(x, y, 10.0),
                    scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Fire)
            .insert(FromEnemy)
            .insert(SpriteSize::from(FIRE_SIZE))
            .insert(Movable {
                on_outside_window: OnOutsideWindow::Despawn,
            })
            .insert(Velocity { x: 0.0, y: -1.0 })
            .insert(Animate {
                range: 0..=2,
                ..Default::default()
            });
    }
}

fn enemy_movement_system(mut query: Query<(&mut Transform, &mut Formation), With<Enemy>>) {
    for (mut transform, mut formation) in query.iter_mut() {
        let (x_org, y_org) = (transform.translation.x, transform.translation.y);
        let max_distance = TIME_STEP * formation.speed;

        // 1 for counter clockwise, -1 for clockwise
        let dir: f32 = if formation.start.0 < 0.0 { 1.0 } else { -1.0 };
        let (x_pivot, y_pivot) = formation.pivot;
        let (x_radius, y_radius) = formation.radius;
        let angle = formation.angle
            + dir * formation.speed * TIME_STEP / (x_radius.min(y_radius) * PI / 2.0);

        let x_dst = x_radius * angle.cos() + x_pivot;
        let y_dst = y_radius * angle.sin() + y_pivot;

        let dx = x_org - x_dst;
        let dy = y_org - y_dst;
        let distance = (dx * dx + dy * dy).sqrt();
        let distance_ratio = if distance != 0.0 {
            max_distance / distance
        } else {
            0.0
        };

        let x = x_org - dx * distance_ratio;
        let x = if dx > 0.0 { x.max(x_dst) } else { x.min(x_dst) };
        let y = y_org - dy * distance_ratio;
        let y = if dy > 0.0 { y.max(y_dst) } else { y.min(y_dst) };

        if distance < max_distance * formation.speed / 20.0 {
            formation.angle = angle;
        }

        transform.scale.x = match transform.translation.x > x {
            true => -1.0 * SPRITE_SCALE,
            false => 1.0 * SPRITE_SCALE,
        };

        let translation = &mut transform.translation;
        (translation.x, translation.y) = (x, y);
    }
}

use std::collections::HashSet;

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::Vec3Swizzles,
    prelude::*,
    sprite::collide_aabb::collide,
};
use bevy_inspector_egui::WorldInspectorPlugin;
use components::{
    Animate, Enemy, Explosion, ExplosionTimer, ExplosionToSpawn, Fire, FromEnemy, FromPlayer,
    Movable, Player, SpriteSize, Velocity,
};

use crate::components::OnOutsideWindow;

mod components;
mod enemy;
mod player;

const PLAYER_SHEET: &str = "monkey.png";
const PLAYER_SIZE: (f32, f32) = (140.0, 168.0);

const PLAYER_FIRE_SHEET: &str = "sun.png";
const PLAYER_FIRE_SIZE: (f32, f32) = (70.0, 70.0);

const ENEMY_SHEET: &str = "ninja_cat.png";
const ENEMY_SIZE: (f32, f32) = (256.0, 222.0);

const ENEMY_FIRE_SHEET: &str = "penguin.png";
const ENEMY_FIRE_SIZE: (f32, f32) = (72.0, 64.0);

const EXPLOSION_SHEET: &str = "nuclear_explosion.png";
const EXPLOSION_LEN: usize = 10;

const SPRITE_SCALE: f32 = 0.5;

const TIME_STEP: f32 = 1.0 / 60.0;
const BASE_SPEED: f32 = 500.0;
const PLAYER_RESPAWN_DELAY: f64 = 2.0;
const ENEMY_MAX: u32 = 2;
const FORMATION_MEMBERS_MAX: u32 = 2;

pub struct WinSize {
    pub width: f32,
    pub height: f32,
}

struct GameTextures {
    player: Handle<TextureAtlas>,
    player_fire: Handle<TextureAtlas>,
    enemy: Handle<TextureAtlas>,
    enemy_fire: Handle<TextureAtlas>,
    explosion: Handle<TextureAtlas>,
}

struct EnemyCount(u32);

#[derive(Debug)]
struct PlayerState {
    on: bool,
    last_shot: f64, // -1 if not shot
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            on: false,
            last_shot: -1.0,
        }
    }
}

impl PlayerState {
    pub fn shot(&mut self, time: f64) {
        self.on = false;
        self.last_shot = time;
    }

    pub fn spawned(&mut self) {
        self.on = true;
        self.last_shot = -1.0;
    }
}

#[derive(Default)]
struct Scoreboard {
    pub score: u32,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.3, 0.3, 0.3)))
        .insert_resource(WindowDescriptor {
            title: "Monkey Fire".to_string(),
            width: 1280.0,
            height: 720.0,
            ..Default::default()
        })
        .insert_resource(Scoreboard::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(player::PlayerPlugin)
        .add_plugin(enemy::EnemyPlugin)
        .add_startup_system(setup_system)
        .add_system(movable_system)
        .add_system(player_fire_hit_enemy_system)
        .add_system(enemy_fire_hit_player_system)
        .add_system(explosion_to_spawn_system)
        .add_system(explosion_animation_system)
        .add_system(animate_system)
        .add_system(scoreboard_system)
        .run();
}

fn setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    windows: Res<Windows>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let window = windows.get_primary().unwrap();
    let (win_w, win_h) = (window.width(), window.height());
    let win_size = WinSize {
        width: win_w,
        height: win_h,
    };
    commands.insert_resource(win_size);

    let texture_handle = asset_server.load(PLAYER_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(140.0, 168.0), 7, 1);
    let player = texture_atlases.add(texture_atlas);

    let texture_handle = asset_server.load(PLAYER_FIRE_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(70.0, 70.0), 3, 1);
    let player_fire = texture_atlases.add(texture_atlas);

    let texture_handle = asset_server.load(ENEMY_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(256.0, 222.0), 8, 1);
    let enemy = texture_atlases.add(texture_atlas);

    let texture_handle = asset_server.load(ENEMY_FIRE_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(72.0, 64.0), 2, 1);
    let enemy_fire = texture_atlases.add(texture_atlas);

    let texture_handle = asset_server.load(EXPLOSION_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(256.0, 256.0), 10, 1);
    let explosion = texture_atlases.add(texture_atlas);

    let game_textures = GameTextures {
        player,
        player_fire,
        enemy,
        enemy_fire,
        explosion,
    };
    commands.insert_resource(game_textures);
    commands.insert_resource(EnemyCount(0));

    commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Score: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.5, 0.5, 1.0),
                    },
                },
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(1.0, 0.5, 0.5),
                    },
                },
            ],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });
}

fn movable_system(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &Velocity, &mut Transform, &Movable)>,
) {
    for (entity, velocity, mut transform, movable) in query.iter_mut() {
        transform.translation.x += velocity.x * TIME_STEP * BASE_SPEED;
        transform.translation.y += velocity.y * TIME_STEP * BASE_SPEED;

        const MARGIN: f32 = 50.0;
        let left = -win_size.width / 2.0 - MARGIN;
        let right = win_size.width / 2.0 + MARGIN;
        let top = win_size.height / 2.0 + MARGIN;
        let bottom = -win_size.height / 2.0 - MARGIN;

        let left_of_screen = transform.translation.x < left;
        let right_of_screen = transform.translation.x > right;
        let top_of_screen = transform.translation.y < -win_size.height / 2.0 - MARGIN;
        let bottom_of_screen = transform.translation.y > win_size.height / 2.0 + MARGIN;

        match movable.on_outside_window {
            OnOutsideWindow::Despawn => {
                if left_of_screen | right_of_screen | top_of_screen | bottom_of_screen {
                    commands.entity(entity).despawn();
                }
            }
            OnOutsideWindow::Wrap => {
                if left_of_screen {
                    transform.translation.x = right;
                } else if right_of_screen {
                    transform.translation.x = left;
                }
                if top_of_screen {
                    transform.translation.y = bottom;
                } else if bottom_of_screen {
                    transform.translation.y = top;
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn player_fire_hit_enemy_system(
    mut commands: Commands,
    mut enemy_count: ResMut<EnemyCount>,
    mut scoreboard: ResMut<Scoreboard>,
    fire_query: Query<(Entity, &Transform, &SpriteSize), (With<Fire>, With<FromPlayer>)>,
    enemy_query: Query<(Entity, &Transform, &SpriteSize), With<Enemy>>,
) {
    let mut despawned_entities: HashSet<Entity> = HashSet::new();

    for (fire_entity, fire_tf, fire_size) in fire_query.iter() {
        if despawned_entities.contains(&fire_entity) {
            continue;
        }

        let fire_scale = fire_tf.scale.xy().abs();

        for (enemy_entity, enemy_tf, enemy_size) in enemy_query.iter() {
            if despawned_entities.contains(&enemy_entity)
                || despawned_entities.contains(&fire_entity)
            {
                continue;
            }

            let enemy_scale = enemy_tf.scale.xy().abs();

            let collision = collide(
                fire_tf.translation,
                fire_size.0 * fire_scale,
                enemy_tf.translation,
                enemy_size.0 * enemy_scale,
            );

            if collision.is_some() {
                commands.entity(enemy_entity).despawn();
                despawned_entities.insert(enemy_entity);
                enemy_count.0 -= 1;

                commands.entity(fire_entity).despawn();
                despawned_entities.insert(fire_entity);

                scoreboard.score = scoreboard.score.saturating_add(1);

                commands
                    .spawn()
                    .insert(ExplosionToSpawn(enemy_tf.translation));
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn enemy_fire_hit_player_system(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    mut scoreboard: ResMut<Scoreboard>,
    time: Res<Time>,
    fire_query: Query<(Entity, &Transform, &SpriteSize), (With<Fire>, With<FromEnemy>)>,
    player_query: Query<(Entity, &Transform, &SpriteSize), With<Player>>,
) {
    if let Ok((player_entity, player_tf, player_size)) = player_query.get_single() {
        let player_scale = player_tf.scale.xy().abs();

        for (fire_entity, fire_tf, fire_size) in fire_query.iter() {
            let fire_scale = fire_tf.scale.xy().abs();

            let collision = collide(
                fire_tf.translation,
                fire_size.0 * fire_scale,
                player_tf.translation,
                player_size.0 * player_scale,
            );

            if collision.is_some() {
                commands.entity(player_entity).despawn();
                player_state.shot(time.seconds_since_startup());

                commands.entity(fire_entity).despawn();

                scoreboard.score = scoreboard.score.saturating_sub(1);

                commands
                    .spawn()
                    .insert(ExplosionToSpawn(player_tf.translation));

                break;
            }
        }
    }
}

fn explosion_to_spawn_system(
    mut commands: Commands,
    game_textures: Res<GameTextures>,
    query: Query<(Entity, &ExplosionToSpawn)>,
) {
    for (explosion_spawn_entity, explosion_to_spawn) in query.iter() {
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: game_textures.explosion.clone(),
                transform: Transform {
                    translation: explosion_to_spawn.0,
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Explosion)
            .insert(ExplosionTimer::default());

        commands.entity(explosion_spawn_entity).despawn();
    }
}

fn explosion_animation_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ExplosionTimer, &mut TextureAtlasSprite), With<Explosion>>,
) {
    for (entity, mut timer, mut sprite) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            sprite.index += 1;
            if sprite.index >= EXPLOSION_LEN {
                commands.entity(entity).despawn()
            }
        }
    }
}

fn animate_system(time: Res<Time>, mut query: Query<(&mut Animate, &mut TextureAtlasSprite)>) {
    for (mut animate, mut sprite) in query.iter_mut() {
        animate.timer.tick(time.delta());
        if animate.timer.finished() {
            let range = &animate.range;
            sprite.index = sprite.index.saturating_add(1);
            if sprite.index < *range.start() {
                sprite.index = *range.start();
            }
            if sprite.index > *range.end() {
                sprite.index = *range.start();
            }
        }
    }
}

fn scoreboard_system(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    for mut text in query.iter_mut() {
        text.sections[1].value = scoreboard.score.to_string();
    }
}

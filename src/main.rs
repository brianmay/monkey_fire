use std::collections::HashSet;

use bevy::{math::Vec3Swizzles, prelude::*, sprite::collide_aabb::collide};
use components::{
    Animate, Enemy, Explosion, ExplosionTimer, ExplosionToSpawn, Fire, FromEnemy, FromPlayer,
    Movable, Player, SpriteSize, Velocity,
};

mod components;
mod enemy;
mod player;

const PLAYER_SHEET: &str = "monkey.png";
const PLAYER_SIZE: (f32, f32) = (140.0, 168.0);

const ENEMY_SHEET: &str = "ninja_cat.png";
const ENEMY_SIZE: (f32, f32) = (256.0, 222.0);

const FIRE_SHEET: &str = "sun.png";
const FIRE_SIZE: (f32, f32) = (70.0, 70.0);

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
    fire: Handle<TextureAtlas>,
    enemy: Handle<TextureAtlas>,
    explosion: Handle<TextureAtlas>,
}

struct EnemyCount(u32);

pub enum PlayerAnimation {
    Idle,
    Walking,
}

#[derive(Copy, Clone)]
pub enum FaceDirection {
    Left,
    Right,
}

struct PlayerState {
    on: bool,
    last_shot: f64, // -1 if not shot
    pub state: PlayerAnimation,
    pub timer: Timer,
    pub direction: FaceDirection,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            on: false,
            last_shot: -1.0,
            state: PlayerAnimation::Idle,
            timer: Timer::from_seconds(0.2, true),
            direction: FaceDirection::Left,
        }
    }
}

impl PlayerState {
    pub fn shot(&mut self, time: f64) {
        self.on = false;
        self.last_shot = time;
        self.state = PlayerAnimation::Idle;
    }

    pub fn spawned(&mut self) {
        self.on = true;
        self.last_shot = -1.0;
        self.state = PlayerAnimation::Idle;
    }
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
        .add_plugins(DefaultPlugins)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(enemy::EnemyPlugin)
        .add_startup_system(setup_system)
        .add_system(movable_system)
        .add_system(player_fire_hit_enemy_system)
        .add_system(enemy_fire_hit_player_system)
        .add_system(explosion_to_spawn_system)
        .add_system(explosion_animation_system)
        .add_system(animate)
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
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(PLAYER_SIZE.0, PLAYER_SIZE.1),
        5,
        1,
    );
    let player = texture_atlases.add(texture_atlas);

    let texture_handle = asset_server.load(ENEMY_SHEET);
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(ENEMY_SIZE.0, ENEMY_SIZE.1), 8, 1);
    let enemy = texture_atlases.add(texture_atlas);

    let texture_handle = asset_server.load(FIRE_SHEET);
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(FIRE_SIZE.0, FIRE_SIZE.1), 3, 1);
    let fire = texture_atlases.add(texture_atlas);

    let texture_handle = asset_server.load(EXPLOSION_SHEET);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(256.0, 256.0), 10, 1);
    let explosion = texture_atlases.add(texture_atlas);

    let game_textures = GameTextures {
        player,
        fire,
        enemy,
        explosion,
    };
    commands.insert_resource(game_textures);
    commands.insert_resource(EnemyCount(0));
}

fn movable_system(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &Velocity, &mut Transform, &Movable)>,
) {
    for (entity, velocity, mut transform, movable) in query.iter_mut() {
        transform.translation.x += velocity.x * TIME_STEP * BASE_SPEED;
        transform.translation.y += velocity.y * TIME_STEP * BASE_SPEED;

        if movable.auto_despawn {
            const MARGIN: f32 = 200.0;
            if transform.translation.x < -win_size.width / 2.0 - MARGIN
                || transform.translation.x > win_size.width / 2.0 + MARGIN
                || transform.translation.y < -win_size.height / 2.0 - MARGIN
                || transform.translation.y > win_size.height / 2.0 + MARGIN
            {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn player_fire_hit_enemy_system(
    mut commands: Commands,
    mut enemy_count: ResMut<EnemyCount>,
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

fn animate(time: Res<Time>, mut query: Query<(&mut Animate, &mut TextureAtlasSprite)>) {
    for (mut animate, mut sprite) in query.iter_mut() {
        animate.timer.tick(time.delta());
        if animate.timer.finished() {
            sprite.index = (sprite.index + 1) % animate.length;
        }
    }
}

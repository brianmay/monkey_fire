use bevy::{core::FixedTimestep, prelude::*};

use crate::{
    components::{
        Animate, Fire, FromPlayer, Movable, OnOutsideWindow, Player, SpriteSize, Velocity,
    },
    GameTextures, PlayerState, WinSize, FIRE_SIZE, PLAYER_RESPAWN_DELAY, PLAYER_SIZE, SPRITE_SCALE,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerState::default())
            .insert_resource(PlayerSprite::default())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(0.5))
                    .with_system(player_spawn_system),
            )
            .add_system(player_keyboard_event_system)
            .add_system(player_fire_system)
            .add_system(
                player_animate
                    .after(player_spawn_system)
                    .after(player_keyboard_event_system),
            );
    }
}

#[derive(Debug)]
pub enum PlayerAnimation {
    Idle,
    Walking,
}

#[derive(Copy, Clone, Debug)]
pub enum PlayerDirection {
    Left,
    Right,
}

#[derive(Debug)]
struct PlayerSprite {
    pub state: PlayerAnimation,
    pub direction: PlayerDirection,
}

impl Default for PlayerSprite {
    fn default() -> Self {
        Self {
            state: PlayerAnimation::Idle,
            direction: PlayerDirection::Left,
        }
    }
}

fn player_spawn_system(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    time: Res<Time>,
    game_textures: Res<GameTextures>,
    win_size: Res<WinSize>,
) {
    let now = time.seconds_since_startup();
    let last_shot = player_state.last_shot;

    if !player_state.on && (last_shot == -1.0 || now > last_shot + PLAYER_RESPAWN_DELAY) {
        let bottom = -win_size.height / 2.0;
        commands
            .spawn_bundle(SpriteSheetBundle {
                texture_atlas: game_textures.player.clone(),
                sprite: TextureAtlasSprite {
                    index: 6,
                    ..Default::default()
                },
                transform: Transform {
                    translation: Vec3::new(0.0, bottom + PLAYER_SIZE.1 / 2.0 * SPRITE_SCALE, 10.0),
                    scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Player)
            .insert(SpriteSize::from(PLAYER_SIZE))
            .insert(Movable {
                on_outside_window: OnOutsideWindow::Wrap,
            })
            .insert(Velocity { x: 0.0, y: 0.0 })
            .insert(Animate {
                range: 6..=6,
                ..Default::default()
            });

        player_state.spawned();
    }
}

fn player_keyboard_event_system(
    kb: Res<Input<KeyCode>>,
    mut sprite: ResMut<PlayerSprite>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Player>>,
) {
    if let Ok((mut velocity, mut transform)) = query.get_single_mut() {
        let (direction, animation, velocity_x) = if kb.pressed(KeyCode::Left) {
            (PlayerDirection::Left, PlayerAnimation::Walking, -1.0)
        } else if kb.pressed(KeyCode::Right) {
            (PlayerDirection::Right, PlayerAnimation::Walking, 1.0)
        } else {
            (sprite.direction, PlayerAnimation::Idle, 0.0)
        };

        sprite.direction = direction;
        sprite.state = animation;
        velocity.x = velocity_x;

        transform.scale.x = match sprite.direction {
            PlayerDirection::Left => -1.0 * SPRITE_SCALE,
            PlayerDirection::Right => 1.0 * SPRITE_SCALE,
        }
    }
}

fn player_fire_system(
    mut commands: Commands,
    kb: Res<Input<KeyCode>>,
    game_textures: Res<GameTextures>,
    query: Query<&Transform, With<Player>>,
) {
    if let Ok(player_tf) = query.get_single() {
        if kb.just_pressed(KeyCode::Space) {
            let (x, y) = (player_tf.translation.x, player_tf.translation.y);

            commands
                .spawn_bundle(SpriteSheetBundle {
                    texture_atlas: game_textures.fire.clone(),
                    transform: Transform {
                        translation: Vec3::new(x, y, 0.0),
                        scale: Vec3::new(1.0, 1.0, 1.0),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Fire)
                .insert(FromPlayer)
                .insert(SpriteSize::from(FIRE_SIZE))
                .insert(Velocity { x: 0.0, y: 1.0 })
                .insert(Movable {
                    on_outside_window: OnOutsideWindow::Despawn,
                })
                .insert(Animate {
                    range: 0..=2,
                    ..Default::default()
                });
        }
    }
}

fn player_animate(sprite: Res<PlayerSprite>, mut query: Query<&mut Animate, With<Player>>) {
    let range = match sprite.state {
        PlayerAnimation::Idle => 6..=6,
        PlayerAnimation::Walking => 0..=3,
    };
    if let Ok(mut animate) = query.get_single_mut() {
        animate.range = range;
    }
}

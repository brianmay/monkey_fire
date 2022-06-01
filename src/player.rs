use bevy::{core::FixedTimestep, prelude::*};

use crate::{
    components::{Animate, Fire, FromPlayer, Movable, Player, SpriteSize, Velocity},
    FaceDirection, GameTextures, PlayerAnimation, PlayerState, WinSize, FIRE_SIZE,
    PLAYER_RESPAWN_DELAY, PLAYER_SIZE, SPRITE_SCALE,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerState::default())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(0.5))
                    .with_system(player_spawn_system),
            )
            .add_system(player_keyboard_event_system)
            .add_system(player_fire_system)
            .add_system(player_animate);
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
                    index: 4,
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
                auto_despawn: false,
            })
            .insert(Velocity { x: 0.0, y: 0.0 });

        player_state.spawned();
    }
}

fn player_keyboard_event_system(
    kb: Res<Input<KeyCode>>,
    mut state: ResMut<PlayerState>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Player>>,
) {
    if let Ok((mut velocity, mut transform)) = query.get_single_mut() {
        (state.direction, state.state, velocity.x) = if kb.pressed(KeyCode::Left) {
            (FaceDirection::Left, PlayerAnimation::Walking, -1.0)
        } else if kb.pressed(KeyCode::Right) {
            (FaceDirection::Right, PlayerAnimation::Walking, 1.0)
        } else {
            (state.direction, PlayerAnimation::Idle, 0.0)
        };

        transform.scale.x = match state.direction {
            FaceDirection::Left => -1.0 * SPRITE_SCALE,
            FaceDirection::Right => 1.0 * SPRITE_SCALE,
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
                .insert(Movable { auto_despawn: true })
                .insert(Animate {
                    length: 3,
                    ..Default::default()
                });
        }
    }
}

fn player_animate(
    time: Res<Time>,
    mut state: ResMut<PlayerState>,
    mut query: Query<&mut TextureAtlasSprite, With<Player>>,
) {
    for mut sprite in query.iter_mut() {
        state.timer.tick(time.delta());
        if state.timer.finished() {
            sprite.index = match state.state {
                PlayerAnimation::Idle => 4,
                PlayerAnimation::Walking => (sprite.index + 1) % 4,
            }
        }
    }
}

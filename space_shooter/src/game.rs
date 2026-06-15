use bevy::{
    color::Color,
    ecs::{
        entity::Entity,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    input::{keyboard::KeyCode, ButtonInput},
    math::{Vec2, Vec3},
    prelude::default,
    sprite::Sprite,
    time::Time,
    transform::components::Transform,
};

use crate::components::{
    Asteroid, AsteroidSpeed, BULLET_SIZE, BULLET_SPEED, BASE_ASTEROID_SPEED, Bullet,
    GameState, PLAYER_SIZE, PLAYER_SPEED, Player, ShootCooldown,
    SCORE_PER_LEVEL, SpawnTimer, SPEED_INCREMENT, WINDOW_HEIGHT, WINDOW_WIDTH,
};

pub fn spawn_player(commands: &mut Commands) {
    commands.spawn((
        Sprite {
            color: Color::srgb(0.2, 0.6, 1.0),
            custom_size: Some(PLAYER_SIZE),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, -WINDOW_HEIGHT / 2.0 + 60.0, 0.0)),
        Player,
    ));
}

pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let Ok(mut transform) = query.get_single_mut() else {
        return;
    };
    let mut direction = 0.0;
    if keyboard.pressed(KeyCode::ArrowLeft) {
        direction -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        direction += 1.0;
    }
    let half_width = WINDOW_WIDTH / 2.0 - PLAYER_SIZE.x / 2.0;
    transform.translation.x = (transform.translation.x + direction * PLAYER_SPEED * time.delta_secs())
        .clamp(-half_width, half_width);
}

pub fn player_shoot(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    query: Query<&Transform, With<Player>>,
    mut cooldown: ResMut<ShootCooldown>,
    time: Res<Time>,
) {
    cooldown.timer.tick(time.delta());
    if !keyboard.pressed(KeyCode::Space) || !cooldown.timer.finished() {
        return;
    }
    let Ok(player_transform) = query.get_single() else {
        return;
    };
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 1.0, 0.3),
            custom_size: Some(BULLET_SIZE),
            ..default()
        },
        Transform::from_translation(Vec3::new(
            player_transform.translation.x,
            player_transform.translation.y + PLAYER_SIZE.y / 2.0 + BULLET_SIZE.y / 2.0,
            0.0,
        )),
        Bullet,
    ));
    cooldown.timer.reset();
}

pub fn move_bullets(mut query: Query<&mut Transform, With<Bullet>>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        transform.translation.y += BULLET_SPEED * time.delta_secs();
    }
}

pub fn despawn_offscreen_bullets(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Bullet>>,
) {
    for (entity, transform) in query.iter() {
        if transform.translation.y > WINDOW_HEIGHT / 2.0 + 20.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn spawn_asteroids(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<SpawnTimer>,
    game_state: Res<GameState>,
) {
    if game_state.game_over {
        return;
    }
    spawn_timer.timer.tick(time.delta());
    if !spawn_timer.timer.just_finished() {
        return;
    }
    let size = 20.0 + rand::random::<f32>() * 30.0;
    let half_width = WINDOW_WIDTH / 2.0 - size / 2.0;
    let x = (rand::random::<f32>() * 2.0 - 1.0) * half_width;
    let speed = BASE_ASTEROID_SPEED + (game_state.speed_level - 1) as f32 * SPEED_INCREMENT;
    let color_val = 0.4 + rand::random::<f32>() * 0.3;
    commands.spawn((
        Sprite {
            color: Color::srgb(color_val, color_val * 0.7, color_val * 0.5),
            custom_size: Some(Vec2::new(size, size)),
            ..default()
        },
        Transform::from_translation(Vec3::new(x, WINDOW_HEIGHT / 2.0 + size / 2.0, 0.0)),
        Asteroid,
        AsteroidSpeed(speed),
    ));
}

pub fn move_asteroids(
    mut query: Query<(&mut Transform, &AsteroidSpeed), With<Asteroid>>,
    time: Res<Time>,
    game_state: Res<GameState>,
) {
    if game_state.game_over {
        return;
    }
    for (mut transform, speed) in query.iter_mut() {
        transform.translation.y -= speed.0 * time.delta_secs();
    }
}

pub fn despawn_offscreen_asteroids(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Asteroid>>,
) {
    for (entity, transform) in query.iter() {
        if transform.translation.y < -WINDOW_HEIGHT / 2.0 - 50.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn bullet_asteroid_collision(
    mut commands: Commands,
    bullet_query: Query<(Entity, &Transform), With<Bullet>>,
    asteroid_query: Query<(Entity, &Transform, &Sprite), With<Asteroid>>,
    mut game_state: ResMut<GameState>,
) {
    let mut bullets_to_despawn: Vec<Entity> = Vec::new();
    let mut asteroids_to_despawn: Vec<Entity> = Vec::new();
    for (bullet_entity, bullet_transform) in bullet_query.iter() {
        for (asteroid_entity, asteroid_transform, asteroid_sprite) in asteroid_query.iter() {
            let asteroid_size = asteroid_sprite.custom_size.unwrap_or(Vec2::splat(30.0));
            let collision = bullet_transform.translation.x - BULLET_SIZE.x / 2.0
                < asteroid_transform.translation.x + asteroid_size.x / 2.0
                && bullet_transform.translation.x + BULLET_SIZE.x / 2.0
                    > asteroid_transform.translation.x - asteroid_size.x / 2.0
                && bullet_transform.translation.y - BULLET_SIZE.y / 2.0
                    < asteroid_transform.translation.y + asteroid_size.y / 2.0
                && bullet_transform.translation.y + BULLET_SIZE.y / 2.0
                    > asteroid_transform.translation.y - asteroid_size.y / 2.0;
            if collision {
                bullets_to_despawn.push(bullet_entity);
                asteroids_to_despawn.push(asteroid_entity);
                game_state.score += 10;
                let new_level = game_state.score / SCORE_PER_LEVEL + 1;
                if new_level > game_state.speed_level {
                    game_state.speed_level = new_level;
                }
                break;
            }
        }
    }
    for entity in bullets_to_despawn {
        commands.entity(entity).despawn();
    }
    for entity in asteroids_to_despawn {
        commands.entity(entity).despawn();
    }
}

pub fn player_asteroid_collision(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    asteroid_query: Query<(Entity, &Transform, &Sprite), With<Asteroid>>,
    mut game_state: ResMut<GameState>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    for (asteroid_entity, asteroid_transform, asteroid_sprite) in asteroid_query.iter() {
        let asteroid_size = asteroid_sprite.custom_size.unwrap_or(Vec2::splat(30.0));
        let collision = player_transform.translation.x - PLAYER_SIZE.x / 2.0
            < asteroid_transform.translation.x + asteroid_size.x / 2.0
            && player_transform.translation.x + PLAYER_SIZE.x / 2.0
                > asteroid_transform.translation.x - asteroid_size.x / 2.0
            && player_transform.translation.y - PLAYER_SIZE.y / 2.0
                < asteroid_transform.translation.y + asteroid_size.y / 2.0
            && player_transform.translation.y + PLAYER_SIZE.y / 2.0
                > asteroid_transform.translation.y - asteroid_size.y / 2.0;
        if collision {
            commands.entity(asteroid_entity).despawn();
            game_state.game_over = true;
            return;
        }
    }
}

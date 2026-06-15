use bevy::{
    color::Color,
    ecs::{
        entity::Entity,
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    input::{ButtonInput, keyboard::KeyCode},
    math::{Quat, Vec2, Vec3},
    prelude::{default, BuildChildren, ChildBuild, DespawnRecursiveExt},
    sprite::Sprite,
    time::Time,
    transform::components::Transform,
};

use crate::components::{
    Asteroid, AsteroidRotation, AsteroidSpeed, BASE_ASTEROID_SPEED, BULLET_SIZE, BULLET_SPEED,
    Bullet, EngineFlame, GameState, PLAYER_SIZE, PLAYER_SPEED, Player, PlayerWing, SCORE_PER_LEVEL,
    ShootCooldown, SpawnTimer, SPEED_INCREMENT, Star, WINDOW_HEIGHT, WINDOW_WIDTH,
};

pub fn spawn_player(commands: &mut Commands) {
    commands
        .spawn((
            Sprite {
                color: Color::srgb(0.3, 0.75, 1.0),
                custom_size: Some(Vec2::new(20.0, 44.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, -WINDOW_HEIGHT / 2.0 + 60.0, 0.0)),
            Player,
        ))
        .with_children(|parent| {
            parent.spawn((
                Sprite {
                    color: Color::srgb(0.15, 0.45, 0.85),
                    custom_size: Some(Vec2::new(18.0, 22.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(-16.0, -6.0, 0.0))
                    .with_rotation(Quat::from_rotation_z(0.35)),
                PlayerWing,
            ));
            parent.spawn((
                Sprite {
                    color: Color::srgb(0.15, 0.45, 0.85),
                    custom_size: Some(Vec2::new(18.0, 22.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(16.0, -6.0, 0.0))
                    .with_rotation(Quat::from_rotation_z(-0.35)),
                PlayerWing,
            ));
            parent.spawn((
                Sprite {
                    color: Color::srgb(1.0, 0.55, 0.1),
                    custom_size: Some(Vec2::new(8.0, 12.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, -28.0, 0.0)),
                EngineFlame,
            ));
        });
}

pub fn spawn_stars(commands: &mut Commands) {
    for _ in 0..120 {
        let x = (rand::random::<f32>() - 0.5) * WINDOW_WIDTH;
        let y = (rand::random::<f32>() - 0.5) * WINDOW_HEIGHT;
        let size = 1.0 + rand::random::<f32>() * 2.0;
        let brightness = 0.3 + rand::random::<f32>() * 0.7;
        commands.spawn((
            Sprite {
                color: Color::srgb(brightness, brightness, brightness + 0.05),
                custom_size: Some(Vec2::new(size, size)),
                ..default()
            },
            Transform::from_translation(Vec3::new(x, y, -1.0)),
            Star,
        ));
    }
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
    transform.translation.x = (transform.translation.x
        + direction * PLAYER_SPEED * time.delta_secs())
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
            color: Color::srgb(0.5, 1.0, 1.0),
            custom_size: Some(BULLET_SIZE),
            ..default()
        },
        Transform::from_translation(Vec3::new(
            player_transform.translation.x,
            player_transform.translation.y + PLAYER_SIZE.y / 2.0 + BULLET_SIZE.y / 2.0,
            0.5,
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
    let size = 20.0 + rand::random::<f32>() * 35.0;
    let half_width = WINDOW_WIDTH / 2.0 - size / 2.0;
    let x = (rand::random::<f32>() * 2.0 - 1.0) * half_width;
    let speed = BASE_ASTEROID_SPEED + (game_state.speed_level - 1) as f32 * SPEED_INCREMENT;
    let rotation_speed = (rand::random::<f32>() - 0.5) * 3.0;
    let gray = 0.35 + rand::random::<f32>() * 0.3;
    let r_tint = gray + rand::random::<f32>() * 0.1;
    let g_tint = gray * (0.8 + rand::random::<f32>() * 0.2);
    let b_tint = gray * (0.6 + rand::random::<f32>() * 0.2);
    let initial_rotation = rand::random::<f32>() * std::f32::consts::PI * 2.0;
    commands.spawn((
        Sprite {
            color: Color::srgb(r_tint, g_tint, b_tint),
            custom_size: Some(Vec2::new(size, size)),
            ..default()
        },
        Transform::from_translation(Vec3::new(x, WINDOW_HEIGHT / 2.0 + size / 2.0, 0.0))
            .with_rotation(Quat::from_rotation_z(initial_rotation)),
        Asteroid,
        AsteroidSpeed(speed),
        AsteroidRotation(rotation_speed),
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

pub fn rotate_asteroids(
    mut query: Query<(&mut Transform, &AsteroidRotation), With<Asteroid>>,
    time: Res<Time>,
    game_state: Res<GameState>,
) {
    if game_state.game_over {
        return;
    }
    for (mut transform, rotation) in query.iter_mut() {
        transform.rotate_z(rotation.0 * time.delta_secs());
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

pub fn flicker_flame(
    mut query: Query<&mut Sprite, With<EngineFlame>>,
    time: Res<Time>,
    game_state: Res<GameState>,
) {
    if game_state.game_over {
        return;
    }
    for mut sprite in query.iter_mut() {
        let t = time.elapsed_secs();
        let flicker = 0.7 + 0.3 * ((t * 18.0).sin());
        sprite.color = Color::srgb(1.0, 0.4 * flicker, 0.05 * flicker);
        let size_y = 10.0 + 6.0 * ((t * 14.0 + 1.0).sin());
        sprite.custom_size = Some(Vec2::new(8.0, size_y));
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
    player_query: Query<Entity, With<Player>>,
    player_transform_query: Query<&Transform, With<Player>>,
    asteroid_query: Query<(Entity, &Transform, &Sprite), With<Asteroid>>,
    mut game_state: ResMut<GameState>,
) {
    let Ok(player_transform) = player_transform_query.get_single() else {
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
            if let Ok(player_entity) = player_query.get_single() {
                commands.entity(player_entity).despawn_recursive();
            }
            game_state.game_over = true;
            return;
        }
    }
}

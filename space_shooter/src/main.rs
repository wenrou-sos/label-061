use bevy::{
    color::palettes::css::{RED, WHITE},
    prelude::*,
};

const WINDOW_WIDTH: f32 = 800.0;
const WINDOW_HEIGHT: f32 = 600.0;
const PLAYER_SPEED: f32 = 350.0;
const BULLET_SPEED: f32 = 500.0;
const BASE_ASTEROID_SPEED: f32 = 120.0;
const SPEED_INCREMENT: f32 = 30.0;
const SCORE_PER_LEVEL: u32 = 100;
const PLAYER_SIZE: Vec2 = Vec2::new(40.0, 50.0);
const BULLET_SIZE: Vec2 = Vec2::new(4.0, 16.0);
const SHOOT_COOLDOWN: f32 = 0.25;

#[derive(Resource)]
struct GameState {
    score: u32,
    speed_level: u32,
    game_over: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            score: 0,
            speed_level: 1,
            game_over: false,
        }
    }
}

#[derive(Resource)]
struct SpawnTimer {
    timer: Timer,
}

impl Default for SpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.8, TimerMode::Repeating),
        }
    }
}

#[derive(Resource)]
struct ShootCooldown {
    timer: Timer,
}

impl Default for ShootCooldown {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(SHOOT_COOLDOWN, TimerMode::Once),
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Bullet;

#[derive(Component)]
struct Asteroid;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct GameOverUI;

#[derive(Component)]
struct RestartButton;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    spawn_player(&mut commands);
    spawn_score_text(&mut commands);
}

fn spawn_player(commands: &mut Commands) {
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

fn spawn_score_text(commands: &mut Commands) {
    commands.spawn((
        Text::new("Score: 0"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        ScoreText,
    ));
}

fn player_movement(
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

fn player_shoot(
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

fn move_bullets(mut query: Query<&mut Transform, With<Bullet>>, time: Res<Time>) {
    for mut transform in query.iter_mut() {
        transform.translation.y += BULLET_SPEED * time.delta_secs();
    }
}

fn despawn_offscreen_bullets(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Bullet>>,
) {
    for (entity, transform) in query.iter() {
        if transform.translation.y > WINDOW_HEIGHT / 2.0 + 20.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_asteroids(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<SpawnTimer>,
    game_state: Res<GameState>,
) {
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

#[derive(Component)]
struct AsteroidSpeed(f32);

fn move_asteroids(
    mut query: Query<(&mut Transform, &AsteroidSpeed), With<Asteroid>>,
    time: Res<Time>,
) {
    for (mut transform, speed) in query.iter_mut() {
        transform.translation.y -= speed.0 * time.delta_secs();
    }
}

fn despawn_offscreen_asteroids(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Asteroid>>,
) {
    for (entity, transform) in query.iter() {
        if transform.translation.y < -WINDOW_HEIGHT / 2.0 - 50.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn bullet_asteroid_collision(
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

fn player_asteroid_collision(
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

fn update_score_text(
    game_state: Res<GameState>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    if game_state.is_changed() {
        let Ok(mut text) = query.get_single_mut() else {
            return;
        };
        text.0 = format!("Score: {}", game_state.score);
    }
}

fn check_game_over(
    mut commands: Commands,
    game_state: Res<GameState>,
    player_query: Query<Entity, With<Player>>,
    score_text_query: Query<Entity, With<ScoreText>>,
    existing_ui: Query<Entity, With<GameOverUI>>,
) {
    if !game_state.game_over {
        return;
    }
    for entity in player_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in score_text_query.iter() {
        commands.entity(entity).despawn();
    }
    if !existing_ui.is_empty() {
        return;
    }
    let score_val = game_state.score;
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            GameOverUI,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Game Over"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(RED.into()),
            ));
            parent.spawn((
                Text::new(format!("Final Score: {}", score_val)),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(WHITE.into()),
            ));
            parent
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::all(Val::Px(16.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.7, 0.3)),
                    RestartButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Restart"),
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(WHITE.into()),
                    ));
                });
        });
}

fn restart_game(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (With<RestartButton>, Changed<Interaction>)>,
    mut game_state: ResMut<GameState>,
    asteroids: Query<Entity, With<Asteroid>>,
    bullets: Query<Entity, With<Bullet>>,
    game_over_ui: Query<Entity, With<GameOverUI>>,
) {
    let Ok(interaction) = interaction_query.get_single() else {
        return;
    };
    if *interaction != Interaction::Pressed {
        return;
    }
    *game_state = GameState::default();
    for entity in asteroids.iter() {
        commands.entity(entity).despawn();
    }
    for entity in bullets.iter() {
        commands.entity(entity).despawn();
    }
    for entity in game_over_ui.iter() {
        commands.entity(entity).despawn();
    }
    spawn_player(&mut commands);
    spawn_score_text(&mut commands);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Space Shooter".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<GameState>()
        .init_resource::<SpawnTimer>()
        .init_resource::<ShootCooldown>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_movement,
                player_shoot,
                move_bullets,
                despawn_offscreen_bullets,
                spawn_asteroids,
                move_asteroids,
                despawn_offscreen_asteroids,
                bullet_asteroid_collision,
                player_asteroid_collision,
                update_score_text,
                check_game_over,
                restart_game,
            )
                .chain(),
        )
        .run();
}

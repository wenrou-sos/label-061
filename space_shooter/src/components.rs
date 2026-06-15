use bevy::{
    math::Vec2,
    prelude::{Component, Resource},
    time::{Timer, TimerMode},
};

pub const WINDOW_WIDTH: f32 = 800.0;
pub const WINDOW_HEIGHT: f32 = 600.0;
pub const PLAYER_SPEED: f32 = 350.0;
pub const BULLET_SPEED: f32 = 500.0;
pub const BASE_ASTEROID_SPEED: f32 = 120.0;
pub const SPEED_INCREMENT: f32 = 30.0;
pub const SCORE_PER_LEVEL: u32 = 100;
pub const PLAYER_SIZE: Vec2 = Vec2::new(44.0, 52.0);
pub const BULLET_SIZE: Vec2 = Vec2::new(6.0, 18.0);
pub const SHOOT_COOLDOWN: f32 = 0.25;
pub const HIGH_SCORE_FILE: &str = "high_score.json";

#[derive(Resource)]
pub struct GameState {
    pub score: u32,
    pub speed_level: u32,
    pub game_over: bool,
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
pub struct SpawnTimer {
    pub timer: Timer,
}

impl Default for SpawnTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.8, TimerMode::Repeating),
        }
    }
}

#[derive(Resource)]
pub struct ShootCooldown {
    pub timer: Timer,
}

impl Default for ShootCooldown {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(SHOOT_COOLDOWN, TimerMode::Once),
        }
    }
}

#[derive(Resource, Default)]
pub struct HighScore {
    pub score: u32,
    pub is_new_record: bool,
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerWing;

#[derive(Component)]
pub struct EngineFlame;

#[derive(Component)]
pub struct Bullet;

#[derive(Component)]
pub struct Asteroid;

#[derive(Component)]
pub struct AsteroidSpeed(pub f32);

#[derive(Component)]
pub struct AsteroidRotation(pub f32);

#[derive(Component)]
pub struct Star;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct GameOverUI;

#[derive(Component)]
pub struct RestartButton;

#[derive(Component)]
pub struct PlayAgainButton;

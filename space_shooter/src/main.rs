use bevy::prelude::*;

mod components;
mod game;
mod score;
mod ui;

use components::{GameState, ShootCooldown, SpawnTimer, WINDOW_HEIGHT, WINDOW_WIDTH};
use game::{
    bullet_asteroid_collision, despawn_offscreen_asteroids, despawn_offscreen_bullets,
    flicker_flame, move_asteroids, move_bullets, player_asteroid_collision, player_movement,
    player_shoot, rotate_asteroids, spawn_asteroids, spawn_player, spawn_stars,
};
use score::{check_and_save_high_score, load_high_score};
use ui::{check_game_over, play_again, restart_game, spawn_score_text, update_score_text};

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    spawn_player(&mut commands);
    spawn_score_text(&mut commands);
    spawn_stars(&mut commands);
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Space Shooter".to_string(),
                    resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                    ..default()
                }),
                ..default()
            }),
        )
        .init_resource::<GameState>()
        .init_resource::<SpawnTimer>()
        .init_resource::<ShootCooldown>()
        .add_systems(Startup, (setup, load_high_score))
        .add_systems(
            Update,
            (
                player_movement,
                player_shoot,
                move_bullets,
                despawn_offscreen_bullets,
                spawn_asteroids,
                move_asteroids,
                rotate_asteroids,
                despawn_offscreen_asteroids,
                bullet_asteroid_collision,
                player_asteroid_collision,
                flicker_flame,
                update_score_text,
                check_and_save_high_score,
                check_game_over,
                restart_game,
                play_again,
            )
                .chain(),
        )
        .run();
}

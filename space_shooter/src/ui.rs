use bevy::{
    color::{Color, palettes::css::{GOLD, RED, WHITE}},
    ecs::{
        change_detection::DetectChanges,
        entity::Entity,
        query::{Changed, With},
        system::{Commands, Query, Res, ResMut},
    },
    prelude::{default, BuildChildren, ChildBuild, BackgroundColor, Button, Interaction, Node, Text, TextColor, TextFont},
    ui::{FlexDirection, JustifyContent, AlignItems, PositionType, UiRect, Val},
};

use crate::components::{
    Asteroid, Bullet, GameOverUI, GameState, HighScore, PlayAgainButton, Player,
    RestartButton, ScoreText,
};
use crate::game::spawn_player;

pub fn spawn_score_text(commands: &mut Commands) {
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

pub fn update_score_text(
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

pub fn check_game_over(
    mut commands: Commands,
    game_state: Res<GameState>,
    high_score: Res<HighScore>,
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
    let high_score_val = high_score.score;
    let is_new_record = high_score.is_new_record;

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
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

            if is_new_record {
                parent.spawn((
                    Text::new("New Record!"),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(GOLD.into()),
                ));
            }

            parent.spawn((
                Text::new(format!("Final Score: {}", score_val)),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(WHITE.into()),
            ));

            parent.spawn((
                Text::new(format!("High Score: {}", high_score_val)),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(GOLD.into()),
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

            parent
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::all(Val::Px(16.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.5, 0.9)),
                    PlayAgainButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("Play Again"),
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(WHITE.into()),
                    ));
                });
        });
}

fn reset_game(
    commands: &mut Commands,
    game_state: &mut ResMut<GameState>,
    high_score: &mut ResMut<HighScore>,
    asteroids: &Query<Entity, With<Asteroid>>,
    bullets: &Query<Entity, With<Bullet>>,
    game_over_ui: &Query<Entity, With<GameOverUI>>,
) {
    **game_state = GameState::default();
    high_score.is_new_record = false;
    for entity in asteroids.iter() {
        commands.entity(entity).despawn();
    }
    for entity in bullets.iter() {
        commands.entity(entity).despawn();
    }
    for entity in game_over_ui.iter() {
        commands.entity(entity).despawn();
    }
    spawn_player(commands);
    spawn_score_text(commands);
}

pub fn restart_game(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (With<RestartButton>, Changed<Interaction>)>,
    mut game_state: ResMut<GameState>,
    mut high_score: ResMut<HighScore>,
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
    reset_game(
        &mut commands,
        &mut game_state,
        &mut high_score,
        &asteroids,
        &bullets,
        &game_over_ui,
    );
}

pub fn play_again(
    mut commands: Commands,
    interaction_query: Query<&Interaction, (With<PlayAgainButton>, Changed<Interaction>)>,
    mut game_state: ResMut<GameState>,
    mut high_score: ResMut<HighScore>,
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
    reset_game(
        &mut commands,
        &mut game_state,
        &mut high_score,
        &asteroids,
        &bullets,
        &game_over_ui,
    );
}

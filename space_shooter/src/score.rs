use std::fs;

use bevy::ecs::{
    change_detection::DetectChanges,
    system::{Commands, Res, ResMut},
};
use serde::{Deserialize, Serialize};

use crate::components::{GameState, HIGH_SCORE_FILE, HighScore};

#[derive(Serialize, Deserialize)]
struct HighScoreData {
    score: u32,
}

pub fn load_high_score(mut commands: Commands) {
    let score = fs::read_to_string(HIGH_SCORE_FILE)
        .ok()
        .and_then(|content| serde_json::from_str::<HighScoreData>(&content).ok())
        .map(|data| data.score)
        .unwrap_or(0);
    commands.insert_resource(HighScore {
        score,
        is_new_record: false,
    });
}

pub fn check_and_save_high_score(game_state: Res<GameState>, mut high_score: ResMut<HighScore>) {
    if game_state.is_changed() && game_state.game_over {
        high_score.is_new_record = game_state.score > high_score.score;
        if high_score.is_new_record {
            high_score.score = game_state.score;
            let data = HighScoreData {
                score: high_score.score,
            };
            if let Ok(json) = serde_json::to_string(&data) {
                let _ = fs::write(HIGH_SCORE_FILE, json);
            }
        }
    }
}

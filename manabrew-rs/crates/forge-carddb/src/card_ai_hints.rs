use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardAiHints {
    pub rem_ai_decks: bool,
    pub rem_random_decks: bool,
    pub rem_non_commander_decks: bool,
}

use forge_foundation::ColorSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckColors {
    chosen: ColorSet,
    locked: bool,
    #[serde(default)]
    picks: u32,
    #[serde(default)]
    color_weights: [f64; 5],
}

impl Default for DeckColors {
    fn default() -> Self {
        Self {
            chosen: ColorSet::COLORLESS,
            locked: false,
            picks: 0,
            color_weights: [0.0; 5],
        }
    }
}

const COLORS: [ColorSet; 5] = [
    ColorSet::WHITE,
    ColorSet::BLUE,
    ColorSet::BLACK,
    ColorSet::RED,
    ColorSet::GREEN,
];

impl DeckColors {
    pub const MAX_COLORS: u32 = 2;
    pub const PICKS_BEFORE_COMMITTING: u32 = 8;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn chosen(&self) -> ColorSet {
        self.chosen
    }

    pub fn count(&self) -> u32 {
        self.chosen.count_colors()
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn can_choose_more_colors(&self) -> bool {
        !self.locked && self.chosen.count_colors() < Self::MAX_COLORS
    }

    pub fn observe(&mut self, card_colors: ColorSet, score: f64) -> bool {
        if self.locked {
            return false;
        }
        self.picks += 1;
        let count = card_colors.count_colors();
        if count > 0 {
            let share = score.max(0.0) / count as f64;
            for (i, color) in COLORS.iter().enumerate() {
                if card_colors.shares_color_with(*color) {
                    self.color_weights[i] += share;
                }
            }
        }
        if self.picks < Self::PICKS_BEFORE_COMMITTING {
            return false;
        }
        let mut order: Vec<usize> = (0..5).filter(|&i| self.color_weights[i] > 0.0).collect();
        order.sort_by(|&a, &b| self.color_weights[b].total_cmp(&self.color_weights[a]));
        self.chosen = order
            .into_iter()
            .take(Self::MAX_COLORS as usize)
            .fold(ColorSet::COLORLESS, |acc, i| acc.union(COLORS[i]));
        self.locked = true;
        true
    }
}

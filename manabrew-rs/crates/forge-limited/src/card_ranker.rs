use std::sync::Arc;

use forge_foundation::sealed_product::{PaperCard, Rarity};
use forge_foundation::ColorSet;
use serde::{Deserialize, Serialize};

use crate::card_ranking_comparator::CardRankingComparator;
use crate::draft_rank_cache::DraftRankCache;
use crate::magic_db::rules_of;
use crate::read_draft_rankings::ReadDraftRankings;

const BASIC_LANDS: [&str; 5] = ["Plains", "Island", "Swamp", "Mountain", "Forest"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardRankerState {
    by_set: std::collections::HashMap<String, std::collections::HashMap<String, u32>>,
    custom_rankings: Option<std::collections::HashMap<String, u32>>,
}

#[derive(Serialize)]
pub struct CardRanker {
    #[serde(rename = "by_set")]
    rank_cache: Arc<DraftRankCache>,
    custom_rankings: Option<std::collections::HashMap<String, u32>>,
}

impl CardRanker {
    pub const SCORE_UNPICKABLE: f64 = -100.0;

    pub fn new(rank_cache: Arc<DraftRankCache>) -> Self {
        Self {
            rank_cache,
            custom_rankings: None,
        }
    }

    pub fn export_state(&self) -> CardRankerState {
        CardRankerState {
            by_set: self.rank_cache.export_state(),
            custom_rankings: self.custom_rankings.clone(),
        }
    }

    pub fn from_state(state: CardRankerState) -> Self {
        Self {
            rank_cache: Arc::new(DraftRankCache::from_state(state.by_set)),
            custom_rankings: state.custom_rankings,
        }
    }

    pub fn with_custom_rankings(
        mut self,
        rankings: std::collections::HashMap<String, u32>,
    ) -> Self {
        self.custom_rankings = Some(rankings);
        self
    }

    pub fn with_cube_rankings(self, pool: &[PaperCard]) -> Self {
        let Some(body) = crate::magic_db::cube_rankings() else {
            return self;
        };
        let all = ReadDraftRankings::parse(body);
        let rankings = pool
            .iter()
            .filter_map(|card| {
                let key = card.name.to_lowercase();
                all.get(&key).map(|&rank| (key, rank))
            })
            .collect();
        self.with_custom_rankings(rankings)
    }

    pub fn get_raw_score(&self, card: &PaperCard) -> f64 {
        if BASIC_LANDS.contains(&card.name.as_str()) {
            return Self::SCORE_UNPICKABLE;
        }
        let rkg = match &self.custom_rankings {
            Some(custom) => custom
                .get(&card.name.to_lowercase())
                .map(|&rank| DraftRankCache::relative(rank, custom))
                .or_else(|| self.default_ranking(card).map(|r| r + 1.0))
                .map(|r| r / 2.0),
            None => self.default_ranking(card),
        };
        match rkg {
            Some(rkg) => 100.0 - 100.0 * rkg,
            None => match card.rarity {
                Rarity::Mythic => 20.0,
                Rarity::Rare => 15.0,
                Rarity::Uncommon => 10.0,
                Rarity::Common => 5.0,
                Rarity::BasicLand => Self::SCORE_UNPICKABLE,
                Rarity::Special | Rarity::Token | Rarity::Unknown => 0.0,
            },
        }
    }

    fn default_ranking(&self, card: &PaperCard) -> Option<f64> {
        self.rank_cache
            .ranking(&card.set_code, &card.name)
            .or_else(|| self.rank_cache.best_ranking(&card.name))
    }

    fn rem_ai_decks(card: &PaperCard) -> bool {
        rules_of(card).is_some_and(|rules| rules.ai_hints.rem_ai_decks)
    }

    pub fn get_scores(&self, cards: &[PaperCard]) -> Vec<(f64, PaperCard)> {
        cards
            .iter()
            .map(|c| {
                let mut score = self.get_raw_score(c);
                if Self::rem_ai_decks(c) {
                    score -= 20.0;
                }
                (score, c.clone())
            })
            .collect()
    }

    pub fn get_ordered_raw_scores(&self, cards: &[PaperCard]) -> Vec<PaperCard> {
        let mut scored: Vec<(f64, PaperCard)> = cards
            .iter()
            .map(|c| (self.get_raw_score(c), c.clone()))
            .collect();
        CardRankingComparator::sort(&mut scored);
        scored.into_iter().map(|(_, c)| c).collect()
    }

    pub fn rank_cards_in_deck(&self, cards: &[PaperCard]) -> Vec<PaperCard> {
        let mut scored = self.get_scores(cards);
        CardRankingComparator::sort(&mut scored);
        scored.into_iter().map(|(_, c)| c).collect()
    }

    pub fn rank_cards_in_pack(
        &self,
        pack: &[PaperCard],
        deck: &[PaperCard],
        chosen_colors: ColorSet,
        can_add_more_colors: bool,
        card_color_lookup: impl Fn(&PaperCard) -> ColorSet,
    ) -> Vec<PaperCard> {
        let _ = deck;
        let mut scored: Vec<(f64, PaperCard)> = pack
            .iter()
            .map(|c| {
                let mut score = self.get_raw_score(c);
                if Self::rem_ai_decks(c) {
                    score -= 20.0;
                }
                let colors = rules_of(c)
                    .map(|rules| rules.deckbuilding_colors())
                    .unwrap_or_else(|| card_color_lookup(c));
                if !can_add_more_colors && !chosen_colors.contains_all_colors_from(colors) {
                    score -= 50.0;
                }
                (score, c.clone())
            })
            .collect();
        CardRankingComparator::sort(&mut scored);
        scored.into_iter().map(|(_, c)| c).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cache() -> Arc<DraftRankCache> {
        let c = DraftRankCache::new();
        c.register_from_rnk("TST", "1|Best|C|TST\n10|Mid|C|TST\n100|Filler|C|TST\n");
        Arc::new(c)
    }

    fn card(name: &str, rarity: Rarity) -> PaperCard {
        PaperCard::new(name, "TST", "1", rarity)
    }

    #[test]
    fn higher_rank_wins() {
        let r = CardRanker::new(cache());
        let pack = vec![
            card("Filler", Rarity::Common),
            card("Best", Rarity::Common),
            card("Mid", Rarity::Common),
        ];
        let order = r.get_ordered_raw_scores(&pack);
        assert_eq!(order[0].name, "Best");
        assert_eq!(order[1].name, "Mid");
        assert_eq!(order[2].name, "Filler");
    }

    #[test]
    fn rarity_fallback_when_no_rank() {
        let r = CardRanker::new(Arc::new(DraftRankCache::new()));
        let pack = vec![
            card("RareUnranked", Rarity::Rare),
            card("CommonUnranked", Rarity::Common),
        ];
        let order = r.get_ordered_raw_scores(&pack);
        assert_eq!(order[0].name, "RareUnranked");
    }

    #[test]
    fn off_color_locked_drops_to_unpickable() {
        let r = CardRanker::new(cache());
        let red = card("Best", Rarity::Common);
        let blue = card("Mid", Rarity::Common);
        let order = r.rank_cards_in_pack(
            &[red.clone(), blue.clone()],
            &[],
            ColorSet::WHITE,
            false,
            |c| {
                if c.name == "Best" {
                    ColorSet::RED
                } else {
                    ColorSet::WHITE
                }
            },
        );
        assert_eq!(order[0].name, "Mid");
    }
}

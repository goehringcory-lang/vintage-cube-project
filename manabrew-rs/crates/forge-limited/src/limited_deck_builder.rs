use std::sync::Arc;

use forge_foundation::sealed_product::{PaperCard, Rarity};
use forge_foundation::ColorSet;
use serde::{Deserialize, Serialize};

use crate::card_ranker::CardRanker;
use crate::card_ranking_comparator::CardRankingComparator;
use crate::deck_colors::DeckColors;
use crate::magic_db::rules_of;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitedDeck {
    pub name: String,
    pub main: Vec<PaperCard>,
    pub sideboard: Vec<PaperCard>,
}

pub struct LimitedDeckBuilder {
    available: Vec<PaperCard>,
    deck_colors: DeckColors,
    ranker: Arc<CardRanker>,
    color_of: Box<dyn Fn(&PaperCard) -> ColorSet + Send + Sync>,
    is_land: Box<dyn Fn(&PaperCard) -> bool + Send + Sync>,
}

struct CardInfo {
    color: ColorSet,
    color_identity: ColorSet,
    deckbuilding_colors: ColorSet,
    is_land: bool,
    is_creature: bool,
    is_planeswalker: bool,
    cmc: i32,
    shard_masks: Vec<u8>,
    no_cost: bool,
    kept_in_ai_limited_decks: bool,
}

const BASIC_LANDS: [&str; 5] = ["Plains", "Island", "Swamp", "Mountain", "Forest"];
const COLORS: [ColorSet; 5] = [
    ColorSet::WHITE,
    ColorSet::BLUE,
    ColorSet::BLACK,
    ColorSet::RED,
    ColorSet::GREEN,
];

struct Build<'a> {
    cards: &'a [PaperCard],
    infos: Vec<CardInfo>,
    scores: Vec<f64>,
    colors: ColorSet,
    ai_playables: Vec<usize>,
    ranked_color_list: Vec<usize>,
    deck_list: Vec<usize>,
    basics: Vec<PaperCard>,
    lands_needed: usize,
    land_set_code: String,
}

impl LimitedDeckBuilder {
    pub const DECK_SIZE: usize = 40;
    pub const LAND_PERCENTAGE: f32 = 0.44;
    const NUM_SPELLS_NEEDED: usize = 22;
    const MAX_COLORLESS_NONBASIC_LANDS: usize = 3;
    const TARGET_CMCS: [usize; 6] = [2, 6, 7, 4, 3, 2];

    pub fn new(
        available: Vec<PaperCard>,
        deck_colors: DeckColors,
        ranker: Arc<CardRanker>,
        color_of: impl Fn(&PaperCard) -> ColorSet + Send + Sync + 'static,
        is_land: impl Fn(&PaperCard) -> bool + Send + Sync + 'static,
    ) -> Self {
        Self {
            available,
            deck_colors,
            ranker,
            color_of: Box::new(color_of),
            is_land: Box::new(is_land),
        }
    }

    pub fn deck_colors(&self) -> &DeckColors {
        &self.deck_colors
    }

    fn card_info(&self, card: &PaperCard) -> CardInfo {
        match rules_of(card) {
            Some(rules) => {
                let mana_cost = rules.mana_cost();
                let type_line = &rules.main_part.type_line;
                CardInfo {
                    color: rules.color(),
                    color_identity: rules.color_identity,
                    deckbuilding_colors: rules.deckbuilding_colors(),
                    is_land: type_line.is_land(),
                    is_creature: type_line.is_creature(),
                    is_planeswalker: type_line.is_planeswalker(),
                    cmc: rules.cmc(),
                    shard_masks: mana_cost.shards().iter().map(|s| s.color_mask()).collect(),
                    no_cost: mana_cost.is_no_cost(),
                    kept_in_ai_limited_decks: !rules.ai_hints.rem_ai_decks
                        && !rules.ai_hints.rem_non_commander_decks,
                }
            }
            None => {
                let color = (self.color_of)(card);
                let is_land = (self.is_land)(card);
                CardInfo {
                    color,
                    color_identity: color,
                    deckbuilding_colors: color,
                    is_land,
                    is_creature: false,
                    is_planeswalker: false,
                    cmc: 0,
                    shard_masks: color.iter().map(|c| c.mask()).collect(),
                    no_cost: is_land,
                    kept_in_ai_limited_decks: true,
                }
            }
        }
    }

    pub fn build_deck(
        &mut self,
        deck_name: impl Into<String>,
        land_set_code: Option<&str>,
    ) -> LimitedDeck {
        let infos: Vec<CardInfo> = self.available.iter().map(|c| self.card_info(c)).collect();
        let scores: Vec<f64> = self
            .ranker
            .get_scores(&self.available)
            .into_iter()
            .map(|(score, _)| score)
            .collect();
        let ai_playables: Vec<usize> = (0..self.available.len())
            .filter(|&i| infos[i].kept_in_ai_limited_decks)
            .collect();
        let mut build = Build {
            cards: &self.available,
            infos,
            scores,
            colors: ColorSet::COLORLESS,
            ai_playables,
            ranked_color_list: Vec::new(),
            deck_list: Vec::new(),
            basics: Vec::new(),
            lands_needed: Self::DECK_SIZE - Self::NUM_SPELLS_NEEDED,
            land_set_code: land_set_code.unwrap_or("").to_string(),
        };
        build.colors = build.choose_colors();
        build.run();

        let main: Vec<PaperCard> = build
            .deck_list
            .iter()
            .map(|&i| self.available[i].clone())
            .chain(build.basics.iter().cloned())
            .collect();
        let sideboard: Vec<PaperCard> = (0..self.available.len())
            .filter(|i| !build.deck_list.contains(i))
            .map(|i| self.available[i].clone())
            .collect();
        LimitedDeck {
            name: deck_name.into(),
            main,
            sideboard,
        }
    }
}

impl Build<'_> {
    fn rank(&self, indices: impl IntoIterator<Item = usize>) -> Vec<usize> {
        let mut scored: Vec<(f64, PaperCard, usize)> = indices
            .into_iter()
            .map(|i| (self.scores[i], self.cards[i].clone(), i))
            .collect();
        scored.sort_by(|a, b| {
            CardRankingComparator::compare(&(a.0, a.1.clone()), &(b.0, b.1.clone()))
        });
        scored.into_iter().map(|(_, _, i)| i).collect()
    }

    fn has_color(&self, i: usize, colors: ColorSet) -> bool {
        let info = &self.infos[i];
        !info.no_cost && !info.is_land && colors.contains_all_colors_from(info.color_identity)
    }

    fn choose_colors(&self) -> ColorSet {
        let ranked = self.rank(self.ai_playables.iter().copied());
        let limit = (ranked.len() as f64 * 0.33).ceil() as usize;
        let mut counts = [0usize; 5];
        for &i in ranked.iter().take(limit) {
            for (c, mask) in COLORS.iter().enumerate() {
                if self.infos[i].color.shares_color_with(*mask) {
                    counts[c] += 1;
                }
            }
        }
        let mut order: Vec<usize> = (0..5).collect();
        order.sort_by(|&a, &b| counts[b].cmp(&counts[a]));
        COLORS[order[0]].union(COLORS[order[1]])
    }

    fn take(&mut self, i: usize) {
        self.deck_list.push(i);
        self.ai_playables.retain(|&x| x != i);
        self.ranked_color_list.retain(|&x| x != i);
    }

    fn spells_missing(&self) -> usize {
        LimitedDeckBuilder::NUM_SPELLS_NEEDED.saturating_sub(self.deck_list.len())
    }

    fn run(&mut self) {
        let colors = self.colors;
        let color_list: Vec<usize> = self
            .ai_playables
            .iter()
            .copied()
            .filter(|&i| self.has_color(i, colors))
            .collect();
        self.ranked_color_list = self.rank(color_list);

        let walkers: Vec<usize> = self
            .ranked_color_list
            .iter()
            .copied()
            .filter(|&i| self.infos[i].is_planeswalker)
            .collect();
        for i in walkers {
            self.take(i);
        }

        self.add_mana_curve_creatures(16);
        let missing = self.spells_missing();
        self.add_non_creatures(missing);
        let missing = self.spells_missing();
        self.add_creatures(missing);

        if self.deck_list.len() == LimitedDeckBuilder::NUM_SPELLS_NEEDED && self.average_cmc() < 4.0
        {
            if let Some(&i) = self.ranked_color_list.first() {
                self.take(i);
                self.lands_needed -= 1;
            }
        }

        let missing = self.spells_missing();
        self.add_third_color_cards(missing);
        self.add_non_basic_lands();

        let clr_cnts = self.calculate_land_needs();
        if self.lands_needed > 0 {
            self.add_lands(clr_cnts);
        }
        self.fix_deck_size(clr_cnts);
    }

    fn add_mana_curve_creatures(&mut self, mut num: usize) {
        let mut creature_costs = [0usize; 6];
        let creatures: Vec<usize> = self
            .ranked_color_list
            .iter()
            .copied()
            .filter(|&i| self.infos[i].is_creature)
            .collect();
        for i in creatures {
            if num == 0 {
                break;
            }
            let slot = (self.infos[i].cmc.clamp(1, 6) - 1) as usize;
            if creature_costs[slot] < LimitedDeckBuilder::TARGET_CMCS[slot] {
                creature_costs[slot] += 1;
                self.take(i);
                num -= 1;
            }
        }
    }

    fn add_creatures(&mut self, num: usize) {
        let creatures: Vec<usize> = self
            .ranked_color_list
            .iter()
            .copied()
            .filter(|&i| self.infos[i].is_creature)
            .take(num)
            .collect();
        for i in creatures {
            self.take(i);
        }
    }

    fn add_non_creatures(&mut self, num: usize) {
        let others: Vec<usize> = self
            .ranked_color_list
            .iter()
            .copied()
            .filter(|&i| !self.infos[i].is_creature && !self.infos[i].is_land)
            .take(num)
            .collect();
        for i in others {
            self.take(i);
        }
    }

    fn average_cmc(&self) -> f64 {
        if self.deck_list.is_empty() {
            return 0.0;
        }
        let sum: i32 = self.deck_list.iter().map(|&i| self.infos[i].cmc).sum();
        sum as f64 / self.deck_list.len() as f64
    }

    fn add_third_color_cards(&mut self, num: usize) {
        if num == 0 {
            return;
        }
        let others = self.rank(
            self.ai_playables
                .iter()
                .copied()
                .filter(|&i| !self.infos[i].is_land),
        );
        for &i in &others {
            let off = ColorSet::from_mask(self.infos[i].color.mask() & !self.colors.mask());
            if off.is_mono_color() {
                self.colors = self.colors.union(off);
                break;
            }
        }
        let colors = self.colors;
        let to_add: Vec<usize> = others
            .into_iter()
            .filter(|&i| self.has_color(i, colors))
            .take(num)
            .collect();
        for i in to_add {
            self.take(i);
        }
    }

    fn add_non_basic_lands(&mut self) {
        let colors = self.colors;
        let mut lands: Vec<usize> = self
            .ai_playables
            .iter()
            .copied()
            .filter(|&i| {
                let info = &self.infos[i];
                info.is_land
                    && !self.cards[i].rarity.eq(&Rarity::BasicLand)
                    && colors.contains_all_colors_from(info.deckbuilding_colors)
            })
            .collect();
        lands.sort_by_key(|&i| {
            std::cmp::Reverse(
                self.infos[i]
                    .deckbuilding_colors
                    .intersection(colors)
                    .count_colors(),
            )
        });
        let mut colorless = 0;
        for i in lands {
            if self.lands_needed == 0 {
                break;
            }
            if self.infos[i].deckbuilding_colors.is_colorless() {
                if colorless == LimitedDeckBuilder::MAX_COLORLESS_NONBASIC_LANDS {
                    continue;
                }
                colorless += 1;
            }
            self.take(i);
            self.lands_needed -= 1;
        }
    }

    fn calculate_land_needs(&self) -> [usize; 5] {
        let mut clr_cnts = [0usize; 5];
        for &i in &self.deck_list {
            for &mask in &self.infos[i].shard_masks {
                for (c, color) in COLORS.iter().enumerate() {
                    if mask & color.mask() != 0 && self.colors.shares_color_with(*color) {
                        clr_cnts[c] += 1;
                    }
                }
            }
        }
        if clr_cnts.iter().all(|&n| n == 0) {
            for (c, color) in COLORS.iter().enumerate() {
                if self.colors.shares_color_with(*color) {
                    clr_cnts[c] = 1;
                }
            }
        }
        clr_cnts
    }

    fn basic_land(&mut self, color: usize) -> PaperCard {
        let name = match color {
            5 => "Wastes",
            c => BASIC_LANDS[c],
        };
        PaperCard::new(
            name,
            self.land_set_code.clone(),
            format!("basic{}", self.basics.len()),
            Rarity::BasicLand,
        )
    }

    fn main_size(&self) -> usize {
        self.deck_list.len() + self.basics.len()
    }

    fn add_lands(&mut self, clr_cnts: [usize; 5]) {
        let mut total_color: usize = clr_cnts.iter().sum();
        if total_color == 0 {
            return;
        }
        let target = self.basics.len() + self.lands_needed;
        for (c, &cnt) in clr_cnts.iter().enumerate() {
            if cnt > 0 && self.basics.len() + 2 <= target {
                for _ in 0..2 {
                    let land = self.basic_land(c);
                    self.basics.push(land);
                }
            }
        }
        for (c, &cnt) in clr_cnts.iter().enumerate() {
            if cnt == 0 {
                continue;
            }
            let slots_remaining = target.saturating_sub(self.basics.len());
            let n_land =
                (slots_remaining as f64 * cnt as f64 / total_color as f64).round() as usize;
            total_color -= cnt;
            for _ in 0..n_land {
                let land = self.basic_land(c);
                self.basics.push(land);
            }
        }
    }

    fn fix_deck_size(&mut self, clr_cnts: [usize; 5]) {
        while self.main_size() > LimitedDeckBuilder::DECK_SIZE {
            if self.basics.pop().is_some() {
                continue;
            }
            let worst = self
                .rank(
                    self.deck_list
                        .iter()
                        .copied()
                        .filter(|&i| !self.infos[i].is_land),
                )
                .last()
                .copied();
            if let Some(i) = worst {
                self.deck_list.retain(|&x| x != i);
                self.ai_playables.push(i);
            }
        }
        while self.main_size() < LimitedDeckBuilder::DECK_SIZE {
            let best = self
                .rank(
                    self.ai_playables
                        .iter()
                        .copied()
                        .filter(|&i| !self.infos[i].is_land),
                )
                .first()
                .copied()
                .or_else(|| self.ai_playables.first().copied());
            match best {
                Some(i) => self.take(i),
                None => {
                    let color = clr_cnts.iter().position(|&n| n > 0).unwrap_or(5);
                    let land = self.basic_land(color);
                    self.basics.push(land);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draft_rank_cache::DraftRankCache;

    fn pool() -> Vec<PaperCard> {
        let mut v = Vec::new();
        for i in 0..30 {
            v.push(PaperCard::new(
                format!("Spell {i}"),
                "TST",
                format!("s{i}"),
                Rarity::Common,
            ));
        }
        for i in 0..10 {
            v.push(PaperCard::new(
                format!("Forest {i}"),
                "TST",
                format!("l{i}"),
                Rarity::BasicLand,
            ));
        }
        v
    }

    #[test]
    fn builds_a_40_card_deck() {
        let cache = Arc::new(DraftRankCache::new());
        let ranker = Arc::new(CardRanker::new(cache));
        let mut builder = LimitedDeckBuilder::new(
            pool(),
            DeckColors::new(),
            ranker,
            |_| ColorSet::COLORLESS,
            |c| c.rarity == Rarity::BasicLand,
        );
        let deck = builder.build_deck("test", Some("TST"));
        assert_eq!(deck.main.len(), LimitedDeckBuilder::DECK_SIZE);
        let lands = deck
            .main
            .iter()
            .filter(|c| c.rarity == Rarity::BasicLand)
            .count();
        assert!((17..=18).contains(&lands), "lands = {lands}");
    }

    #[test]
    fn synthesises_basics_when_pool_lacks_them() {
        let cache = Arc::new(DraftRankCache::new());
        let ranker = Arc::new(CardRanker::new(cache));
        let mut spells = Vec::new();
        for i in 0..30 {
            spells.push(PaperCard::new(
                format!("Spell {i}"),
                "TST",
                format!("s{i}"),
                Rarity::Common,
            ));
        }
        let mut builder = LimitedDeckBuilder::new(
            spells,
            DeckColors::new(),
            ranker,
            |_| ColorSet::COLORLESS,
            |_| false,
        );
        let deck = builder.build_deck("test", Some("M21"));
        assert_eq!(deck.main.len(), 40);
        let basics = deck.main.iter().filter(|c| c.set_code == "M21").count();
        assert!(basics >= 17, "basics = {basics}");
    }
}

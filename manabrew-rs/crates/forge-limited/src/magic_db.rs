use std::sync::OnceLock;

use forge_carddb::CardRules;
use forge_foundation::sealed_product::PaperCard;

pub struct MagicDb {
    pub card_rules: fn(&str) -> Option<&'static CardRules>,
    pub cube_rankings: fn() -> Option<&'static str>,
}

static MAGIC_DB: OnceLock<MagicDb> = OnceLock::new();

pub fn set_magic_db(db: MagicDb) {
    let _ = MAGIC_DB.set(db);
}

pub fn rules_of(card: &PaperCard) -> Option<&'static CardRules> {
    MAGIC_DB.get().and_then(|db| (db.card_rules)(&card.name))
}

pub fn cube_rankings() -> Option<&'static str> {
    MAGIC_DB.get().and_then(|db| (db.cube_rankings)())
}

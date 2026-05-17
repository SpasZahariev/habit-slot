use crate::models::{RewardTier, SlotSymbol};

const LOW_HEART_URI: &str = concat!(
    "data:image/png;base64,",
    include_str!("low_heart_base64.txt")
);
const MED_SKULL_URI: &str = concat!(
    "data:image/png;base64,",
    include_str!("med_crystal_base64.txt")
);
const HIGH_CHEST_URI: &str = concat!(
    "data:image/png;base64,",
    include_str!("high_chest_base64.txt")
);

const EXTRA_ROLL_COINS_URI: &str = concat!(
    "data:image/png;base64,",
    include_str!("extra_roll_coins_base64.txt")
);

/// Single source of truth for all slot symbol properties.
/// To add a new symbol: add an entry to this array and update the SlotSymbol enum.
pub static SYMBOLS: [SpriteConfig; 7] = [
    SpriteConfig {
        display_name: "Heart",
        tier: RewardTier::Small,
        weight: 25.0,
        payout_multiplier: 2,
        sprite_uri: LOW_HEART_URI,
        gray_at_low_bet: false,
    },
    SpriteConfig {
        display_name: "Heart",
        tier: RewardTier::Small,
        weight: 20.0,
        payout_multiplier: 4,
        sprite_uri: LOW_HEART_URI,
        gray_at_low_bet: false,
    },
    SpriteConfig {
        display_name: "Heart",
        tier: RewardTier::Small,
        weight: 15.0,
        payout_multiplier: 3,
        sprite_uri: LOW_HEART_URI,
        gray_at_low_bet: false,
    },
    SpriteConfig {
        display_name: "Skull",
        tier: RewardTier::Medium,
        weight: 18.0,
        payout_multiplier: 8,
        sprite_uri: MED_SKULL_URI,
        gray_at_low_bet: true,
    },
    SpriteConfig {
        display_name: "Skull",
        tier: RewardTier::Medium,
        weight: 12.0,
        payout_multiplier: 12,
        sprite_uri: MED_SKULL_URI,
        gray_at_low_bet: true,
    },
    SpriteConfig {
        display_name: "Chest",
        tier: RewardTier::Jackpot,
        weight: 10.0,
        payout_multiplier: 50,
        sprite_uri: HIGH_CHEST_URI,
        gray_at_low_bet: true,
    },
    SpriteConfig {
        display_name: "ExtraRoll",
        tier: RewardTier::ExtraRoll,
        weight: 8.0,
        payout_multiplier: 0,
        sprite_uri: EXTRA_ROLL_COINS_URI,
        gray_at_low_bet: false,
    },
];

/// Per-symbol configuration. Each array index maps to a SlotSymbol variant in order.
#[derive(Clone, Copy)]
pub struct SpriteConfig {
    pub display_name: &'static str,
    pub tier: RewardTier,
    pub weight: f64,
    pub payout_multiplier: u32,
    pub sprite_uri: &'static str,
    /// Gray out this symbol when matched at less than max bet.
    pub gray_at_low_bet: bool,
}

impl SlotSymbol {
    fn index(self) -> usize {
        match self {
            SlotSymbol::Low0 => 0,
            SlotSymbol::Low1 => 1,
            SlotSymbol::Low2 => 2,
            SlotSymbol::Mid0 => 3,
            SlotSymbol::Mid1 => 4,
            SlotSymbol::High0 => 5,
            SlotSymbol::ExtraRoll0 => 6,
        }
    }

    pub fn config(self) -> &'static SpriteConfig {
        &SYMBOLS[self.index()]
    }
}

pub fn symbol_sprite_uri(symbol: &SlotSymbol) -> &'static str {
    symbol.config().sprite_uri
}

pub fn symbol_display_name(symbol: &SlotSymbol) -> &'static str {
    symbol.config().display_name
}

const GOLD_COINS_URI: &str = concat!(
    "data:image/png;base64,",
    include_str!("gold_coins_base64.txt")
);

pub fn coin_icon_uri() -> &'static str {
    GOLD_COINS_URI
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_symbols_map_to_valid_uris() {
        let symbols = [
            SlotSymbol::Low0,
            SlotSymbol::Low1,
            SlotSymbol::Low2,
            SlotSymbol::Mid0,
            SlotSymbol::Mid1,
            SlotSymbol::High0,
            SlotSymbol::ExtraRoll0,
        ];

        for symbol in &symbols {
            let uri = symbol_sprite_uri(symbol);
            assert!(!uri.is_empty(), "Sprite URI for {:?} is empty", symbol);
            assert!(
                uri.starts_with("data:image/png;base64,"),
                "URI should be a PNG data URI: {:?}",
                symbol
            );
        }
    }

    #[test]
    fn display_names_are_correct() {
        assert_eq!(symbol_display_name(&SlotSymbol::Low0), "Heart");
        assert_eq!(symbol_display_name(&SlotSymbol::Low1), "Heart");
        assert_eq!(symbol_display_name(&SlotSymbol::Low2), "Heart");
        assert_eq!(symbol_display_name(&SlotSymbol::Mid0), "Skull");
        assert_eq!(symbol_display_name(&SlotSymbol::Mid1), "Skull");
        assert_eq!(symbol_display_name(&SlotSymbol::High0), "Chest");
        assert_eq!(symbol_display_name(&SlotSymbol::ExtraRoll0), "ExtraRoll");
    }

    #[test]
    fn coin_icon_returns_data_uri() {
        assert!(coin_icon_uri().starts_with("data:image/png;base64,"));
    }
}

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
/// Weights: Heart 17/14/11 (42, 30%), Skull 22/14 (36, 25.7%), Chest 28 (20%), ExtraRoll 34 (24.3%)
pub static SYMBOLS: [SpriteConfig; 7] = [
    SpriteConfig {
        display_name: "Heart",
        tier: RewardTier::Small,
        weight: 17.0,
        sprite_uri: LOW_HEART_URI,
    },
    SpriteConfig {
        display_name: "Heart",
        tier: RewardTier::Small,
        weight: 14.0,
        sprite_uri: LOW_HEART_URI,
    },
    SpriteConfig {
        display_name: "Heart",
        tier: RewardTier::Small,
        weight: 11.0,
        sprite_uri: LOW_HEART_URI,
    },
    SpriteConfig {
        display_name: "Skull",
        tier: RewardTier::Medium,
        weight: 22.0,
        sprite_uri: MED_SKULL_URI,
    },
    SpriteConfig {
        display_name: "Skull",
        tier: RewardTier::Medium,
        weight: 14.0,
        sprite_uri: MED_SKULL_URI,
    },
    SpriteConfig {
        display_name: "Chest",
        tier: RewardTier::Jackpot,
        weight: 28.0,
        sprite_uri: HIGH_CHEST_URI,
    },
    SpriteConfig {
        display_name: "ExtraRoll",
        tier: RewardTier::ExtraRoll,
        weight: 34.0,
        sprite_uri: EXTRA_ROLL_COINS_URI,
    },
];

/// Per-symbol configuration. Each array index maps to a SlotSymbol variant in order.
#[derive(Clone, Copy)]
pub struct SpriteConfig {
    pub display_name: &'static str,
    pub tier: RewardTier,
    pub weight: f64,
    pub sprite_uri: &'static str,
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

/// Check if three symbols share the same display name (i.e., form a 3-of-a-kind).
pub fn display_names_match(a: SlotSymbol, b: SlotSymbol, c: SlotSymbol) -> bool {
    let n = |s: SlotSymbol| symbol_display_name(&s);
    n(a) == n(b) && n(b) == n(c)
}

/// Return the border color hex for a winning row based on the matched symbol's tier.
pub fn winning_border_color(symbol: SlotSymbol) -> &'static str {
    match symbol.config().tier {
        RewardTier::Small => "#4ade80",
        RewardTier::Medium => "#c084fc",
        RewardTier::Jackpot => "#fb923c",
        RewardTier::ExtraRoll => "#facc15",
        RewardTier::None => "#00f5d4",
    }
}

const GOLD_COINS_URI: &str = concat!(
    "data:image/png;base64,",
    include_str!("gold_coins_base64.txt")
);

pub fn coin_icon_uri() -> &'static str {
    GOLD_COINS_URI
}

const BACK_ARROW_URI: &str = concat!(
    "data:image/png;base64,",
    include_str!("back_arrow_base64.txt")
);

pub fn back_arrow_uri() -> &'static str {
    BACK_ARROW_URI
}

const CHECK_GRAY_URI: &str = concat!(
    "data:image/png;base64,",
    include_str!("check_gray_base64.txt")
);

pub fn check_gray_uri() -> &'static str {
    CHECK_GRAY_URI
}

const CHECK_GREEN_URI: &str = concat!(
    "data:image/png;base64,",
    include_str!("check_green_base64.txt")
);

pub fn check_green_uri() -> &'static str {
    CHECK_GREEN_URI
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

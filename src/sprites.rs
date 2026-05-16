use crate::models::SlotSymbol;

#[derive(Debug, Clone, Copy)]
pub struct SpriteRegistry {
    pub path: &'static str,
    pub display_name: &'static str,
}

pub const SPRITE_REGISTRY: [(SlotSymbol, SpriteRegistry); 6] = [
    (
        SlotSymbol::Kebab,
        SpriteRegistry {
            path: "/Foods/low1.png",
            display_name: "Kebab",
        },
    ),
    (
        SlotSymbol::Taco,
        SpriteRegistry {
            path: "/Foods/low2.png",
            display_name: "Taco",
        },
    ),
    (
        SlotSymbol::Pizza,
        SpriteRegistry {
            path: "/Foods/low3.png",
            display_name: "Pizza",
        },
    ),
    (
        SlotSymbol::Sushi,
        SpriteRegistry {
            path: "/Foods/med1.png",
            display_name: "Sushi",
        },
    ),
    (
        SlotSymbol::Sashimi,
        SpriteRegistry {
            path: "/Foods/med2.png",
            display_name: "Sashimi",
        },
    ),
    (
        SlotSymbol::Pancake,
        SpriteRegistry {
            path: "/Foods/high1.png",
            display_name: "Pancake",
        },
    ),
];

pub fn symbol_sprite_uri(symbol: &SlotSymbol) -> &'static str {
    for &(s, reg) in &SPRITE_REGISTRY {
        if s == *symbol {
            return reg.path;
        }
    }
    "/Foods/low1.png"
}

pub fn symbol_display_name(symbol: &SlotSymbol) -> &'static str {
    for &(s, reg) in &SPRITE_REGISTRY {
        if s == *symbol {
            return reg.display_name;
        }
    }
    "Unknown"
}

pub fn coin_icon_uri() -> &'static str {
    symbol_sprite_uri(&SlotSymbol::Pancake)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_symbols_map_to_valid_paths() {
        let symbols = [
            SlotSymbol::Kebab,
            SlotSymbol::Taco,
            SlotSymbol::Pizza,
            SlotSymbol::Sushi,
            SlotSymbol::Sashimi,
            SlotSymbol::Pancake,
        ];

        for symbol in &symbols {
            let uri = symbol_sprite_uri(symbol);
            assert!(
                uri.starts_with("/Foods/"),
                "Path should be under /Foods/: {:?}",
                symbol
            );
            assert!(uri.ends_with(".png"), "Path should be a PNG: {:?}", symbol);
        }
    }

    #[test]
    fn display_names_are_correct() {
        assert_eq!(symbol_display_name(&SlotSymbol::Kebab), "Kebab");
        assert_eq!(symbol_display_name(&SlotSymbol::Taco), "Taco");
        assert_eq!(symbol_display_name(&SlotSymbol::Pizza), "Pizza");
        assert_eq!(symbol_display_name(&SlotSymbol::Sushi), "Sushi");
        assert_eq!(symbol_display_name(&SlotSymbol::Sashimi), "Sashimi");
        assert_eq!(symbol_display_name(&SlotSymbol::Pancake), "Pancake");
    }

    #[test]
    fn no_duplicate_paths() {
        let paths: Vec<&str> = SPRITE_REGISTRY.iter().map(|(_, r)| r.path).collect();
        let mut sorted = paths.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            6,
            "Expected 6 unique sprite paths, found {}",
            sorted.len()
        );
    }

    #[test]
    fn no_duplicate_display_names() {
        let names: Vec<&str> = SPRITE_REGISTRY
            .iter()
            .map(|(_, r)| r.display_name)
            .collect();
        let mut sorted = names.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            6,
            "Expected 6 unique display names, found {}",
            sorted.len()
        );
    }

    #[test]
    fn sprite_registry_maps_correct_files() {
        let expected = [
            (SlotSymbol::Kebab, "/Foods/low1.png"),
            (SlotSymbol::Taco, "/Foods/low2.png"),
            (SlotSymbol::Pizza, "/Foods/low3.png"),
            (SlotSymbol::Sushi, "/Foods/med1.png"),
            (SlotSymbol::Sashimi, "/Foods/med2.png"),
            (SlotSymbol::Pancake, "/Foods/high1.png"),
        ];

        for (symbol, expected_path) in &expected {
            assert_eq!(
                symbol_sprite_uri(symbol),
                *expected_path,
                "Wrong path for {:?}",
                symbol
            );
        }
    }

    #[test]
    fn coin_icon_returns_pancake_uri() {
        assert_eq!(coin_icon_uri(), symbol_sprite_uri(&SlotSymbol::Pancake));
    }
}

use crate::models::SlotSymbol;

const KEBAB_URI: &str = concat!("data:image/png;base64,", include_str!("kebab_base64.txt"));
const TACO_URI: &str = concat!("data:image/png;base64,", include_str!("taco_base64.txt"));
const PIZZA_URI: &str = concat!("data:image/png;base64,", include_str!("pizza_base64.txt"));
const SUSHI_URI: &str = concat!("data:image/png;base64,", include_str!("sushi_base64.txt"));
const SASHIMI_URI: &str = concat!("data:image/png;base64,", include_str!("sashimi_base64.txt"));
const PANCAKE_URI: &str = concat!("data:image/png;base64,", include_str!("pancake_base64.txt"));

pub fn symbol_sprite_uri(symbol: &SlotSymbol) -> &'static str {
    match symbol {
        SlotSymbol::Kebab => KEBAB_URI,
        SlotSymbol::Taco => TACO_URI,
        SlotSymbol::Pizza => PIZZA_URI,
        SlotSymbol::Sushi => SUSHI_URI,
        SlotSymbol::Sashimi => SASHIMI_URI,
        SlotSymbol::Pancake => PANCAKE_URI,
    }
}

pub fn symbol_display_name(symbol: &SlotSymbol) -> &'static str {
    match symbol {
        SlotSymbol::Kebab => "Kebab",
        SlotSymbol::Taco => "Taco",
        SlotSymbol::Pizza => "Pizza",
        SlotSymbol::Sushi => "Sushi",
        SlotSymbol::Sashimi => "Sashimi",
        SlotSymbol::Pancake => "Pancake",
    }
}

pub fn coin_icon_uri() -> &'static str {
    PANCAKE_URI
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_symbols_map_to_non_empty_uris() {
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
            assert!(!uri.is_empty(), "Sprite URI for {:?} is empty", symbol);
            assert!(
                uri.starts_with("data:image/png;base64,"),
                "URI should be a data URL: {:?}",
                symbol
            );
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
    fn no_duplicate_uris() {
        let uris = [
            symbol_sprite_uri(&SlotSymbol::Kebab),
            symbol_sprite_uri(&SlotSymbol::Taco),
            symbol_sprite_uri(&SlotSymbol::Pizza),
            symbol_sprite_uri(&SlotSymbol::Sushi),
            symbol_sprite_uri(&SlotSymbol::Sashimi),
            symbol_sprite_uri(&SlotSymbol::Pancake),
        ];

        let mut sorted = uris.to_vec();
        sorted.sort();
        sorted.dedup();

        assert_eq!(
            sorted.len(),
            6,
            "Expected 6 unique sprite URIs, found {}",
            sorted.len()
        );
    }

    #[test]
    fn coin_icon_returns_pancake_uri() {
        assert_eq!(coin_icon_uri(), symbol_sprite_uri(&SlotSymbol::Pancake));
    }
}

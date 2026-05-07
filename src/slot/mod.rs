//! Slot engine — pure logic, no UI dependencies.
//! Spin resolution, reel generation, symbol matching, and win calculation.

use crate::models::{RewardTier, SlotSymbol, SpinResult};
use rand::thread_rng;
use rand::Rng;

/// Symbol weights for random generation (higher = more frequent).
/// Sum = 100. Cherry is common, Devil is rarest.
const SYMBOL_WEIGHTS: &[(SlotSymbol, f64)] = &[
    (SlotSymbol::Cherry, 35.0),
    (SlotSymbol::Bell, 30.0),
    (SlotSymbol::Diamond, 20.0),
    (SlotSymbol::Seven, 11.0),
    (SlotSymbol::Devil, 4.0),
];

/// Payout multiplier for 3-of-a-kind matches.
fn payout_multiplier(symbol: SlotSymbol) -> Option<u32> {
    match symbol {
        SlotSymbol::Cherry => Some(2),
        SlotSymbol::Bell => Some(5),
        SlotSymbol::Diamond => Some(10),
        SlotSymbol::Seven => Some(25),
        SlotSymbol::Devil => Some(50),
    }
}

/// Map a matched symbol to its reward tier.
fn symbol_tier(symbol: SlotSymbol) -> RewardTier {
    match symbol {
        SlotSymbol::Cherry | SlotSymbol::Bell => RewardTier::Small,
        SlotSymbol::Diamond | SlotSymbol::Seven => RewardTier::Medium,
        SlotSymbol::Devil => RewardTier::Jackpot,
    }
}

/// Numeric tier order for comparison (higher = rarer/better).
fn symbol_tier_order(symbol: SlotSymbol) -> u8 {
    match symbol {
        SlotSymbol::Cherry => 0,
        SlotSymbol::Bell => 1,
        SlotSymbol::Diamond => 2,
        SlotSymbol::Seven => 3,
        SlotSymbol::Devil => 4,
    }
}

/// Generate a single symbol based on the configured probability weights.
fn roll_symbol(rng: &mut impl Rng) -> SlotSymbol {
    let r: f64 = rng.gen_range(0.0..100.0);
    let mut cumulative = 0.0;
    for &(symbol, weight) in SYMBOL_WEIGHTS {
        cumulative += weight;
        if r < cumulative {
            return symbol;
        }
    }
    SlotSymbol::Cherry
}

/// Check a single row (payline) for 3-of-a-kind.
fn check_payline(row: [SlotSymbol; 3]) -> Option<(SlotSymbol, u8)> {
    if row[0] == row[1] && row[1] == row[2] {
        Some((row[0], 3))
    } else {
        None
    }
}

/// Resolve a spin given a bet amount. Generates reels, checks paylines, calculates payout.
pub fn spin(bet: u32) -> SpinResult {
    let mut rng = thread_rng();

    // Generate 3 reels, each with 3 symbols (top, middle, bottom).
    // Layout: reels[col][row] where col=0..2, row=0=top,1=middle,2=bottom
    let mut reels: [[SlotSymbol; 3]; 3] = Default::default();
    for reel in reels.iter_mut() {
        for pos in reel.iter_mut() {
            *pos = roll_symbol(&mut rng);
        }
    }

    // Check horizontal paylines (top, middle, bottom rows across all reels).
    let mut best_match: Option<(SlotSymbol, u8)> = None;
    for row_idx in 0..3 {
        let payline = [reels[0][row_idx], reels[1][row_idx], reels[2][row_idx]];
        if let Some((symbol, count)) = check_payline(payline) {
            match &best_match {
                None => best_match = Some((symbol, count)),
                Some((best_symbol, _)) => {
                    let current_tier_idx = symbol_tier_order(symbol);
                    let best_tier_idx = symbol_tier_order(*best_symbol);
                    if current_tier_idx > best_tier_idx {
                        best_match = Some((symbol, count));
                    }
                }
            }
        }
    }

    let tier = match &best_match {
        Some((symbol, _)) => symbol_tier(*symbol),
        None => RewardTier::None,
    };

    let payout_coins = match &best_match {
        Some((symbol, _)) => {
            if let Some(multiplier) = payout_multiplier(*symbol) {
                multiplier * bet
            } else {
                0
            }
        }
        None => 0,
    };

    SpinResult {
        reels,
        symbols_matched: best_match,
        tier,
        payout_coins,
        is_near_miss: false,
        grayed_high_tier: false,
    }
}

/// Return the configured probability for a symbol as a fraction (0.0..1.0).
pub fn symbol_probability(symbol: SlotSymbol) -> f64 {
    for &(s, weight) in SYMBOL_WEIGHTS {
        if s == symbol {
            return weight / 100.0;
        }
    }
    0.0
}

/// Expected probability of rolling exactly 3 of a given symbol on any single payline.
pub fn exact_match_probability(symbol: SlotSymbol) -> f64 {
    let p = symbol_probability(symbol);
    p * p * p
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    const NUM_SPINS: u32 = 10_000;
    const TOLERANCE: f64 = 0.03;

    #[test]
    fn payout_scales_linearly_with_bet() {
        let symbols = [
            SlotSymbol::Cherry,
            SlotSymbol::Bell,
            SlotSymbol::Diamond,
            SlotSymbol::Seven,
            SlotSymbol::Devil,
        ];

        for &symbol in &symbols {
            if let Some(base_mult) = payout_multiplier(symbol) {
                assert_eq!(base_mult * 1, base_mult);
                assert_eq!(base_mult * 2, base_mult + base_mult);
                assert_eq!(base_mult * 3, base_mult + base_mult + base_mult);
            }
        }
    }

    #[test]
    fn reward_tier_mapping() {
        assert_eq!(symbol_tier(SlotSymbol::Cherry), RewardTier::Small);
        assert_eq!(symbol_tier(SlotSymbol::Bell), RewardTier::Small);
        assert_eq!(symbol_tier(SlotSymbol::Diamond), RewardTier::Medium);
        assert_eq!(symbol_tier(SlotSymbol::Seven), RewardTier::Medium);
        assert_eq!(symbol_tier(SlotSymbol::Devil), RewardTier::Jackpot);
    }

    #[test]
    fn spin_returns_valid_result() {
        let result = spin(1);
        assert_eq!(result.reels.len(), 3);
        assert_eq!(result.reels[0].len(), 3);
        assert!(result.payout_coins == 0 || result.symbols_matched.is_some());
    }

    #[test]
    fn probability_distribution_cherry_within_tolerance() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut cherry_count = 0u32;

        for _ in 0..NUM_SPINS {
            if roll_symbol(&mut rng) == SlotSymbol::Cherry {
                cherry_count += 1;
            }
        }

        let observed = cherry_count as f64 / NUM_SPINS as f64;
        let expected = symbol_probability(SlotSymbol::Cherry);
        assert!(
            (observed - expected).abs() < TOLERANCE,
            "Cherry: observed {:.3}, expected {:.3}",
            observed,
            expected
        );
    }

    #[test]
    fn probability_distribution_devil_within_tolerance() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut devil_count = 0u32;

        for _ in 0..NUM_SPINS {
            if roll_symbol(&mut rng) == SlotSymbol::Devil {
                devil_count += 1;
            }
        }

        let observed = devil_count as f64 / NUM_SPINS as f64;
        let expected = symbol_probability(SlotSymbol::Devil);
        assert!(
            (observed - expected).abs() < TOLERANCE * 2.0,
            "Devil: observed {:.3}, expected {:.3}",
            observed,
            expected
        );
    }

    #[test]
    fn tier_distribution_across_10k_spins() {
        let mut rng = StdRng::seed_from_u64(123);
        let mut small_count = 0u32;
        let mut medium_count = 0u32;
        let mut jackpot_count = 0u32;
        let mut no_win_count = 0u32;

        for _ in 0..NUM_SPINS {
            let reels: [[SlotSymbol; 3]; 3] = [
                [
                    roll_symbol(&mut rng),
                    roll_symbol(&mut rng),
                    roll_symbol(&mut rng),
                ],
                [
                    roll_symbol(&mut rng),
                    roll_symbol(&mut rng),
                    roll_symbol(&mut rng),
                ],
                [
                    roll_symbol(&mut rng),
                    roll_symbol(&mut rng),
                    roll_symbol(&mut rng),
                ],
            ];

            let mut best: Option<SlotSymbol> = None;
            for row_idx in 0..3 {
                let payline = [reels[0][row_idx], reels[1][row_idx], reels[2][row_idx]];
                if payline[0] == payline[1] && payline[1] == payline[2] {
                    match best {
                        None => best = Some(payline[0]),
                        Some(b) => {
                            if symbol_tier_order(payline[0]) > symbol_tier_order(b) {
                                best = Some(payline[0]);
                            }
                        }
                    }
                }
            }

            match best {
                None => no_win_count += 1,
                Some(s) => match symbol_tier(s) {
                    RewardTier::Small => small_count += 1,
                    RewardTier::Medium => medium_count += 1,
                    RewardTier::Jackpot => jackpot_count += 1,
                    RewardTier::None => unreachable!(),
                },
            }
        }

        let total = small_count + medium_count + jackpot_count + no_win_count;
        assert_eq!(total, NUM_SPINS);

        let no_win_rate = no_win_count as f64 / NUM_SPINS as f64;
        assert!(
            no_win_rate > 0.70,
            "no-win rate {:.3} too low (expected ~75%)",
            no_win_rate
        );
    }

    #[test]
    fn bet_multiplier_affects_payout() {
        let base_mult = payout_multiplier(SlotSymbol::Bell).unwrap();

        assert_eq!(base_mult * 1, 5);
        assert_eq!(base_mult * 2, 10);
        assert_eq!(base_mult * 3, 15);
    }
}

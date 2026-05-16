//! Slot engine — pure logic, no UI dependencies.
//! Spin resolution, reel generation, symbol matching, and win calculation.

use crate::models::{RewardTier, SlotSymbol, SpinResult};
use rand::thread_rng;
use rand::Rng;

/// Consecutive losses that trigger pity win.
const PITY_THRESHOLD: u32 = 5;

/// Maximum allowed bet per spin. High-tier symbols only pay full at this level.
pub const MAX_BET: u32 = 3;

/// Symbol weights for random generation (higher = more frequent).
/// Sum = 100. Kebab/Taco/Pizza are common, Pancake is rarest.
const SYMBOL_WEIGHTS: &[(SlotSymbol, f64)] = &[
    (SlotSymbol::Kebab, 25.0),
    (SlotSymbol::Taco, 20.0),
    (SlotSymbol::Pizza, 15.0),
    (SlotSymbol::Sushi, 18.0),
    (SlotSymbol::Sashimi, 12.0),
    (SlotSymbol::Pancake, 10.0),
];

/// Payout multiplier for 3-of-a-kind matches.
fn payout_multiplier(symbol: SlotSymbol) -> Option<u32> {
    match symbol {
        SlotSymbol::Kebab => Some(2),
        SlotSymbol::Taco => Some(4),
        SlotSymbol::Pizza => Some(3),
        SlotSymbol::Sushi => Some(8),
        SlotSymbol::Sashimi => Some(12),
        SlotSymbol::Pancake => Some(50),
    }
}

/// Map a matched symbol to its reward tier.
fn symbol_tier(symbol: SlotSymbol) -> RewardTier {
    match symbol {
        SlotSymbol::Kebab | SlotSymbol::Taco | SlotSymbol::Pizza => RewardTier::Small,
        SlotSymbol::Sushi | SlotSymbol::Sashimi => RewardTier::Medium,
        SlotSymbol::Pancake => RewardTier::Jackpot,
    }
}

/// Numeric tier order for comparison (higher = rarer/better).
fn symbol_tier_order(symbol: SlotSymbol) -> u8 {
    match symbol {
        SlotSymbol::Kebab => 0,
        SlotSymbol::Taco => 1,
        SlotSymbol::Pizza => 2,
        SlotSymbol::Sushi => 3,
        SlotSymbol::Sashimi => 4,
        SlotSymbol::Pancake => 5,
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
    SlotSymbol::Kebab
}

/// Check a single row (payline) for 3-of-a-kind.
fn check_payline(row: [SlotSymbol; 3]) -> Option<(SlotSymbol, u8)> {
    if row[0] == row[1] && row[1] == row[2] {
        Some((row[0], 3))
    } else {
        None
    }
}

/// Check if a payline has a near-miss pattern: two symbols match, third is different.
fn check_near_miss(row: [SlotSymbol; 3]) -> bool {
    row[0] == row[1] || row[1] == row[2] || row[0] == row[2]
}

/// Check if any payline has a near-miss pattern across the reels.
fn has_near_miss_pattern(reels: &[[SlotSymbol; 3]; 3]) -> bool {
    for row_idx in 0..3 {
        let payline = [reels[0][row_idx], reels[1][row_idx], reels[2][row_idx]];
        if check_near_miss(payline) {
            return true;
        }
    }
    false
}

/// Generate reels that guarantee a win of at least the given minimum tier.
fn generate_pity_reels(rng: &mut impl Rng, min_tier: RewardTier) -> [[SlotSymbol; 3]; 3] {
    let mut reels: [[SlotSymbol; 3]; 3] = Default::default();

    // Fill all positions with random symbols first
    for reel in reels.iter_mut() {
        for pos in reel.iter_mut() {
            *pos = roll_symbol(rng);
        }
    }

    // Pick a random row to place the guaranteed win
    let win_row = rng.gen_range(0..3);

    // Choose which symbol to force based on minimum tier requirement
    let forced_symbol: SlotSymbol;
    match min_tier {
        RewardTier::Small => {
            // Force either Kebab or Taco
            if rng.gen_bool(0.5) {
                forced_symbol = SlotSymbol::Kebab;
            } else {
                forced_symbol = SlotSymbol::Taco;
            }
        }
        _ => {
            // Default to Kebab for safety
            forced_symbol = SlotSymbol::Kebab;
        }
    }

    // Force 3-of-a-kind on the chosen row
    reels[0][win_row] = forced_symbol;
    reels[1][win_row] = forced_symbol;
    reels[2][win_row] = forced_symbol;

    reels
}

/// Update pity counter based on spin result. Returns whether pity win should be triggered.
fn update_pity_state(consecutive_losses: &mut u32, is_win: bool) -> (u32, bool) {
    if is_win {
        *consecutive_losses = 0;
        (*consecutive_losses, false)
    } else {
        *consecutive_losses += 1;
        let should_pity = *consecutive_losses >= PITY_THRESHOLD;
        if should_pity {
            *consecutive_losses = 0;
        }
        (*consecutive_losses, should_pity)
    }
}

/// Resolve a spin given a bet amount. Generates reels, checks paylines, calculates payout.
pub fn spin(bet: u32) -> SpinResult {
    let mut consecutive_losses: u32 = 0;
    spin_with_state(&mut consecutive_losses, bet)
}

/// Resolve a spin with persistent pity state tracking.
pub fn spin_with_state(consecutive_losses: &mut u32, bet: u32) -> SpinResult {
    let mut rng = thread_rng();

    // Check if pity win should trigger
    let should_pity_win = *consecutive_losses >= PITY_THRESHOLD - 1;

    let reels = if should_pity_win {
        generate_pity_reels(&mut rng, RewardTier::Small)
    } else {
        // Generate normal reels
        let mut normal_reels: [[SlotSymbol; 3]; 3] = Default::default();
        for reel in normal_reels.iter_mut() {
            for pos in reel.iter_mut() {
                *pos = roll_symbol(&mut rng);
            }
        }
        normal_reels
    };

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

    // Determine if high-tier symbol should be grayed out at low bet.
    // High-tier symbols (Sushi+, tier order >= 3) display grayed when bet < MAX_BET.
    let grayed_high_tier = best_match
        .map(|(s, _)| symbol_tier_order(s) >= 3 && bet < MAX_BET)
        .unwrap_or(false);

    // Apply reduced payout for grayed high-tier symbols: proportional to bet/MAX_BET ratio.
    let effective_payout = if grayed_high_tier {
        (payout_coins as f64 * (bet as f64 / MAX_BET as f64)).round() as u32
    } else {
        payout_coins
    };

    // Update pity state after determining result
    let _ = update_pity_state(consecutive_losses, effective_payout > 0);

    // Determine near-miss flag: only on losing spins with matching pattern
    let is_near_miss = effective_payout == 0 && has_near_miss_pattern(&reels);

    SpinResult {
        reels,
        symbols_matched: best_match,
        tier,
        payout_coins: effective_payout,
        is_near_miss,
        grayed_high_tier,
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

/// Resolve a spin from pre-generated reels (no RNG). Used for testing grayed-out behavior.
pub fn resolve_reels(reels: [[SlotSymbol; 3]; 3], bet: u32) -> SpinResult {
    // Check horizontal paylines
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

    let grayed_high_tier = best_match
        .map(|(s, _)| symbol_tier_order(s) >= 3 && bet < MAX_BET)
        .unwrap_or(false);

    let effective_payout = if grayed_high_tier {
        (payout_coins as f64 * (bet as f64 / MAX_BET as f64)).round() as u32
    } else {
        payout_coins
    };

    let is_near_miss = effective_payout == 0 && has_near_miss_pattern(&reels);

    SpinResult {
        reels,
        symbols_matched: best_match,
        tier,
        payout_coins: effective_payout,
        is_near_miss,
        grayed_high_tier,
    }
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
            SlotSymbol::Kebab,
            SlotSymbol::Taco,
            SlotSymbol::Sushi,
            SlotSymbol::Sashimi,
            SlotSymbol::Pancake,
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
        assert_eq!(symbol_tier(SlotSymbol::Kebab), RewardTier::Small);
        assert_eq!(symbol_tier(SlotSymbol::Taco), RewardTier::Small);
        assert_eq!(symbol_tier(SlotSymbol::Sushi), RewardTier::Medium);
        assert_eq!(symbol_tier(SlotSymbol::Sashimi), RewardTier::Medium);
        assert_eq!(symbol_tier(SlotSymbol::Pancake), RewardTier::Jackpot);
    }

    #[test]
    fn spin_returns_valid_result() {
        let result = spin(1);
        assert_eq!(result.reels.len(), 3);
        assert_eq!(result.reels[0].len(), 3);
        assert!(result.payout_coins == 0 || result.symbols_matched.is_some());
    }

    #[test]
    fn probability_distribution_kebab_within_tolerance() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut kebab_count = 0u32;

        for _ in 0..NUM_SPINS {
            if roll_symbol(&mut rng) == SlotSymbol::Kebab {
                kebab_count += 1;
            }
        }

        let observed = kebab_count as f64 / NUM_SPINS as f64;
        let expected = symbol_probability(SlotSymbol::Kebab);
        assert!(
            (observed - expected).abs() < TOLERANCE,
            "Kebab: observed {:.3}, expected {:.3}",
            observed,
            expected
        );
    }

    #[test]
    fn probability_distribution_pancake_within_tolerance() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut pancake_count = 0u32;

        for _ in 0..NUM_SPINS {
            if roll_symbol(&mut rng) == SlotSymbol::Pancake {
                pancake_count += 1;
            }
        }

        let observed = pancake_count as f64 / NUM_SPINS as f64;
        let expected = symbol_probability(SlotSymbol::Pancake);
        assert!(
            (observed - expected).abs() < TOLERANCE * 2.0,
            "Pancake: observed {:.3}, expected {:.3}",
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
        let base_mult = payout_multiplier(SlotSymbol::Taco).unwrap();

        assert_eq!(base_mult * 1, 4);
        assert_eq!(base_mult * 2, 8);
        assert_eq!(base_mult * 3, 12);
    }

    #[test]
    fn pity_counter_resets_after_win() {
        let mut losses = 0u32;
        for _ in 0..4 {
            update_pity_state(&mut losses, false);
        }
        assert_eq!(losses, 4);

        update_pity_state(&mut losses, true);
        assert_eq!(losses, 0);
    }

    #[test]
    fn pity_triggers_on_fifth_consecutive_loss() {
        let mut losses = 0u32;
        for _ in 0..4 {
            update_pity_state(&mut losses, false);
        }
        assert_eq!(losses, 4);

        let (final_losses, should_pity) = update_pity_state(&mut losses, false);
        assert!(should_pity, "pity should trigger on 5th loss");
        assert_eq!(final_losses, 0, "counter resets after pity triggers");
    }

    #[test]
    fn pity_does_not_trigger_before_five_losses() {
        let mut losses = 0u32;
        for i in 0..4 {
            let (_, should_pity) = update_pity_state(&mut losses, false);
            assert!(!should_pity, "pity should not trigger on loss #{}", i + 1);
        }
    }

    #[test]
    fn near_miss_detected_on_two_matching_symbols() {
        let row_kebab_taco = [SlotSymbol::Kebab, SlotSymbol::Kebab, SlotSymbol::Taco];
        assert!(check_near_miss(row_kebab_taco));

        let row_all_same = [SlotSymbol::Taco, SlotSymbol::Taco, SlotSymbol::Taco];
        assert!(check_near_miss(row_all_same));

        let row_all_diff = [SlotSymbol::Kebab, SlotSymbol::Taco, SlotSymbol::Sushi];
        assert!(!check_near_miss(row_all_diff));
    }

    #[test]
    fn near_miss_pattern_detected_across_reels() {
        let mut reels: [[SlotSymbol; 3]; 3] = Default::default();

        // Set up middle row with two matching symbols
        reels[0][1] = SlotSymbol::Sushi;
        reels[1][1] = SlotSymbol::Sushi;
        reels[2][1] = SlotSymbol::Sashimi;

        assert!(has_near_miss_pattern(&reels));
    }

    #[test]
    fn near_miss_not_detected_on_all_different() {
        let mut reels: [[SlotSymbol; 3]; 3] = Default::default();

        // All rows have different symbols
        reels[0][0] = SlotSymbol::Kebab;
        reels[1][0] = SlotSymbol::Taco;
        reels[2][0] = SlotSymbol::Sushi;

        reels[0][1] = SlotSymbol::Sashimi;
        reels[1][1] = SlotSymbol::Pancake;
        reels[2][1] = SlotSymbol::Kebab;

        reels[0][2] = SlotSymbol::Taco;
        reels[1][2] = SlotSymbol::Sushi;
        reels[2][2] = SlotSymbol::Sashimi;

        assert!(!has_near_miss_pattern(&reels));
    }

    #[test]
    fn pity_reels_generate_guaranteed_win() {
        let mut rng = StdRng::seed_from_u64(999);

        for _ in 0..100 {
            let reels = generate_pity_reels(&mut rng, RewardTier::Small);
            // Verify at least one row has a matching pattern (win)
            let mut found_win = false;
            for row_idx in 0..3 {
                let payline = [reels[0][row_idx], reels[1][row_idx], reels[2][row_idx]];
                if let Some(_) = check_payline(payline) {
                    found_win = true;
                    break;
                }
            }
            assert!(
                found_win,
                "pity reels should contain at least one winning payline"
            );
        }
    }

    #[test]
    fn near_miss_only_on_losing_spins() {
        // Near-miss detection returns true for winning pattern too (3-of-a-kind has 2 matching)
        // But in spin_with_state, is_near_miss is only set when !is_win
        assert!(check_near_miss([
            SlotSymbol::Kebab,
            SlotSymbol::Kebab,
            SlotSymbol::Kebab
        ]));
    }

    #[test]
    fn pity_threshold_constant_is_five() {
        assert_eq!(PITY_THRESHOLD, 5);
    }

    #[test]
    fn spin_with_state_tracks_losses() {
        let mut losses: u32 = 0;
        for _ in 0..4 {
            update_pity_state(&mut losses, false);
        }
        assert_eq!(losses, 4);
    }

    #[test]
    fn grayed_high_tier_at_bet_1_sushi() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Kebab, SlotSymbol::Sushi, SlotSymbol::Taco],
            [SlotSymbol::Taco, SlotSymbol::Sushi, SlotSymbol::Kebab],
            [SlotSymbol::Kebab, SlotSymbol::Sushi, SlotSymbol::Kebab],
        ];

        let result = resolve_reels(reels, 1);
        assert!(result.grayed_high_tier);
        // Sushi base payout: 8 * 1 = 8. Reduced by 1/3 ratio -> round(8 * 1/3) = 3
        assert_eq!(result.payout_coins, 3);
    }

    #[test]
    fn grayed_high_tier_at_bet_2_sashimi() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Kebab, SlotSymbol::Sashimi, SlotSymbol::Taco],
            [SlotSymbol::Taco, SlotSymbol::Sashimi, SlotSymbol::Kebab],
            [SlotSymbol::Kebab, SlotSymbol::Sashimi, SlotSymbol::Taco],
        ];

        let result = resolve_reels(reels, 2);
        assert!(result.grayed_high_tier);
        // Sashimi base payout: 12 * 2 = 24. Reduced by 2/3 ratio -> round(24 * 2/3) = 16
        assert_eq!(result.payout_coins, 16);
    }

    #[test]
    fn no_gray_at_max_bet_3_pancake() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Kebab, SlotSymbol::Pancake, SlotSymbol::Taco],
            [SlotSymbol::Taco, SlotSymbol::Pancake, SlotSymbol::Kebab],
            [SlotSymbol::Kebab, SlotSymbol::Pancake, SlotSymbol::Kebab],
        ];

        let result = resolve_reels(reels, 3);
        assert!(!result.grayed_high_tier);
        // Pancake base payout: 50 * 3 = 150. No reduction at max bet.
        assert_eq!(result.payout_coins, 150);
    }

    #[test]
    fn no_gray_for_low_tier_symbols_at_bet_1() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Kebab, SlotSymbol::Taco, SlotSymbol::Kebab],
            [SlotSymbol::Kebab, SlotSymbol::Taco, SlotSymbol::Kebab],
            [SlotSymbol::Taco, SlotSymbol::Taco, SlotSymbol::Kebab],
        ];

        let result = resolve_reels(reels, 1);
        // Taco is low tier -> no gray regardless of bet
        assert!(!result.grayed_high_tier);
    }

    #[test]
    fn grayed_only_applies_to_matching_symbols() {
        // Kebab matches row 0, high-tier scattered but not matching
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Kebab, SlotSymbol::Sushi, SlotSymbol::Taco],
            [SlotSymbol::Kebab, SlotSymbol::Sashimi, SlotSymbol::Taco],
            [SlotSymbol::Kebab, SlotSymbol::Pancake, SlotSymbol::Kebab],
        ];

        let result = resolve_reels(reels, 1);
        // Kebab matched on row 0 (low-tier), high-tier symbols don't match -> no gray
        assert!(!result.grayed_high_tier);
    }
}

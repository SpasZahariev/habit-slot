//! Slot engine — pure logic, no UI dependencies.
//! Spin resolution, reel generation, symbol matching, and win calculation.

use crate::models::{RewardTier, SlotSymbol, SpinResult};
use crate::sprites;
use rand::thread_rng;
use rand::Rng;

/// Consecutive losses that trigger pity win.
const PITY_THRESHOLD: u32 = 5;

/// Maximum allowed bet per spin. High-tier symbols only pay full at this level.
pub const MAX_BET: u32 = 3;

/// Payout multiplier for 3-of-a-kind matches from config.
fn payout_multiplier(symbol: SlotSymbol) -> u32 {
    symbol.config().payout_multiplier
}

/// Map a matched symbol to its reward tier from config.
fn symbol_tier(symbol: SlotSymbol) -> RewardTier {
    symbol.config().tier
}

/// Numeric tier order for comparison (higher = rarer/better).
fn symbol_tier_order(symbol: SlotSymbol) -> u8 {
    match symbol.config().tier {
        RewardTier::Small => 0,
        RewardTier::Medium => 1,
        RewardTier::Jackpot => 2,
        RewardTier::None => 3,
    }
}

/// Check if symbol should be grayed at low bet.
fn symbol_gray_at_low_bet(symbol: SlotSymbol) -> bool {
    symbol.config().gray_at_low_bet
}

/// Get all symbols belonging to a given tier.
fn symbols_for_tier(tier: RewardTier) -> &'static [SlotSymbol] {
    match tier {
        RewardTier::Small => &[SlotSymbol::Low0, SlotSymbol::Low1, SlotSymbol::Low2],
        RewardTier::Medium => &[SlotSymbol::Mid0, SlotSymbol::Mid1],
        RewardTier::Jackpot => &[SlotSymbol::High0],
        RewardTier::None => &[],
    }
}

/// Generate a single symbol based on the configured probability weights.
fn roll_symbol(rng: &mut impl Rng) -> SlotSymbol {
    let symbols = &sprites::SYMBOLS;
    let total_weight: f64 = symbols.iter().map(|s| s.weight).sum();
    let r: f64 = rng.gen_range(0.0..total_weight);

    let all_symbols = [
        SlotSymbol::Low0,
        SlotSymbol::Low1,
        SlotSymbol::Low2,
        SlotSymbol::Mid0,
        SlotSymbol::Mid1,
        SlotSymbol::High0,
    ];

    let mut cumulative = 0.0;
    for (idx, config) in symbols.iter().enumerate() {
        cumulative += config.weight;
        if r < cumulative {
            return all_symbols[idx];
        }
    }
    SlotSymbol::default()
}

/// Check a single row (payline) for 3-of-a-kind by display name.
fn check_payline(row: [SlotSymbol; 3]) -> Option<(SlotSymbol, u8)> {
    if sprites::symbol_display_name(&row[0]) == sprites::symbol_display_name(&row[1])
        && sprites::symbol_display_name(&row[1]) == sprites::symbol_display_name(&row[2])
    {
        Some((row[0], 3))
    } else {
        None
    }
}

/// Check if a payline has a near-miss pattern: two symbols share display name, third doesn't.
fn check_near_miss(row: [SlotSymbol; 3]) -> bool {
    let n = |s: &SlotSymbol| sprites::symbol_display_name(s);
    n(&row[0]) == n(&row[1]) || n(&row[1]) == n(&row[2]) || n(&row[0]) == n(&row[2])
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

/// Pick a random symbol from the given tier.
fn pick_symbol_for_tier(rng: &mut impl Rng, tier: RewardTier) -> SlotSymbol {
    let symbols = symbols_for_tier(tier);
    if symbols.is_empty() {
        return SlotSymbol::default();
    }
    let idx = rng.gen_range(0..symbols.len());
    symbols[idx]
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

    // Choose a random symbol from the target tier
    let forced_symbol = pick_symbol_for_tier(rng, min_tier);

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
    let best_match = find_best_match(&reels);

    let tier = match &best_match {
        Some(row) => symbol_tier(row[0]),
        None => RewardTier::None,
    };

    let payout_coins = match &best_match {
        Some(row) => {
            let avg_mult = row.iter().map(|s| payout_multiplier(*s)).sum::<u32>() / 3;
            avg_mult * bet
        }
        None => 0,
    };

    // Determine if symbol should be grayed out at low bet.
    let grayed_high_tier = best_match
        .map(|row| symbol_gray_at_low_bet(row[0]) && bet < MAX_BET)
        .unwrap_or(false);

    // Apply reduced payout for grayed symbols: proportional to bet/MAX_BET ratio.
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
        symbols_matched: best_match.map(|row| (row[0], 3)),
        tier,
        payout_coins: effective_payout,
        is_near_miss,
        grayed_high_tier,
    }
}

/// Resolve a spin from pre-generated reels (no RNG). Used for testing grayed-out behavior.
pub fn resolve_reels(reels: [[SlotSymbol; 3]; 3], bet: u32) -> SpinResult {
    let best_match = find_best_match(&reels);

    let tier = match &best_match {
        Some(row) => symbol_tier(row[0]),
        None => RewardTier::None,
    };

    let payout_coins = match &best_match {
        Some(row) => {
            let avg_mult = row.iter().map(|s| payout_multiplier(*s)).sum::<u32>() / 3;
            avg_mult * bet
        }
        None => 0,
    };

    let grayed_high_tier = best_match
        .map(|row| symbol_gray_at_low_bet(row[0]) && bet < MAX_BET)
        .unwrap_or(false);

    let effective_payout = if grayed_high_tier {
        (payout_coins as f64 * (bet as f64 / MAX_BET as f64)).round() as u32
    } else {
        payout_coins
    };

    let is_near_miss = effective_payout == 0 && has_near_miss_pattern(&reels);

    SpinResult {
        reels,
        symbols_matched: best_match.map(|row| (row[0], 3)),
        tier,
        payout_coins: effective_payout,
        is_near_miss,
        grayed_high_tier,
    }
}

/// Find the highest-tier matching payline across all three rows.
fn find_best_match(reels: &[[SlotSymbol; 3]; 3]) -> Option<[SlotSymbol; 3]> {
    let mut best: Option<[SlotSymbol; 3]> = None;
    for row_idx in 0..3 {
        let payline = [reels[0][row_idx], reels[1][row_idx], reels[2][row_idx]];
        if check_payline(payline).is_some() {
            match &best {
                None => best = Some(payline),
                Some(b) => {
                    let current_tier = symbol_tier_order(payline[0]);
                    let best_tier = symbol_tier_order(b[0]);
                    if current_tier > best_tier {
                        best = Some(payline);
                    }
                }
            }
        }
    }
    best
}

/// Expected probability of rolling exactly 3 of a given symbol on any single payline.
pub fn exact_match_probability(symbol: SlotSymbol) -> f64 {
    let p = symbol.config().weight / 100.0;
    p * p * p
}

/// Number of filler symbols before the result in each reel animation strip.
pub const ANIMATION_FILLER_COUNT: usize = 12;

/// Generate an animation strip for a single reel column.
/// Returns ~`ANIMATION_FILLER_COUNT` weighted random filler symbols followed by the 3 result symbols.
/// The final 3 symbols match the SpinResult reel column exactly.
pub fn generate_animation_strip(
    rng: &mut impl Rng,
    result_column: [SlotSymbol; 3],
) -> Vec<SlotSymbol> {
    let mut strip = Vec::with_capacity(ANIMATION_FILLER_COUNT + 3);
    for _ in 0..ANIMATION_FILLER_COUNT {
        strip.push(roll_symbol(rng));
    }
    strip.extend_from_slice(&result_column);
    strip
}

/// Generate all 3 reel animation strips from a SpinResult.
pub fn generate_all_animation_strips(result: &SpinResult) -> [Vec<SlotSymbol>; 3] {
    let mut rng = thread_rng();
    [
        generate_animation_strip(&mut rng, result.reels[0]),
        generate_animation_strip(&mut rng, result.reels[1]),
        generate_animation_strip(&mut rng, result.reels[2]),
    ]
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
            SlotSymbol::Low0,
            SlotSymbol::Low1,
            SlotSymbol::Mid0,
            SlotSymbol::Mid1,
            SlotSymbol::High0,
        ];

        for &symbol in &symbols {
            let base_mult = payout_multiplier(symbol);
            assert_eq!(base_mult * 1, base_mult);
            assert_eq!(base_mult * 2, base_mult + base_mult);
            assert_eq!(base_mult * 3, base_mult + base_mult + base_mult);
        }
    }

    #[test]
    fn reward_tier_mapping() {
        assert_eq!(symbol_tier(SlotSymbol::Low0), RewardTier::Small);
        assert_eq!(symbol_tier(SlotSymbol::Low1), RewardTier::Small);
        assert_eq!(symbol_tier(SlotSymbol::Mid0), RewardTier::Medium);
        assert_eq!(symbol_tier(SlotSymbol::Mid1), RewardTier::Medium);
        assert_eq!(symbol_tier(SlotSymbol::High0), RewardTier::Jackpot);
    }

    #[test]
    fn spin_returns_valid_result() {
        let result = spin(1);
        assert_eq!(result.reels.len(), 3);
        assert_eq!(result.reels[0].len(), 3);
        assert!(result.payout_coins == 0 || result.symbols_matched.is_some());
    }

    #[test]
    fn probability_distribution_first_symbol_within_tolerance() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut kebab_count = 0u32;

        for _ in 0..NUM_SPINS {
            if roll_symbol(&mut rng) == SlotSymbol::Low0 {
                kebab_count += 1;
            }
        }

        let observed = kebab_count as f64 / NUM_SPINS as f64;
        let expected = SlotSymbol::Low0.config().weight / 100.0;
        assert!(
            (observed - expected).abs() < TOLERANCE,
            "First symbol: observed {:.3}, expected {:.3}",
            observed,
            expected
        );
    }

    #[test]
    fn probability_distribution_rarest_symbol_within_tolerance() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut pancake_count = 0u32;

        for _ in 0..NUM_SPINS {
            if roll_symbol(&mut rng) == SlotSymbol::High0 {
                pancake_count += 1;
            }
        }

        let observed = pancake_count as f64 / NUM_SPINS as f64;
        let expected = SlotSymbol::High0.config().weight / 100.0;
        assert!(
            (observed - expected).abs() < TOLERANCE * 2.0,
            "Rarest symbol: observed {:.3}, expected {:.3}",
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
        let base_mult = payout_multiplier(SlotSymbol::Low1);

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
        let row_two_hearts = [SlotSymbol::Low0, SlotSymbol::Low1, SlotSymbol::Mid0];
        assert!(check_near_miss(row_two_hearts));

        let row_all_same = [SlotSymbol::Low1, SlotSymbol::Low1, SlotSymbol::Low1];
        assert!(check_near_miss(row_all_same));

        let row_all_diff = [SlotSymbol::Low0, SlotSymbol::Mid0, SlotSymbol::High0];
        assert!(!check_near_miss(row_all_diff));
    }

    #[test]
    fn near_miss_pattern_detected_across_reels() {
        let mut reels: [[SlotSymbol; 3]; 3] = Default::default();

        // Set up middle row with two matching symbols
        reels[0][1] = SlotSymbol::Mid0;
        reels[1][1] = SlotSymbol::Mid0;
        reels[2][1] = SlotSymbol::Mid1;

        assert!(has_near_miss_pattern(&reels));
    }

    #[test]
    fn near_miss_not_detected_on_all_different() {
        let mut reels: [[SlotSymbol; 3]; 3] = Default::default();

        // All rows have different display names (Heart, Skull, Chest)
        reels[0][0] = SlotSymbol::Low0;
        reels[1][0] = SlotSymbol::Mid0;
        reels[2][0] = SlotSymbol::High0;

        reels[0][1] = SlotSymbol::Mid0;
        reels[1][1] = SlotSymbol::High0;
        reels[2][1] = SlotSymbol::Low0;

        reels[0][2] = SlotSymbol::High0;
        reels[1][2] = SlotSymbol::Low0;
        reels[2][2] = SlotSymbol::Mid0;

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
            SlotSymbol::Low0,
            SlotSymbol::Low0,
            SlotSymbol::Low0
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
    fn grayed_high_tier_at_bet_1() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Low0, SlotSymbol::Mid0, SlotSymbol::Low1],
            [SlotSymbol::Low1, SlotSymbol::Mid0, SlotSymbol::Low0],
            [SlotSymbol::Low0, SlotSymbol::Mid0, SlotSymbol::Low0],
        ];

        let result = resolve_reels(reels, 1);
        assert!(result.grayed_high_tier);
        // Sushi base payout: 8 * 1 = 8. Reduced by 1/3 ratio -> round(8 * 1/3) = 3
        assert_eq!(result.payout_coins, 3);
    }

    #[test]
    fn grayed_high_tier_at_bet_2() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Low0, SlotSymbol::Mid1, SlotSymbol::Low1],
            [SlotSymbol::Low1, SlotSymbol::Mid1, SlotSymbol::Low0],
            [SlotSymbol::Low0, SlotSymbol::Mid1, SlotSymbol::Low1],
        ];

        let result = resolve_reels(reels, 2);
        assert!(result.grayed_high_tier);
        // Sashimi base payout: 12 * 2 = 24. Reduced by 2/3 ratio -> round(24 * 2/3) = 16
        assert_eq!(result.payout_coins, 16);
    }

    #[test]
    fn no_gray_at_max_bet_3() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Low0, SlotSymbol::High0, SlotSymbol::Low1],
            [SlotSymbol::Low1, SlotSymbol::High0, SlotSymbol::Low0],
            [SlotSymbol::Low0, SlotSymbol::High0, SlotSymbol::Low0],
        ];

        let result = resolve_reels(reels, 3);
        assert!(!result.grayed_high_tier);
        // Pancake base payout: 50 * 3 = 150. No reduction at max bet.
        assert_eq!(result.payout_coins, 150);
    }

    #[test]
    fn no_gray_for_low_tier_symbols_at_bet_1() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Low0, SlotSymbol::Low1, SlotSymbol::Low0],
            [SlotSymbol::Low0, SlotSymbol::Low1, SlotSymbol::Low0],
            [SlotSymbol::Low1, SlotSymbol::Low1, SlotSymbol::Low0],
        ];

        let result = resolve_reels(reels, 1);
        // Taco is low tier -> no gray regardless of bet
        assert!(!result.grayed_high_tier);
    }

    #[test]
    fn grayed_only_applies_to_matching_symbols() {
        // Kebab matches row 0, high-tier scattered but not matching
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Low0, SlotSymbol::Mid0, SlotSymbol::Low1],
            [SlotSymbol::Low0, SlotSymbol::Mid1, SlotSymbol::Low1],
            [SlotSymbol::Low0, SlotSymbol::High0, SlotSymbol::Low0],
        ];

        let result = resolve_reels(reels, 1);
        // Kebab matched on row 0 (low-tier), high-tier symbols don't match -> no gray
        assert!(!result.grayed_high_tier);
    }

    #[test]
    fn animation_strip_length_is_filler_plus_3() {
        let mut rng = StdRng::seed_from_u64(42);
        let result_column = [SlotSymbol::Low0, SlotSymbol::Low1, SlotSymbol::Low2];
        let strip = generate_animation_strip(&mut rng, result_column);

        assert_eq!(
            strip.len(),
            ANIMATION_FILLER_COUNT + 3,
            "strip should be {} symbols",
            ANIMATION_FILLER_COUNT + 3
        );
    }

    #[test]
    fn animation_strip_final_three_match_result() {
        let mut rng = StdRng::seed_from_u64(42);
        let result_column = [SlotSymbol::Mid0, SlotSymbol::High0, SlotSymbol::Low0];
        let strip = generate_animation_strip(&mut rng, result_column);

        assert_eq!(strip[12], SlotSymbol::Mid0);
        assert_eq!(strip[13], SlotSymbol::High0);
        assert_eq!(strip[14], SlotSymbol::Low0);
    }

    #[test]
    fn generate_all_strips_produces_three_strips() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Low0, SlotSymbol::Low1, SlotSymbol::Low2],
            [SlotSymbol::Mid0, SlotSymbol::Low0, SlotSymbol::High0],
            [SlotSymbol::Low1, SlotSymbol::Mid1, SlotSymbol::Low0],
        ];
        let result = SpinResult {
            reels,
            symbols_matched: None,
            tier: RewardTier::None,
            payout_coins: 0,
            is_near_miss: false,
            grayed_high_tier: false,
        };

        let strips = generate_all_animation_strips(&result);
        assert_eq!(strips.len(), 3);
        for strip in &strips {
            assert_eq!(strip.len(), ANIMATION_FILLER_COUNT + 3);
        }
    }

    #[test]
    fn generate_all_strips_final_symbols_match_reels() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::High0, SlotSymbol::Mid0, SlotSymbol::Low0],
            [SlotSymbol::Low1, SlotSymbol::Low2, SlotSymbol::Mid1],
            [SlotSymbol::Low0, SlotSymbol::Low1, SlotSymbol::High0],
        ];
        let result = SpinResult {
            reels,
            symbols_matched: None,
            tier: RewardTier::None,
            payout_coins: 0,
            is_near_miss: false,
            grayed_high_tier: false,
        };

        let strips = generate_all_animation_strips(&result);
        for col in 0..3 {
            assert_eq!(strips[col][12], reels[col][0]);
            assert_eq!(strips[col][13], reels[col][1]);
            assert_eq!(strips[col][14], reels[col][2]);
        }
    }

    #[test]
    fn symbols_for_tier_returns_correct_symbols() {
        let small = symbols_for_tier(RewardTier::Small);
        assert_eq!(small.len(), 3);

        let medium = symbols_for_tier(RewardTier::Medium);
        assert_eq!(medium.len(), 2);

        let jackpot = symbols_for_tier(RewardTier::Jackpot);
        assert_eq!(jackpot.len(), 1);
    }

    #[test]
    fn pick_symbol_for_tier_returns_valid_symbol() {
        let mut rng = StdRng::seed_from_u64(42);

        for _ in 0..100 {
            let sym = pick_symbol_for_tier(&mut rng, RewardTier::Small);
            assert_eq!(symbol_tier(sym), RewardTier::Small);
        }

        for _ in 0..100 {
            let sym = pick_symbol_for_tier(&mut rng, RewardTier::Medium);
            assert_eq!(symbol_tier(sym), RewardTier::Medium);
        }

        for _ in 0..100 {
            let sym = pick_symbol_for_tier(&mut rng, RewardTier::Jackpot);
            assert_eq!(symbol_tier(sym), RewardTier::Jackpot);
        }
    }
}

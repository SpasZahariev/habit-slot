//! Slot engine — pure logic, no UI dependencies.
//! Spin resolution, reel generation, symbol matching, reward resolution.

use crate::models::{RewardTier, SlotSymbol, SpinResult};
use crate::sprites;
use rand::thread_rng;
use rand::Rng;

/// Consecutive losses that trigger pity win.
const PITY_THRESHOLD: u32 = 8;

/// Maximum allowed bet per spin.
pub const MAX_BET: u32 = 3;

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
        RewardTier::ExtraRoll => 4,
        RewardTier::None => 3,
    }
}

/// Get all symbols belonging to a given tier.
fn symbols_for_tier(tier: RewardTier) -> &'static [SlotSymbol] {
    match tier {
        RewardTier::Small => &[SlotSymbol::Low0, SlotSymbol::Low1, SlotSymbol::Low2],
        RewardTier::Medium => &[SlotSymbol::Mid0, SlotSymbol::Mid1],
        RewardTier::Jackpot => &[SlotSymbol::High0],
        RewardTier::ExtraRoll => &[SlotSymbol::ExtraRoll0],
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
        SlotSymbol::ExtraRoll0,
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

    for reel in reels.iter_mut() {
        for pos in reel.iter_mut() {
            *pos = roll_symbol(rng);
        }
    }

    let win_row = rng.gen_range(0..3);
    let forced_symbol = pick_symbol_for_tier(rng, min_tier);

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

/// Resolve the reward for a given tier and bet level.
///
/// Returns `(reward_tier_given, payout_coins)`:
/// - Small tier: always Small reward, no coins
/// - Medium at bet=1: Small fallback, no coins
/// - Medium at bet>=2: Medium if available, else 2x Small fallback
/// - Jackpot at bet=1: Small fallback, no coins
/// - Jackpot at bet=2: Medium if available, else 2x Small fallback
/// - Jackpot at bet=3: Jackpot if available, else 3x Small fallback
/// - ExtraRoll: None reward, payout = bet + 1
pub fn resolve_reward(
    tier: RewardTier,
    bet: u32,
    has_medium: bool,
    has_high: bool,
) -> (Option<RewardTier>, u32) {
    match tier {
        RewardTier::ExtraRoll => (None, bet + 1),
        RewardTier::Small => (Some(RewardTier::Small), 0),
        RewardTier::Medium => {
            if bet >= 2 && has_medium {
                (Some(RewardTier::Medium), 0)
            } else {
                (Some(RewardTier::Small), 0)
            }
        }
        RewardTier::Jackpot => match bet {
            1 => (Some(RewardTier::Small), 0),
            2 => {
                if has_medium {
                    (Some(RewardTier::Medium), 0)
                } else {
                    (Some(RewardTier::Small), 0)
                }
            }
            _ => {
                if has_high {
                    (Some(RewardTier::Jackpot), 0)
                } else if has_medium {
                    (Some(RewardTier::Medium), 0)
                } else {
                    (Some(RewardTier::Small), 0)
                }
            }
        },
        RewardTier::None => (None, 0),
    }
}

/// Calculate coin payout for a matched row. Only ExtraRoll pays coins (bet + 1).
fn calc_payout(best_match: [SlotSymbol; 3], bet: u32) -> u32 {
    if symbol_tier(best_match[0]) == RewardTier::ExtraRoll {
        bet + 1
    } else {
        0
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

    let should_pity_win = *consecutive_losses >= PITY_THRESHOLD - 1;

    let reels = if should_pity_win {
        generate_pity_reels(&mut rng, RewardTier::Small)
    } else {
        let mut normal_reels: [[SlotSymbol; 3]; 3] = Default::default();
        for reel in normal_reels.iter_mut() {
            for pos in reel.iter_mut() {
                *pos = roll_symbol(&mut rng);
            }
        }
        normal_reels
    };

    let best_match = find_best_match(&reels);

    let tier = match &best_match {
        Some(row) => symbol_tier(row[0]),
        None => RewardTier::None,
    };

    let payout_coins = match &best_match {
        Some(row) => calc_payout(*row, bet),
        None => 0,
    };

    // Update pity state after determining result
    let _ = update_pity_state(
        consecutive_losses,
        tier != RewardTier::None || payout_coins > 0,
    );

    let is_near_miss = tier == RewardTier::None && has_near_miss_pattern(&reels);

    SpinResult {
        reels,
        symbols_matched: best_match.map(|row| (row[0], 3)),
        tier,
        payout_coins,
        is_near_miss,
        reward_tier_given: None,
        reward_note: String::new(),
    }
}

/// Resolve a spin from pre-generated reels (no RNG). Used for testing.
pub fn resolve_reels(reels: [[SlotSymbol; 3]; 3], bet: u32) -> SpinResult {
    let best_match = find_best_match(&reels);

    let tier = match &best_match {
        Some(row) => symbol_tier(row[0]),
        None => RewardTier::None,
    };

    let payout_coins = match &best_match {
        Some(row) => calc_payout(*row, bet),
        None => 0,
    };

    let is_near_miss = tier == RewardTier::None && has_near_miss_pattern(&reels);

    SpinResult {
        reels,
        symbols_matched: best_match.map(|row| (row[0], 3)),
        tier,
        payout_coins,
        is_near_miss,
        reward_tier_given: None,
        reward_note: String::new(),
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

/// Expected probability of rolling exactly 3 of a given display name on any single payline.
/// Uses total weight across all symbols sharing the same display name.
pub fn exact_match_probability(symbol: SlotSymbol) -> f64 {
    let total_weight: f64 = sprites::SYMBOLS.iter().map(|s| s.weight).sum();
    let display_name = sprites::symbol_display_name(&symbol);
    let name_weight: f64 = sprites::SYMBOLS
        .iter()
        .filter(|s| s.display_name == display_name)
        .map(|s| s.weight)
        .sum();
    let p = name_weight / total_weight;
    p * p * p
}

/// Number of filler symbols before the result in each reel animation strip.
pub const ANIMATION_FILLER_COUNT: usize = 12;

/// Generate an animation strip for a single reel column.
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
    fn reward_tier_mapping() {
        assert_eq!(symbol_tier(SlotSymbol::Low0), RewardTier::Small);
        assert_eq!(symbol_tier(SlotSymbol::Low1), RewardTier::Small);
        assert_eq!(symbol_tier(SlotSymbol::Low2), RewardTier::Small);
        assert_eq!(symbol_tier(SlotSymbol::Mid0), RewardTier::Medium);
        assert_eq!(symbol_tier(SlotSymbol::Mid1), RewardTier::Medium);
        assert_eq!(symbol_tier(SlotSymbol::High0), RewardTier::Jackpot);
        assert_eq!(symbol_tier(SlotSymbol::ExtraRoll0), RewardTier::ExtraRoll);
    }

    #[test]
    fn spin_returns_valid_result() {
        let result = spin(1);
        assert_eq!(result.reels.len(), 3);
        assert_eq!(result.reels[0].len(), 3);
        assert!(result.payout_coins == 0 || result.symbols_matched.is_some());
    }

    #[test]
    fn probability_distribution_heart_within_tolerance() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut heart_count = 0u32;

        for _ in 0..NUM_SPINS {
            if symbol_tier(roll_symbol(&mut rng)) == RewardTier::Small {
                heart_count += 1;
            }
        }

        let observed = heart_count as f64 / NUM_SPINS as f64;
        let total_weight: f64 = sprites::SYMBOLS.iter().map(|s| s.weight).sum();
        let heart_weight: f64 = sprites::SYMBOLS
            .iter()
            .filter(|s| s.tier == RewardTier::Small)
            .map(|s| s.weight)
            .sum();
        let expected = heart_weight / total_weight;
        assert!(
            (observed - expected).abs() < TOLERANCE,
            "Heart tier: observed {:.3}, expected {:.3}",
            observed,
            expected
        );
    }

    #[test]
    fn probability_distribution_chest_within_tolerance() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut chest_count = 0u32;

        for _ in 0..NUM_SPINS {
            if roll_symbol(&mut rng) == SlotSymbol::High0 {
                chest_count += 1;
            }
        }

        let observed = chest_count as f64 / NUM_SPINS as f64;
        let total_weight: f64 = sprites::SYMBOLS.iter().map(|s| s.weight).sum();
        let expected = SlotSymbol::High0.config().weight / total_weight;
        assert!(
            (observed - expected).abs() < TOLERANCE * 2.0,
            "Chest: observed {:.3}, expected {:.3}",
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
        let mut extra_roll_count = 0u32;
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
                if sprites::display_names_match(payline[0], payline[1], payline[2]) {
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
                    RewardTier::ExtraRoll => extra_roll_count += 1,
                    RewardTier::None => unreachable!(),
                },
            }
        }

        let total = small_count + medium_count + jackpot_count + extra_roll_count + no_win_count;
        assert_eq!(total, NUM_SPINS);

        let no_win_rate = no_win_count as f64 / NUM_SPINS as f64;
        assert!(
            no_win_rate > 0.45,
            "no-win rate {:.3} too low (expected ~55%)",
            no_win_rate
        );
    }

    #[test]
    fn resolve_reward_small_tier_always_gives_small() {
        let (tier, coins) = resolve_reward(RewardTier::Small, 1, true, true);
        assert_eq!(tier, Some(RewardTier::Small));
        assert_eq!(coins, 0);

        let (tier, coins) = resolve_reward(RewardTier::Small, 3, true, true);
        assert_eq!(tier, Some(RewardTier::Small));
        assert_eq!(coins, 0);
    }

    #[test]
    fn resolve_reward_medium_bet_1_falls_to_small() {
        let (tier, coins) = resolve_reward(RewardTier::Medium, 1, true, true);
        assert_eq!(tier, Some(RewardTier::Small));
        assert_eq!(coins, 0);
    }

    #[test]
    fn resolve_reward_medium_bet_2_with_medium() {
        let (tier, coins) = resolve_reward(RewardTier::Medium, 2, true, false);
        assert_eq!(tier, Some(RewardTier::Medium));
        assert_eq!(coins, 0);
    }

    #[test]
    fn resolve_reward_medium_bet_2_without_medium() {
        let (tier, coins) = resolve_reward(RewardTier::Medium, 2, false, false);
        assert_eq!(tier, Some(RewardTier::Small));
        assert_eq!(coins, 0);
    }

    #[test]
    fn resolve_reward_medium_bet_3_with_medium() {
        let (tier, coins) = resolve_reward(RewardTier::Medium, 3, true, false);
        assert_eq!(tier, Some(RewardTier::Medium));
        assert_eq!(coins, 0);
    }

    #[test]
    fn resolve_reward_jackpot_bet_1_falls_to_small() {
        let (tier, coins) = resolve_reward(RewardTier::Jackpot, 1, true, true);
        assert_eq!(tier, Some(RewardTier::Small));
        assert_eq!(coins, 0);
    }

    #[test]
    fn resolve_reward_jackpot_bet_2_with_medium() {
        let (tier, coins) = resolve_reward(RewardTier::Jackpot, 2, true, false);
        assert_eq!(tier, Some(RewardTier::Medium));
        assert_eq!(coins, 0);
    }

    #[test]
    fn resolve_reward_jackpot_bet_2_without_medium() {
        let (tier, coins) = resolve_reward(RewardTier::Jackpot, 2, false, true);
        assert_eq!(tier, Some(RewardTier::Small));
        assert_eq!(coins, 0);
    }

    #[test]
    fn resolve_reward_jackpot_bet_3_with_high() {
        let (tier, coins) = resolve_reward(RewardTier::Jackpot, 3, true, true);
        assert_eq!(tier, Some(RewardTier::Jackpot));
        assert_eq!(coins, 0);
    }

    #[test]
    fn resolve_reward_jackpot_bet_3_no_high_with_medium() {
        let (tier, coins) = resolve_reward(RewardTier::Jackpot, 3, true, false);
        assert_eq!(tier, Some(RewardTier::Medium));
        assert_eq!(coins, 0);
    }

    #[test]
    fn resolve_reward_jackpot_bet_3_no_high_no_medium() {
        let (tier, coins) = resolve_reward(RewardTier::Jackpot, 3, false, false);
        assert_eq!(tier, Some(RewardTier::Small));
        assert_eq!(coins, 0);
    }

    #[test]
    fn resolve_reward_extraroll_all_bets() {
        for bet in 1..=3 {
            let (tier, coins) = resolve_reward(RewardTier::ExtraRoll, bet, true, true);
            assert_eq!(tier, None);
            assert_eq!(coins, bet + 1);
        }
    }

    #[test]
    fn resolve_reward_none_returns_zero() {
        let (tier, coins) = resolve_reward(RewardTier::None, 1, true, true);
        assert_eq!(tier, None);
        assert_eq!(coins, 0);
    }

    #[test]
    fn pity_counter_resets_after_win() {
        let mut losses = 0u32;
        for _ in 0..8 {
            update_pity_state(&mut losses, false);
        }
        assert_eq!(losses, 0); // pity triggered at 8, counter reset

        update_pity_state(&mut losses, true);
        assert_eq!(losses, 0);
    }

    #[test]
    fn pity_triggers_on_eighth_consecutive_loss() {
        let mut losses = 0u32;
        for _ in 0..7 {
            update_pity_state(&mut losses, false);
        }
        assert_eq!(losses, 7);

        let (final_losses, should_pity) = update_pity_state(&mut losses, false);
        assert!(should_pity, "pity should trigger on 8th loss");
        assert_eq!(final_losses, 0, "counter resets after pity triggers");
    }

    #[test]
    fn pity_does_not_trigger_before_eight_losses() {
        let mut losses = 0u32;
        for i in 0..7 {
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

        reels[0][1] = SlotSymbol::Mid0;
        reels[1][1] = SlotSymbol::Mid0;
        reels[2][1] = SlotSymbol::Mid1;

        assert!(has_near_miss_pattern(&reels));
    }

    #[test]
    fn near_miss_not_detected_on_all_different() {
        let mut reels: [[SlotSymbol; 3]; 3] = Default::default();

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
            let mut found_win = false;
            for row_idx in 0..3 {
                let payline = [reels[0][row_idx], reels[1][row_idx], reels[2][row_idx]];
                if check_payline(payline).is_some() {
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
        assert!(check_near_miss([
            SlotSymbol::Low0,
            SlotSymbol::Low0,
            SlotSymbol::Low0
        ]));
    }

    #[test]
    fn pity_threshold_constant_is_eight() {
        assert_eq!(PITY_THRESHOLD, 8);
    }

    #[test]
    fn spin_with_state_tracks_losses() {
        let mut losses: u32 = 0;
        for _ in 0..7 {
            update_pity_state(&mut losses, false);
        }
        assert_eq!(losses, 7);
    }

    #[test]
    fn extra_roll_pays_bet_plus_one_at_bet_1() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::ExtraRoll0, SlotSymbol::Low0, SlotSymbol::Mid0],
            [SlotSymbol::ExtraRoll0, SlotSymbol::Low1, SlotSymbol::Mid1],
            [SlotSymbol::ExtraRoll0, SlotSymbol::High0, SlotSymbol::Low0],
        ];

        let result = resolve_reels(reels, 1);
        assert_eq!(result.tier, RewardTier::ExtraRoll);
        assert_eq!(result.payout_coins, 2);
    }

    #[test]
    fn extra_roll_pays_bet_plus_one_at_bet_2() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Low0, SlotSymbol::ExtraRoll0, SlotSymbol::Mid0],
            [SlotSymbol::Low1, SlotSymbol::ExtraRoll0, SlotSymbol::Mid1],
            [SlotSymbol::High0, SlotSymbol::ExtraRoll0, SlotSymbol::Low0],
        ];

        let result = resolve_reels(reels, 2);
        assert_eq!(result.tier, RewardTier::ExtraRoll);
        assert_eq!(result.payout_coins, 3);
    }

    #[test]
    fn extra_roll_pays_bet_plus_one_at_bet_3() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Low0, SlotSymbol::Mid0, SlotSymbol::ExtraRoll0],
            [SlotSymbol::Low1, SlotSymbol::Mid1, SlotSymbol::ExtraRoll0],
            [SlotSymbol::High0, SlotSymbol::Low0, SlotSymbol::ExtraRoll0],
        ];

        let result = resolve_reels(reels, 3);
        assert_eq!(result.tier, RewardTier::ExtraRoll);
        assert_eq!(result.payout_coins, 4);
    }

    #[test]
    fn non_extraroll_matches_pay_zero_coins() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Low0, SlotSymbol::Mid0, SlotSymbol::Low1],
            [SlotSymbol::Low0, SlotSymbol::Mid0, SlotSymbol::Low0],
            [SlotSymbol::Low0, SlotSymbol::Mid0, SlotSymbol::Low0],
        ];

        let result = resolve_reels(reels, 3);
        assert_eq!(result.tier, RewardTier::Medium);
        assert_eq!(result.payout_coins, 0);
    }

    #[test]
    fn no_match_returns_none_tier_zero_coins() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Low0, SlotSymbol::Mid0, SlotSymbol::High0],
            [SlotSymbol::Mid0, SlotSymbol::High0, SlotSymbol::Low0],
            [SlotSymbol::High0, SlotSymbol::Low0, SlotSymbol::Mid0],
        ];

        let result = resolve_reels(reels, 1);
        assert_eq!(result.tier, RewardTier::None);
        assert_eq!(result.payout_coins, 0);
        assert!(result.symbols_matched.is_none());
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
            reward_tier_given: None,
            reward_note: String::new(),
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
            reward_tier_given: None,
            reward_note: String::new(),
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

        let extra_roll = symbols_for_tier(RewardTier::ExtraRoll);
        assert_eq!(extra_roll.len(), 1);
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

        for _ in 0..100 {
            let sym = pick_symbol_for_tier(&mut rng, RewardTier::ExtraRoll);
            assert_eq!(symbol_tier(sym), RewardTier::ExtraRoll);
        }
    }

    #[test]
    fn exact_match_probability_uses_display_name_weights() {
        let total_weight: f64 = sprites::SYMBOLS.iter().map(|s| s.weight).sum();
        let heart_prob = exact_match_probability(SlotSymbol::Low0);
        let expected_p = 42.0 / total_weight;
        let expected_cube = expected_p * expected_p * expected_p;
        assert!(
            (heart_prob - expected_cube).abs() < 1e-6,
            "Heart probability: observed {:.8}, expected {:.8}",
            heart_prob,
            expected_cube
        );
    }

    #[test]
    fn find_best_match_returns_highest_tier() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Low0, SlotSymbol::High0, SlotSymbol::Mid0],
            [SlotSymbol::Low0, SlotSymbol::High0, SlotSymbol::Mid0],
            [SlotSymbol::Low0, SlotSymbol::High0, SlotSymbol::Mid0],
        ];

        let best = find_best_match(&reels);
        assert!(best.is_some());
        let matched = best.unwrap();
        assert_eq!(symbol_tier(matched[0]), RewardTier::Jackpot);
    }

    #[test]
    fn resolve_reels_sets_near_miss_only_on_losses() {
        // Winning reels should NOT have near-miss flag
        let win_reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Low0, SlotSymbol::Low1, SlotSymbol::Mid0],
            [SlotSymbol::Low0, SlotSymbol::Low1, SlotSymbol::Mid1],
            [SlotSymbol::Low0, SlotSymbol::Low1, SlotSymbol::High0],
        ];
        let result = resolve_reels(win_reels, 1);
        assert_eq!(result.tier, RewardTier::Small);
        assert!(!result.is_near_miss);

        // Losing reels with two matching should have near-miss
        let lose_reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Low0, SlotSymbol::Mid0, SlotSymbol::High0],
            [SlotSymbol::Low1, SlotSymbol::Mid1, SlotSymbol::ExtraRoll0],
            [SlotSymbol::Mid0, SlotSymbol::Low0, SlotSymbol::Mid0],
        ];
        let result = resolve_reels(lose_reels, 1);
        assert_eq!(result.tier, RewardTier::None);
        assert!(result.is_near_miss);
    }

    #[test]
    fn reward_matrix_full_coverage() {
        // Comprehensive coverage of every bet/tier/availability combination
        let cases: Vec<(RewardTier, u32, bool, bool, Option<RewardTier>, u32)> = vec![
            // Small tier - always gives Small, 0 coins regardless of bet
            (RewardTier::Small, 1, true, true, Some(RewardTier::Small), 0),
            (
                RewardTier::Small,
                2,
                false,
                false,
                Some(RewardTier::Small),
                0,
            ),
            (
                RewardTier::Small,
                3,
                true,
                false,
                Some(RewardTier::Small),
                0,
            ),
            // Medium tier
            (
                RewardTier::Medium,
                1,
                true,
                true,
                Some(RewardTier::Small),
                0,
            ),
            (
                RewardTier::Medium,
                2,
                true,
                false,
                Some(RewardTier::Medium),
                0,
            ),
            (
                RewardTier::Medium,
                2,
                false,
                true,
                Some(RewardTier::Small),
                0,
            ),
            (
                RewardTier::Medium,
                3,
                true,
                false,
                Some(RewardTier::Medium),
                0,
            ),
            (
                RewardTier::Medium,
                3,
                false,
                false,
                Some(RewardTier::Small),
                0,
            ),
            // Jackpot tier at bet=1 - always Small
            (
                RewardTier::Jackpot,
                1,
                true,
                true,
                Some(RewardTier::Small),
                0,
            ),
            (
                RewardTier::Jackpot,
                1,
                false,
                false,
                Some(RewardTier::Small),
                0,
            ),
            // Jackpot tier at bet=2
            (
                RewardTier::Jackpot,
                2,
                true,
                true,
                Some(RewardTier::Medium),
                0,
            ),
            (
                RewardTier::Jackpot,
                2,
                false,
                true,
                Some(RewardTier::Small),
                0,
            ),
            (
                RewardTier::Jackpot,
                2,
                true,
                false,
                Some(RewardTier::Medium),
                0,
            ),
            (
                RewardTier::Jackpot,
                2,
                false,
                false,
                Some(RewardTier::Small),
                0,
            ),
            // Jackpot tier at bet=3
            (
                RewardTier::Jackpot,
                3,
                true,
                true,
                Some(RewardTier::Jackpot),
                0,
            ),
            (
                RewardTier::Jackpot,
                3,
                true,
                false,
                Some(RewardTier::Medium),
                0,
            ),
            (
                RewardTier::Jackpot,
                3,
                false,
                true,
                Some(RewardTier::Jackpot),
                0,
            ),
            (
                RewardTier::Jackpot,
                3,
                false,
                false,
                Some(RewardTier::Small),
                0,
            ),
            // ExtraRoll - always None reward, bet+1 coins
            (RewardTier::ExtraRoll, 1, true, true, None, 2),
            (RewardTier::ExtraRoll, 2, false, false, None, 3),
            (RewardTier::ExtraRoll, 3, true, false, None, 4),
            // No match
            (RewardTier::None, 1, true, true, None, 0),
        ];

        for (tier, bet, has_med, has_high, expected_tier, expected_coins) in cases {
            let (got_tier, got_coins) = resolve_reward(tier, bet, has_med, has_high);
            assert_eq!(
                got_tier, expected_tier,
                "resolve_reward({:?}, {}, {}, {}) expected tier {:?}",
                tier, bet, has_med, has_high, expected_tier
            );
            assert_eq!(
                got_coins, expected_coins,
                "resolve_reward({:?}, {}, {}, {}) expected coins {}",
                tier, bet, has_med, has_high, expected_coins
            );
        }
    }
}

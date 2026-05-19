//! Pure multi-row reward resolution logic.
//! No RNG, no side effects — fully deterministic and testable in isolation.

use crate::models::{RewardTier, SlotSymbol};
use crate::sprites;

/// Per-row resolved reward with matched tier, given tier, and fallback multiplier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowReward {
    pub matched_tier: RewardTier,
    pub given_tier: RewardTier,
    pub multiplier: u32,
}

/// Check if a row is a 3-of-a-kind win (all three share the same display name).
fn is_winning_row(row: [SlotSymbol; 3]) -> bool {
    sprites::display_names_match(row[0], row[1], row[2])
}

/// Get the reward tier for a symbol.
fn symbol_tier(symbol: SlotSymbol) -> RewardTier {
    symbol.config().tier
}

/// Resolve a single winning row's reward using the bet-scaled fallback matrix.
///
/// | Matched Tier | Bet   | Condition         | Given Tier | Multiplier |
/// |-------------|-------|-------------------|------------|-----------|
/// | Small       | any   | —                 | Small      | 1x        |
/// | Medium      | <2    | —                 | Small      | 1x        |
/// | Medium      | >=2   | has_med           | Medium     | 1x        |
/// | Medium      | >=2   | !has_med          | Small      | 2x        |
/// | Jackpot     | 1     | —                 | Small      | 1x        |
/// | Jackpot     | 2     | has_high          | Jackpot    | 1x        |
/// | Jackpot     | 2     | !has_high, has_med| Medium     | 1x        |
/// | Jackpot     | 2     | !has_high, !has_med| Small     | 2x        |
/// | Jackpot     | >=3   | has_high          | Jackpot    | 1x        |
/// | Jackpot     | >=3   | !has_high, has_med| Medium     | 1x        |
/// | Jackpot     | >=3   | !has_high, !has_med| Small     | 5x        |
fn resolve_single_row(
    matched_tier: RewardTier,
    bet: u32,
    has_med: bool,
    has_high: bool,
) -> RowReward {
    match matched_tier {
        // Small always gives Small at 1x
        RewardTier::Small => RowReward {
            matched_tier: RewardTier::Small,
            given_tier: RewardTier::Small,
            multiplier: 1,
        },

        // Medium at bet < 2 falls to Small at 1x
        RewardTier::Medium if bet < 2 => RowReward {
            matched_tier: RewardTier::Medium,
            given_tier: RewardTier::Small,
            multiplier: 1,
        },

        // Medium at bet >= 2
        RewardTier::Medium => {
            if has_med {
                RowReward {
                    matched_tier: RewardTier::Medium,
                    given_tier: RewardTier::Medium,
                    multiplier: 1,
                }
            } else {
                RowReward {
                    matched_tier: RewardTier::Medium,
                    given_tier: RewardTier::Small,
                    multiplier: 2,
                }
            }
        },

        // Jackpot at bet=1 always falls to Small at 1x
        RewardTier::Jackpot if bet == 1 => RowReward {
            matched_tier: RewardTier::Jackpot,
            given_tier: RewardTier::Small,
            multiplier: 1,
        },

        // Jackpot at bet=2
        RewardTier::Jackpot if bet == 2 => {
            if has_high {
                RowReward {
                    matched_tier: RewardTier::Jackpot,
                    given_tier: RewardTier::Jackpot,
                    multiplier: 1,
                }
            } else if has_med {
                RowReward {
                    matched_tier: RewardTier::Jackpot,
                    given_tier: RewardTier::Medium,
                    multiplier: 1,
                }
            } else {
                RowReward {
                    matched_tier: RewardTier::Jackpot,
                    given_tier: RewardTier::Small,
                    multiplier: 2,
                }
            }
        },

        // Jackpot at bet >= 3
        RewardTier::Jackpot => {
            if has_high {
                RowReward {
                    matched_tier: RewardTier::Jackpot,
                    given_tier: RewardTier::Jackpot,
                    multiplier: 1,
                }
            } else if has_med {
                RowReward {
                    matched_tier: RewardTier::Jackpot,
                    given_tier: RewardTier::Medium,
                    multiplier: 1,
                }
            } else {
                RowReward {
                    matched_tier: RewardTier::Jackpot,
                    given_tier: RewardTier::Small,
                    multiplier: 5,
                }
            }
        },

        // ExtraRoll and None should not reach here (filtered in resolve_all_wins)
        _ => RowReward {
            matched_tier,
            given_tier: RewardTier::Small,
            multiplier: 1,
        },
    }
}

/// Resolve all winning rows independently.
///
/// Scans each payline for 3-of-a-kind matches (by display name). For each
/// non-ExtraRoll match, resolves the reward using the bet-scaled fallback matrix.
/// ExtraRoll rows are excluded — they pay coins via existing `calc_payout` logic.
///
/// Returns an empty vec when no eligible winning rows exist.
pub fn resolve_all_wins(
    reels: [[SlotSymbol; 3]; 3],
    bet: u32,
    has_med: bool,
    has_high: bool,
) -> Vec<RowReward> {
    let mut rewards = Vec::new();

    for row_idx in 0..3 {
        let payline = [reels[0][row_idx], reels[1][row_idx], reels[2][row_idx]];

        if is_winning_row(payline) {
            let tier = symbol_tier(payline[0]);

            // ExtraRoll pays coins directly, not a RowReward
            if tier == RewardTier::ExtraRoll || tier == RewardTier::None {
                continue;
            }

            let row_reward = resolve_single_row(tier, bet, has_med, has_high);
            rewards.push(row_reward);
        }
    }

    rewards
}

/// Count how many ExtraRoll winning rows exist in the reels.
/// Each ExtraRoll row pays `bet + 1` coins independently.
pub fn count_extraroll_rows(reels: [[SlotSymbol; 3]; 3]) -> u32 {
    let mut count = 0u32;
    for row_idx in 0..3 {
        let payline = [reels[0][row_idx], reels[1][row_idx], reels[2][row_idx]];
        if is_winning_row(payline) && symbol_tier(payline[0]) == RewardTier::ExtraRoll {
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Fallback multiplier matrix coverage ---

    #[test]
    fn small_any_bet_always_small_1x() {
        for bet in 1..=3 {
            let r = resolve_single_row(RewardTier::Small, bet, true, true);
            assert_eq!(r.given_tier, RewardTier::Small);
            assert_eq!(r.multiplier, 1);
        }
        let r = resolve_single_row(RewardTier::Small, 2, false, false);
        assert_eq!(r.given_tier, RewardTier::Small);
        assert_eq!(r.multiplier, 1);
    }

    #[test]
    fn medium_bet_1_falls_to_small_1x() {
        let r = resolve_single_row(RewardTier::Medium, 1, true, true);
        assert_eq!(r.given_tier, RewardTier::Small);
        assert_eq!(r.multiplier, 1);
    }

    #[test]
    fn medium_bet_2_with_med_gives_medium_1x() {
        let r = resolve_single_row(RewardTier::Medium, 2, true, false);
        assert_eq!(r.given_tier, RewardTier::Medium);
        assert_eq!(r.multiplier, 1);
    }

    #[test]
    fn medium_bet_2_without_med_falls_to_small_2x() {
        let r = resolve_single_row(RewardTier::Medium, 2, false, true);
        assert_eq!(r.given_tier, RewardTier::Small);
        assert_eq!(r.multiplier, 2);
    }

    #[test]
    fn medium_bet_3_with_med_gives_medium_1x() {
        let r = resolve_single_row(RewardTier::Medium, 3, true, false);
        assert_eq!(r.given_tier, RewardTier::Medium);
        assert_eq!(r.multiplier, 1);
    }

    #[test]
    fn medium_bet_3_without_med_falls_to_small_2x() {
        let r = resolve_single_row(RewardTier::Medium, 3, false, false);
        assert_eq!(r.given_tier, RewardTier::Small);
        assert_eq!(r.multiplier, 2);
    }

    #[test]
    fn jackpot_bet_1_falls_to_small_1x() {
        let r = resolve_single_row(RewardTier::Jackpot, 1, true, true);
        assert_eq!(r.given_tier, RewardTier::Small);
        assert_eq!(r.multiplier, 1);

        let r = resolve_single_row(RewardTier::Jackpot, 1, false, false);
        assert_eq!(r.given_tier, RewardTier::Small);
        assert_eq!(r.multiplier, 1);
    }

    #[test]
    fn jackpot_bet_2_with_high_gives_jackpot_1x() {
        let r = resolve_single_row(RewardTier::Jackpot, 2, true, true);
        assert_eq!(r.given_tier, RewardTier::Jackpot);
        assert_eq!(r.multiplier, 1);
    }

    #[test]
    fn jackpot_bet_2_no_high_with_med_gives_medium_1x() {
        let r = resolve_single_row(RewardTier::Jackpot, 2, true, false);
        assert_eq!(r.given_tier, RewardTier::Medium);
        assert_eq!(r.multiplier, 1);
    }

    #[test]
    fn jackpot_bet_2_no_high_no_med_falls_to_small_2x() {
        let r = resolve_single_row(RewardTier::Jackpot, 2, false, false);
        assert_eq!(r.given_tier, RewardTier::Small);
        assert_eq!(r.multiplier, 2);
    }

    #[test]
    fn jackpot_bet_3_with_high_gives_jackpot_1x() {
        let r = resolve_single_row(RewardTier::Jackpot, 3, true, true);
        assert_eq!(r.given_tier, RewardTier::Jackpot);
        assert_eq!(r.multiplier, 1);
    }

    #[test]
    fn jackpot_bet_3_no_high_with_med_gives_medium_1x() {
        let r = resolve_single_row(RewardTier::Jackpot, 3, true, false);
        assert_eq!(r.given_tier, RewardTier::Medium);
        assert_eq!(r.multiplier, 1);
    }

    #[test]
    fn jackpot_bet_3_no_high_no_med_falls_to_small_5x() {
        let r = resolve_single_row(RewardTier::Jackpot, 3, false, false);
        assert_eq!(r.given_tier, RewardTier::Small);
        assert_eq!(r.multiplier, 5);
    }

    // --- Multi-row combinations ---
    // Reels layout: reels[col][row]. Payline row R = reels[0][R], reels[1][R], reels[2][R].

    #[test]
    fn two_small_wins_same_tier() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Low0, SlotSymbol::Mid0, SlotSymbol::Low2],
            [SlotSymbol::Low0, SlotSymbol::High0, SlotSymbol::Low2],
            [SlotSymbol::Low0, SlotSymbol::ExtraRoll0, SlotSymbol::Low2],
        ];

        // Row 0: Low0/ Low0/ Low0 -> Small win
        // Row 1: Mid0/ High0/ ExtraRoll0 -> no match
        // Row 2: Low2/ Low2/ Low2 -> Small win
        let rewards = resolve_all_wins(reels, 1, true, true);

        assert_eq!(rewards.len(), 2);
        assert_eq!(rewards[0].matched_tier, RewardTier::Small);
        assert_eq!(rewards[0].given_tier, RewardTier::Small);
        assert_eq!(rewards[0].multiplier, 1);
        assert_eq!(rewards[1].matched_tier, RewardTier::Small);
        assert_eq!(rewards[1].given_tier, RewardTier::Small);
        assert_eq!(rewards[1].multiplier, 1);
    }

    #[test]
    fn two_different_tier_wins() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Low0, SlotSymbol::Mid0, SlotSymbol::High0],
            [SlotSymbol::Low0, SlotSymbol::ExtraRoll0, SlotSymbol::High0],
            [SlotSymbol::Low0, SlotSymbol::Mid1, SlotSymbol::High0],
        ];

        // Row 0: Low0/ Low0/ Low0 -> Small win
        // Row 2: High0/ High0/ High0 -> Jackpot win
        let rewards = resolve_all_wins(reels, 3, true, true);

        assert_eq!(rewards.len(), 2);
        assert_eq!(rewards[0].matched_tier, RewardTier::Small);
        assert_eq!(rewards[0].given_tier, RewardTier::Small);
        assert_eq!(rewards[0].multiplier, 1);
        assert_eq!(rewards[1].matched_tier, RewardTier::Jackpot);
        assert_eq!(rewards[1].given_tier, RewardTier::Jackpot);
        assert_eq!(rewards[1].multiplier, 1);
    }

    #[test]
    fn three_jackpot_wins_at_bet_3_no_rewards_fall_to_small_5x() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::High0, SlotSymbol::High0, SlotSymbol::High0],
            [SlotSymbol::High0, SlotSymbol::High0, SlotSymbol::High0],
            [SlotSymbol::High0, SlotSymbol::High0, SlotSymbol::High0],
        ];

        // All three rows: High0/ High0/ High0 -> Jackpot win
        let rewards = resolve_all_wins(reels, 3, false, false);

        assert_eq!(rewards.len(), 3);
        for r in &rewards {
            assert_eq!(r.matched_tier, RewardTier::Jackpot);
            assert_eq!(r.given_tier, RewardTier::Small);
            assert_eq!(r.multiplier, 5);
        }
    }

    #[test]
    fn extraroll_row_excluded_from_rewards() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::ExtraRoll0, SlotSymbol::Low0, SlotSymbol::Mid0],
            [SlotSymbol::ExtraRoll0, SlotSymbol::Low1, SlotSymbol::Mid1],
            [SlotSymbol::ExtraRoll0, SlotSymbol::High0, SlotSymbol::Low2],
        ];

        // Row 0: ExtraRoll0/ ExtraRoll0/ ExtraRoll0 -> ExtraRoll (excluded from rewards)
        let rewards = resolve_all_wins(reels, 1, true, true);
        assert!(rewards.is_empty());

        let extraroll_count = count_extraroll_rows(reels);
        assert_eq!(extraroll_count, 1);
    }

    #[test]
    fn extraroll_plus_small_win_both_resolved() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::ExtraRoll0, SlotSymbol::Mid0, SlotSymbol::Low0],
            [SlotSymbol::ExtraRoll0, SlotSymbol::High0, SlotSymbol::Low0],
            [SlotSymbol::ExtraRoll0, SlotSymbol::Low2, SlotSymbol::Low0],
        ];

        // Row 0: ExtraRoll0/ ExtraRoll0/ ExtraRoll0 -> ExtraRoll (coins only)
        // Row 2: Low0/ Low0/ Low0 -> Small reward
        let rewards = resolve_all_wins(reels, 1, true, true);
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].matched_tier, RewardTier::Small);
        assert_eq!(rewards[0].given_tier, RewardTier::Small);

        let extraroll_count = count_extraroll_rows(reels);
        assert_eq!(extraroll_count, 1);
    }

    #[test]
    fn no_winning_rows_returns_empty() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Low0, SlotSymbol::Mid0, SlotSymbol::High0],
            [SlotSymbol::Mid0, SlotSymbol::High0, SlotSymbol::Low0],
            [SlotSymbol::High0, SlotSymbol::Low0, SlotSymbol::Mid1],
        ];

        let rewards = resolve_all_wins(reels, 1, true, true);
        assert!(rewards.is_empty());
    }

    #[test]
    fn three_extraroll_rows_no_rewards_all_coins() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::ExtraRoll0, SlotSymbol::ExtraRoll0, SlotSymbol::ExtraRoll0],
            [SlotSymbol::ExtraRoll0, SlotSymbol::ExtraRoll0, SlotSymbol::ExtraRoll0],
            [SlotSymbol::ExtraRoll0, SlotSymbol::ExtraRoll0, SlotSymbol::ExtraRoll0],
        ];

        // All three rows: ExtraRoll0/ ExtraRoll0/ ExtraRoll0 -> ExtraRoll wins
        let rewards = resolve_all_wins(reels, 2, true, true);
        assert!(rewards.is_empty());
        assert_eq!(count_extraroll_rows(reels), 3);
    }

    #[test]
    fn jackpot_bet_2_no_fallback_at_all_small_2x() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::High0, SlotSymbol::Low0, SlotSymbol::Mid0],
            [SlotSymbol::High0, SlotSymbol::Low1, SlotSymbol::Mid1],
            [SlotSymbol::High0, SlotSymbol::ExtraRoll0, SlotSymbol::Low2],
        ];

        // Row 0: High0/ High0/ High0 -> Jackpot win at bet=2, no fallback
        let rewards = resolve_all_wins(reels, 2, false, false);
        assert_eq!(rewards.len(), 1);
        assert_eq!(rewards[0].matched_tier, RewardTier::Jackpot);
        assert_eq!(rewards[0].given_tier, RewardTier::Small);
        assert_eq!(rewards[0].multiplier, 2);
    }

    #[test]
    fn mixed_medium_and_jackpot_no_high_with_med() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::Mid0, SlotSymbol::Low0, SlotSymbol::High0],
            [SlotSymbol::Mid0, SlotSymbol::ExtraRoll0, SlotSymbol::High0],
            [SlotSymbol::Mid0, SlotSymbol::Low1, SlotSymbol::High0],
        ];

        // Row 0: Mid0/ Mid0/ Mid0 -> Medium win at bet=3 with has_med -> Medium 1x
        // Row 2: High0/ High0/ High0 -> Jackpot win at bet=3 no high w/ med -> Medium 1x
        let rewards = resolve_all_wins(reels, 3, true, false);

        assert_eq!(rewards.len(), 2);
        assert_eq!(rewards[0].matched_tier, RewardTier::Medium);
        assert_eq!(rewards[0].given_tier, RewardTier::Medium);
        assert_eq!(rewards[0].multiplier, 1);
        assert_eq!(rewards[1].matched_tier, RewardTier::Jackpot);
        assert_eq!(rewards[1].given_tier, RewardTier::Medium);
        assert_eq!(rewards[1].multiplier, 1);
    }

    #[test]
    fn count_extraroll_returns_zero_on_no_match() {
        let reels: [[SlotSymbol; 3]; 3] = [
            [SlotSymbol::ExtraRoll0, SlotSymbol::Low0, SlotSymbol::Mid0],
            [SlotSymbol::Low0, SlotSymbol::High0, SlotSymbol::Mid1],
            [SlotSymbol::Mid0, SlotSymbol::Low1, SlotSymbol::High0],
        ];
        assert_eq!(count_extraroll_rows(reels), 0);
    }

    #[test]
    fn row_reward_partial_eq() {
        let r1 = RowReward {
            matched_tier: RewardTier::Jackpot,
            given_tier: RewardTier::Small,
            multiplier: 5,
        };
        let r2 = RowReward {
            matched_tier: RewardTier::Jackpot,
            given_tier: RewardTier::Small,
            multiplier: 5,
        };
        assert_eq!(r1, r2);

        let r3 = RowReward {
            multiplier: 2,
            ..r1.clone()
        };
        assert_ne!(r1, r3);
    }
}

//! Reward pool resolver module
//! Tier-based reward pools, random selection, milestone tracking.

use std::collections::HashSet;

use crate::models::{RewardPool, RewardTier};

/// Streak milestone targets: (target_days, reward_tier).
pub const STREAK_MILESTONES: [(u32, RewardTier); 5] = [
    (7, RewardTier::Small),
    (14, RewardTier::Small),
    (30, RewardTier::Medium),
    (60, RewardTier::Medium),
    (90, RewardTier::Jackpot),
];

/// Completion milestone targets: (target_completions, reward_tier).
pub const COMPLETION_MILESTONES: [(u32, RewardTier); 5] = [
    (10, RewardTier::Small),
    (25, RewardTier::Small),
    (50, RewardTier::Medium),
    (100, RewardTier::Medium),
    (200, RewardTier::Jackpot),
];

/// Tracks milestone progress for a single habit.
#[derive(Debug, Clone)]
pub struct MilestoneTracker {
    /// Set of claimed streak tier indices (into STREAK_MILESTONES).
    pub claimed_streak_tiers: HashSet<usize>,
    /// Set of claimed completion tier indices (into COMPLETION_MILESTONES).
    pub claimed_completion_tiers: HashSet<usize>,
}

impl Default for MilestoneTracker {
    fn default() -> Self {
        Self {
            claimed_streak_tiers: HashSet::new(),
            claimed_completion_tiers: HashSet::new(),
        }
    }
}

/// Result of checking milestones.
#[derive(Debug, Clone)]
pub struct MilestoneCheckResult {
    /// Which milestone was just unlocked (only one at a time per call).
    pub newly_claimed: Option<MilestoneKind>,
    /// Next unclaimed streak goal (days target, tier).
    pub next_streak_goal: (u32, RewardTier),
    /// Next unclaimed completion goal (count target, tier).
    pub next_completion_goal: (u32, RewardTier),
}

impl Default for MilestoneCheckResult {
    fn default() -> Self {
        Self {
            newly_claimed: None,
            next_streak_goal: STREAK_MILESTONES[0],
            next_completion_goal: COMPLETION_MILESTONES[0],
        }
    }
}

/// Identifies a claimed milestone kind and tier index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MilestoneKind {
    Streak(usize),
    Completion(usize),
}

/// Check current streak and completion progress against milestone tiers.
/// Returns any newly claimed milestone and the next active goals.
pub fn check_milestones(
    tracker: &mut MilestoneTracker,
    current_streak_days: u32,
    total_completions: u32,
) -> MilestoneCheckResult {
    let mut result = MilestoneCheckResult::default();

    // Check streak milestones
    for (idx, &(target_days, _tier)) in STREAK_MILESTONES.iter().enumerate() {
        if !tracker.claimed_streak_tiers.contains(&idx) && current_streak_days >= target_days {
            tracker.claimed_streak_tiers.insert(idx);
            if result.newly_claimed.is_none() {
                result.newly_claimed = Some(MilestoneKind::Streak(idx));
            }
        }
    }

    // Check completion milestones
    for (idx, &(target_completions, _tier)) in COMPLETION_MILESTONES.iter().enumerate() {
        if !tracker.claimed_completion_tiers.contains(&idx)
            && total_completions >= target_completions
        {
            tracker.claimed_completion_tiers.insert(idx);
            if result.newly_claimed.is_none() {
                result.newly_claimed = Some(MilestoneKind::Completion(idx));
            }
        }
    }

    // Find next unclaimed streak goal
    result.next_streak_goal = (0..STREAK_MILESTONES.len())
        .find(|&i| !tracker.claimed_streak_tiers.contains(&i))
        .map(|i| STREAK_MILESTONES[i])
        .unwrap_or(*STREAK_MILESTONES.last().unwrap());

    // Find next unclaimed completion goal
    result.next_completion_goal = (0..COMPLETION_MILESTONES.len())
        .find(|&i| !tracker.claimed_completion_tiers.contains(&i))
        .map(|i| COMPLETION_MILESTONES[i])
        .unwrap_or(*COMPLETION_MILESTONES.last().unwrap());

    result
}

/// Select a random reward string from the appropriate pool tier.
/// Returns None if the pool for the given tier is empty.
pub fn select_reward(pool: &RewardPool, tier: RewardTier) -> Option<String> {
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    let rewards = match tier {
        RewardTier::Small => &pool.small_rewards,
        RewardTier::Medium => &pool.medium_rewards,
        RewardTier::Jackpot => &pool.jackpot_rewards,
        RewardTier::ExtraRoll | RewardTier::None => return None,
    };

    if rewards.is_empty() {
        return None;
    }

    let chosen = rewards.choose(&mut thread_rng())?;
    Some(chosen.clone())
}

/// Get the reward tier for a claimed milestone.
pub fn get_milestone_tier(kind: MilestoneKind) -> RewardTier {
    match kind {
        MilestoneKind::Streak(idx) => STREAK_MILESTONES
            .get(idx)
            .map(|(_, t)| *t)
            .unwrap_or(RewardTier::Small),
        MilestoneKind::Completion(idx) => COMPLETION_MILESTONES
            .get(idx)
            .map(|(_, t)| *t)
            .unwrap_or(RewardTier::Small),
    }
}

/// Format milestone progress string for display.
pub fn format_progress(current: u32, target: u32, label: &str) -> String {
    if current >= target {
        format!("{} achieved! ({}/{})", label, current, target)
    } else {
        format!("{}/{} {} remaining", current, target, label.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streak_milestone_unlocks_at_target() {
        let mut tracker = MilestoneTracker::default();
        let result = check_milestones(&mut tracker, 7, 0);

        assert!(result.newly_claimed.is_some());
        assert_eq!(result.newly_claimed.unwrap(), MilestoneKind::Streak(0));
        assert!(tracker.claimed_streak_tiers.contains(&0));
    }

    #[test]
    fn completion_milestone_unlocks_at_target() {
        let mut tracker = MilestoneTracker::default();
        let result = check_milestones(&mut tracker, 0, 10);

        assert!(result.newly_claimed.is_some());
        assert_eq!(result.newly_claimed.unwrap(), MilestoneKind::Completion(0));
        assert!(tracker.claimed_completion_tiers.contains(&0));
    }

    #[test]
    fn claim_once_semantic_no_duplicate() {
        let mut tracker = MilestoneTracker::default();
        // First call at target: claims
        let r1 = check_milestones(&mut tracker, 7, 0);
        assert!(r1.newly_claimed.is_some());

        // Second call at same level: does NOT re-claim
        let r2 = check_milestones(&mut tracker, 7, 0);
        assert!(r2.newly_claimed.is_none());
    }

    #[test]
    fn next_tier_activates_after_claim() {
        let mut tracker = MilestoneTracker::default();
        // Claim 7-day streak
        check_milestones(&mut tracker, 7, 0);
        // Now advance to 14-day streak
        let result = check_milestones(&mut tracker, 14, 0);

        assert!(result.newly_claimed.is_some());
        assert_eq!(result.newly_claimed.unwrap(), MilestoneKind::Streak(1));
    }

    #[test]
    fn empty_pool_returns_none() {
        let pool = RewardPool {
            small_rewards: vec![],
            medium_rewards: vec![],
            jackpot_rewards: vec![],
        };
        assert_eq!(select_reward(&pool, RewardTier::Small), None);
        assert_eq!(select_reward(&pool, RewardTier::Medium), None);
        assert_eq!(select_reward(&pool, RewardTier::Jackpot), None);
    }

    #[test]
    fn empty_pool_tier_returns_none() {
        let pool = RewardPool {
            small_rewards: vec!["test".to_string()],
            medium_rewards: vec![],
            jackpot_rewards: vec!["big".to_string()],
        };
        // Small and jackpot have items, medium is empty
        assert!(select_reward(&pool, RewardTier::Small).is_some());
        assert_eq!(select_reward(&pool, RewardTier::Medium), None);
        assert!(select_reward(&pool, RewardTier::Jackpot).is_some());
    }

    #[test]
    fn select_reward_returns_item_from_tier() {
        let pool = RewardPool::default();
        let reward = select_reward(&pool, RewardTier::Small);
        assert!(reward.is_some());
        let val = reward.unwrap();
        assert!(pool.small_rewards.contains(&val));
    }

    #[test]
    fn milestone_tier_mapping_correct() {
        assert_eq!(
            get_milestone_tier(MilestoneKind::Streak(0)),
            RewardTier::Small
        );
        assert_eq!(
            get_milestone_tier(MilestoneKind::Streak(2)),
            RewardTier::Medium
        );
        assert_eq!(
            get_milestone_tier(MilestoneKind::Streak(4)),
            RewardTier::Jackpot
        );
        assert_eq!(
            get_milestone_tier(MilestoneKind::Completion(0)),
            RewardTier::Small
        );
        assert_eq!(
            get_milestone_tier(MilestoneKind::Completion(3)),
            RewardTier::Medium
        );
    }

    #[test]
    fn next_goal_advances_after_claim() {
        let mut tracker = MilestoneTracker::default();
        let result = check_milestones(&mut tracker, 7, 0);

        // After claiming 7-day, next streak goal should be 14
        assert_eq!(result.next_streak_goal.0, 14);
    }

    #[test]
    fn format_progress_before_and_after_target() {
        let before = format_progress(5, 30, "days");
        assert!(before.contains("5/30"));

        let at = format_progress(30, 30, "days");
        assert!(at.contains("achieved"));

        let past = format_progress(42, 30, "days");
        assert!(past.contains("achieved"));
    }

    #[test]
    fn no_milestone_below_threshold() {
        let mut tracker = MilestoneTracker::default();
        let result = check_milestones(&mut tracker, 6, 9);

        assert!(result.newly_claimed.is_none());
        assert!(tracker.claimed_streak_tiers.is_empty());
        assert!(tracker.claimed_completion_tiers.is_empty());
    }
}

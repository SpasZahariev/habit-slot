use std::time::{Duration, Instant};

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotSymbol {
    #[default]
    Low0,
    Low1,
    Low2,
    Mid0,
    Mid1,
    High0,
    ExtraRoll0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RewardTier {
    None,
    Small,
    Medium,
    Jackpot,
    ExtraRoll,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Habit {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDate,
    pub reward_pool: RewardPool,
    pub target_days: u32,
    pub longest_streak: u32,
    pub coin_reward: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RewardPool {
    pub small_rewards: Vec<String>,
    pub medium_rewards: Vec<String>,
    pub jackpot_rewards: Vec<String>,
}

impl Default for RewardPool {
    fn default() -> Self {
        Self {
            small_rewards: vec!["Extra spin".to_string(), "Bonus coin".to_string()],
            medium_rewards: vec!["Day off pass".to_string(), "Coin doubler".to_string()],
            jackpot_rewards: vec!["Soul boost".to_string(), "Free day".to_string()],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpinResult {
    pub reels: [[SlotSymbol; 3]; 3],
    pub symbols_matched: Option<(SlotSymbol, u8)>,
    pub tier: RewardTier,
    pub payout_coins: u32,
    pub is_near_miss: bool,
    /// Which reward tier was actually given to the user from global rewards.
    pub reward_tier_given: Option<RewardTier>,
    /// Human-readable note describing the reward (e.g. "Claimed: Coffee Break").
    pub reward_note: String,
}

#[derive(Debug, Clone)]
pub struct StreakData {
    pub current_streak_days: u32,
    pub max_streak_days: u32,
    pub last_completed_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Completion {
    pub habit_id: Uuid,
    pub date: NaiveDate,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionKind {
    Earn(u32),
    Spend(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub kind: TransactionKind,
    pub amount: i64,
    pub balance_after: i64,
    pub note: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoinBalance {
    pub balance: i64,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PityCounter {
    pub consecutive_losses: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub habit_id: Uuid,
    pub target_days: u32,
    pub target_completions: Option<u32>,
    pub claimed: bool,
    pub reward_tier: RewardTier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarColor {
    Empty,
    Low,
    Mid,
    High,
}

impl CalendarColor {
    pub fn hex(&self) -> &'static str {
        match self {
            CalendarColor::Empty => "#0f0520",
            CalendarColor::Low => "#2a1a4e",
            CalendarColor::Mid => "#ff2d78",
            CalendarColor::High => "#00f5d4",
        }
    }
}

/// Tier for a global reward item in the Rewards page.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlobalRewardTier {
    #[default]
    Low,
    Medium,
    Jackpot,
}

/// A reward item available on the global Rewards page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalReward {
    pub id: Uuid,
    pub name: String,
    pub tier: GlobalRewardTier,
}

/// Represents a global reward that was claimed from a slot spin result.
#[derive(Debug, Clone, PartialEq)]
pub struct ClaimedReward {
    pub name: String,
    pub tier: GlobalRewardTier,
}

/// Toast notification message displayed at top-center of screen.
#[derive(Debug, Clone, PartialEq)]
pub struct ToastMessage {
    pub symbol_name: String,
    pub payout: u32,
    pub created_at: Instant,
}

/// Auto-dismiss timeout for toast notifications.
pub const TOAST_TIMEOUT_MS: u64 = 2500;

/// Manages a queue of toast notifications with FIFO ordering and auto-dismiss.
#[derive(Debug, Clone, Default)]
pub struct ToastManager {
    pub toasts: Vec<ToastMessage>,
}

impl ToastManager {
    /// Push a new toast notification to the end of the queue (FIFO).
    pub fn push(&mut self, symbol_name: String, payout: u32) {
        self.toasts.push(ToastMessage {
            symbol_name,
            payout,
            created_at: Instant::now(),
        });
    }

    /// Remove toasts older than the configured timeout. Called each render frame.
    pub fn dismiss_expired(&mut self) {
        let now = Instant::now();
        self.toasts
            .retain(|t| now.duration_since(t.created_at) < Duration::from_millis(TOAST_TIMEOUT_MS));
    }
}

#[cfg(test)]
mod toast_tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn push_adds_entry() {
        let mut mgr = ToastManager::default();
        assert!(mgr.toasts.is_empty());

        mgr.push("Low0 x3".to_string(), 25);
        assert_eq!(mgr.toasts.len(), 1);
        assert_eq!(mgr.toasts[0].symbol_name, "Low0 x3");
        assert_eq!(mgr.toasts[0].payout, 25);
    }

    #[test]
    fn push_fifo_ordering() {
        let mut mgr = ToastManager::default();

        mgr.push("Low0 x3".to_string(), 2);
        mgr.push("Mid0 x3".to_string(), 8);
        mgr.push("High0 x3".to_string(), 50);

        assert_eq!(mgr.toasts.len(), 3);
        assert_eq!(mgr.toasts[0].symbol_name, "Low0 x3");
        assert_eq!(mgr.toasts[1].symbol_name, "Mid0 x3");
        assert_eq!(mgr.toasts[2].symbol_name, "High0 x3");
    }

    #[test]
    fn dismiss_expired_removes_old() {
        let mut mgr = ToastManager::default();
        mgr.push("Low0 x3".to_string(), 2);

        let old = Instant::now().checked_sub(Duration::from_secs(10)).unwrap();
        mgr.toasts[0].created_at = old;

        mgr.dismiss_expired();
        assert!(mgr.toasts.is_empty());
    }

    #[test]
    fn dismiss_expired_keeps_recent() {
        let mut mgr = ToastManager::default();
        mgr.push("Mid0 x3".to_string(), 8);

        // Created just now, within timeout — should not be removed
        mgr.dismiss_expired();
        assert_eq!(mgr.toasts.len(), 1);
    }

    #[test]
    fn dismiss_expired_removes_only_old() {
        let mut mgr = ToastManager::default();

        // First toast is old
        mgr.push("Low0 x3".to_string(), 2);
        let old = Instant::now().checked_sub(Duration::from_secs(10)).unwrap();
        mgr.toasts[0].created_at = old;

        // Second toast is recent
        mgr.push("High0 x3".to_string(), 50);

        mgr.dismiss_expired();
        assert_eq!(mgr.toasts.len(), 1);
        assert_eq!(mgr.toasts[0].symbol_name, "High0 x3");
    }

    #[test]
    fn claimed_reward_construction() {
        let reward = ClaimedReward {
            name: "Coffee Break".to_string(),
            tier: GlobalRewardTier::Low,
        };
        assert_eq!(reward.name, "Coffee Break");
        assert_eq!(reward.tier, GlobalRewardTier::Low);
    }

    #[test]
    fn claimed_reward_clone_and_partial_eq() {
        let r1 = ClaimedReward {
            name: "Day Off".to_string(),
            tier: GlobalRewardTier::Medium,
        };
        let r2 = r1.clone();
        assert_eq!(r1, r2);
    }

    #[test]
    fn spin_result_new_fields_defaults() {
        let result = SpinResult {
            reels: Default::default(),
            symbols_matched: None,
            tier: RewardTier::None,
            payout_coins: 0,
            is_near_miss: false,
            reward_tier_given: None,
            reward_note: String::new(),
        };
        assert!(result.reward_tier_given.is_none());
        assert_eq!(result.reward_note, "");
    }

    #[test]
    fn spin_result_with_reward_fields() {
        let result = SpinResult {
            reels: Default::default(),
            symbols_matched: None,
            tier: RewardTier::Small,
            payout_coins: 0,
            is_near_miss: false,
            reward_tier_given: Some(RewardTier::Small),
            reward_note: "Claimed: Coffee Break".to_string(),
        };
        assert_eq!(result.reward_tier_given, Some(RewardTier::Small));
        assert_eq!(result.reward_note, "Claimed: Coffee Break");
    }
}

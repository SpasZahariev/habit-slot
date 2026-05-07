use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotSymbol {
    #[default]
    Cherry,
    Bell,
    Diamond,
    Seven,
    Devil,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RewardTier {
    None,
    Small,
    Medium,
    Jackpot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Habit {
    pub id: Uuid,
    pub name: String,
    pub created_at: NaiveDate,
    pub reward_pool: RewardPool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone)]
pub struct SpinResult {
    pub reels: [[SlotSymbol; 3]; 3],
    pub symbols_matched: Option<(SlotSymbol, u8)>,
    pub tier: RewardTier,
    pub payout_coins: u32,
    pub is_near_miss: bool,
    pub grayed_high_tier: bool,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
            CalendarColor::Empty => "#1a1a2e",
            CalendarColor::Low => "#16213e",
            CalendarColor::Mid => "#e94560",
            CalendarColor::High => "#f5c518",
        }
    }
}

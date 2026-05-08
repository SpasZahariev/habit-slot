use std::collections::HashMap;

use dioxus::prelude::*;
use uuid::Uuid;

use habit_slot::economy;
use habit_slot::models::{
    CoinBalance, Completion, Habit, PityCounter, RewardPool, SpinResult, StreakData,
};
use habit_slot::rewards::{self, MilestoneTracker};
use habit_slot::slot;
use habit_slot::streaks;

/// Top-level application state held in a Dioxus signal.
#[derive(Clone)]
pub struct AppState {
    pub habits: Vec<Habit>,
    pub completions: Vec<Completion>,
    pub coin_balance: CoinBalance,
    pub pity_counter: PityCounter,
    pub last_spin_result: Option<SpinResult>,
    /// Milestone tracker per habit.
    pub milestone_trackers: HashMap<Uuid, MilestoneTracker>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            habits: vec![],
            completions: vec![],
            coin_balance: CoinBalance::default(),
            pity_counter: PityCounter::default(),
            last_spin_result: None,
            milestone_trackers: HashMap::new(),
        }
    }
}

impl AppState {
    pub fn add_habit(&mut self, name: String) {
        let habit = Habit {
            id: Uuid::new_v4(),
            name,
            created_at: chrono::Utc::now().naive_utc().date(),
            reward_pool: RewardPool::default(),
        };
        let id = habit.id;
        self.milestone_trackers
            .insert(id, MilestoneTracker::default());
        self.habits.push(habit);
    }

    pub fn remove_habit(&mut self, id: Uuid) {
        self.habits.retain(|h| h.id != id);
        self.milestone_trackers.remove(&id);
    }

    /// Toggle completion for a habit today. Returns true if just completed (was pending).
    pub fn toggle_completion(&mut self, habit_id: Uuid) -> bool {
        let today = chrono::Utc::now().naive_utc().date();
        let already_done = self
            .completions
            .iter()
            .any(|c| c.habit_id == habit_id && c.date == today);

        if already_done {
            // Undo completion — remove today's entry
            self.completions
                .retain(|c| !(c.habit_id == habit_id && c.date == today));
            false
        } else {
            // Complete: record and award coins
            let streak = streaks::compute_streak(habit_id, &self.completions);
            let new_streak = if streak.current_streak_days == 0 {
                1
            } else {
                streak.current_streak_days + 1
            };

            self.completions.push(Completion {
                habit_id,
                date: today,
            });

            economy::on_complete(&mut self.coin_balance, new_streak);

            // Check milestones
            let total_completions = self
                .completions
                .iter()
                .filter(|c| c.habit_id == habit_id)
                .count() as u32;

            if let Some(tracker) = self.milestone_trackers.get_mut(&habit_id) {
                let milestone_result =
                    rewards::check_milestones(tracker, new_streak, total_completions);
                if let Some(kind) = milestone_result.newly_claimed {
                    // Award bonus coins for milestone claim based on tier
                    let tier = rewards::get_milestone_tier(kind);
                    let bonus = match tier {
                        habit_slot::models::RewardTier::Small => 5,
                        habit_slot::models::RewardTier::Medium => 10,
                        habit_slot::models::RewardTier::Jackpot => 25,
                        habit_slot::models::RewardTier::None => 0,
                    };
                    if bonus > 0 {
                        economy::earn(
                            &mut self.coin_balance,
                            bonus as u32,
                            format!("Milestone: {:?}", kind),
                        );
                    }
                }
            }

            true
        }
    }

    /// Get streak data for a habit.
    pub fn get_streak(&self, habit_id: Uuid) -> StreakData {
        streaks::compute_streak(habit_id, &self.completions)
    }

    /// Check milestone progress for a habit without claiming (for UI display).
    pub fn get_milestone_progress(&self, habit_id: Uuid) -> rewards::MilestoneCheckResult {
        let tracker = self
            .milestone_trackers
            .get(&habit_id)
            .cloned()
            .unwrap_or_default();
        let streak = self.get_streak(habit_id);
        let total_completions = self
            .completions
            .iter()
            .filter(|c| c.habit_id == habit_id)
            .count() as u32;

        rewards::check_milestones(&mut tracker, streak.current_streak_days, total_completions)
    }

    /// Check if a habit is completed today.
    pub fn is_completed_today(&self, habit_id: Uuid) -> bool {
        let today = chrono::Utc::now().naive_utc().date();
        self.completions
            .iter()
            .any(|c| c.habit_id == habit_id && c.date == today)
    }

    /// Spend coins for a slot spin bet. Returns true if successful.
    pub fn spend_coins(&mut self, amount: u32, note: String) -> bool {
        economy::spend(&mut self.coin_balance, amount, note)
    }

    /// Credit coins to balance after winning spin.
    pub fn credit_coins(&mut self, amount: u32, note: String) {
        economy::earn(&mut self.coin_balance, amount, note);
    }

    /// Execute a slot spin with integrated pity tracking and economy.
    /// Deducts bet, resolves spin, credits winnings. Returns the SpinResult.
    pub fn execute_spin(&mut self, bet: u32) -> Option<SpinResult> {
        if !economy::spend(&mut self.coin_balance, bet, format!("Bet {} coins", bet)) {
            return None;
        }

        let mut losses = self.pity_counter.consecutive_losses;
        let result = slot::spin_with_state(&mut losses, bet);
        self.pity_counter.consecutive_losses = losses;

        if result.payout_coins > 0 {
            economy::earn(
                &mut self.coin_balance,
                result.payout_coins,
                format!("Slot win: {:?}", result.symbols_matched.map(|(s, _)| s)),
            );
        }

        self.last_spin_result = Some(result.clone());
        self.last_spin_result.clone()
    }
}

/// Create a new writable signal for the app state.
pub fn use_app_state() -> Signal<AppState> {
    use_signal(|| AppState::default())
}

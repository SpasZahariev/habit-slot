use dioxus::prelude::*;
use uuid::Uuid;

use habit_slot::economy;
use habit_slot::models::{CoinBalance, Completion, Habit, PityCounter, RewardPool, StreakData};
use habit_slot::streaks;

/// Top-level application state held in a Dioxus signal.
#[derive(Clone)]
pub struct AppState {
    pub habits: Vec<Habit>,
    pub completions: Vec<Completion>,
    pub coin_balance: CoinBalance,
    pub pity_counter: PityCounter,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            habits: vec![],
            completions: vec![],
            coin_balance: CoinBalance::default(),
            pity_counter: PityCounter::default(),
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
        self.habits.push(habit);
    }

    pub fn remove_habit(&mut self, id: Uuid) {
        self.habits.retain(|h| h.id != id);
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
            true
        }
    }

    /// Get streak data for a habit.
    pub fn get_streak(&self, habit_id: Uuid) -> StreakData {
        streaks::compute_streak(habit_id, &self.completions)
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
}

/// Create a new writable signal for the app state.
pub fn use_app_state() -> Signal<AppState> {
    use_signal(|| AppState::default())
}

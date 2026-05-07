//! Application state management
//! Dioxus signal-based state for the habit slot application.

use dioxus::prelude::*;
use uuid::Uuid;

use habit_slot::models::{Habit, RewardPool, Transaction};

/// Soul coin balance tracked as a signed value to allow auditability.
#[derive(Clone, Default)]
pub struct CoinBalance {
    pub balance: i64,
    pub transactions: Vec<Transaction>,
}

/// Pity mechanic counter: tracks consecutive losses for guaranteed small win.
#[derive(Clone, Copy, Default)]
pub struct PityCounter {
    pub consecutive_losses: u32,
}

/// Top-level application state held in a Dioxus signal.
#[derive(Clone)]
pub struct AppState {
    pub habits: Vec<Habit>,
    pub coin_balance: CoinBalance,
    pub pity_counter: PityCounter,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            habits: vec![],
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
}

/// Create a new writable signal for the app state.
pub fn use_app_state() -> Signal<AppState> {
    use_signal(|| AppState::default())
}

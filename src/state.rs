//! Application state management
//! Dioxus signal-based state for the habit slot application.

use dioxus::prelude::*;
use std::rc::Rc;

use crate::models::{Habit, Transaction};

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
    pub habits: Rc<Vec<Habit>>,
    pub coin_balance: Rc<CoinBalance>,
    pub pity_counter: PityCounter,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            habits: Rc::new(vec![]),
            coin_balance: Rc::new(CoinBalance::default()),
            pity_counter: PityCounter::default(),
        }
    }
}

/// Create a new writable signal for the app state.
pub fn use_app_state() -> Signal<AppState> {
    use_signal(|| AppState::default())
}

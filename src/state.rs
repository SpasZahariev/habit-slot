use std::collections::HashMap;
#[cfg(feature = "db")]
use std::rc::Rc;

use dioxus::prelude::*;
use uuid::Uuid;

use habit_slot::economy;
use habit_slot::models::{
    CoinBalance, Completion, GlobalReward, Habit, PityCounter, RewardPool, SlotSymbol, SpinResult,
    StreakData, ToastMessage,
};
use habit_slot::rewards::{self, MilestoneTracker};
use habit_slot::slot;
use habit_slot::streaks;

/// Page navigation enum for simple signal-based routing.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Page {
    #[default]
    Home,
    SlotMachine,
    Habits,
    CreateHabit,
    Rewards,
}

use std::time::{Duration, Instant};

/// Top-level application state held in a Dioxus signal.
#[derive(Clone)]
pub struct AppState {
    pub current_page: Page,
    pub habits: Vec<Habit>,
    pub completions: Vec<Completion>,
    pub coin_balance: CoinBalance,
    pub pity_counter: PityCounter,
    pub last_spin_result: Option<SpinResult>,
    /// Milestone tracker per habit.
    pub milestone_trackers: HashMap<Uuid, MilestoneTracker>,
    /// Active toast notifications queued FIFO.
    pub toasts: Vec<ToastMessage>,
    /// Global rewards available on the Rewards page.
    pub global_rewards: Vec<GlobalReward>,
    /// Reels are currently animating.
    pub is_spinning: bool,
    /// Animation strips for each reel column during spin. Each strip contains filler + result symbols.
    pub animation_strips: Option<[Vec<SlotSymbol>; 3]>,
    /// Number of reels that have finished their staggered animation (0..3).
    pub reels_stopped: u8,
    #[cfg(feature = "db")]
    pub db: Option<Rc<habit_slot::db::Db>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_page: Page::default(),
            habits: vec![],
            completions: vec![],
            coin_balance: CoinBalance::default(),
            pity_counter: PityCounter::default(),
            last_spin_result: None,
            milestone_trackers: HashMap::new(),
            toasts: vec![],
            is_spinning: false,
            animation_strips: None,
            reels_stopped: 0,
            global_rewards: vec![],
            #[cfg(feature = "db")]
            db: None,
        }
    }
}

#[cfg(feature = "db")]
impl AppState {
    /// Load all state from the database. Returns None if loading fails.
    pub fn from_db(db: &habit_slot::db::Db) -> Option<Self> {
        let habits = db.load_habits().ok()?;
        let completions = db.load_completions().ok()?;
        let coin_balance = db.load_coin_balance().ok()?;
        let pity_losses = db.load_pity_counter().ok()?.or_default();

        let mut milestone_trackers = HashMap::new();
        for habit in &habits {
            if let Ok(tracker) = db.load_milestone_tracker(habit.id) {
                milestone_trackers.insert(habit.id, tracker);
            }
        }

        Some(Self {
            current_page: Page::default(),
            habits,
            completions,
            coin_balance,
            pity_counter: PityCounter {
                consecutive_losses: pity_losses,
            },
            last_spin_result: None,
            milestone_trackers,
            toasts: vec![],
            is_spinning: false,
            animation_strips: None,
            reels_stopped: 0,
            global_rewards: db.load_global_rewards().ok().unwrap_or_default(),
            db: None,
        })
    }

    /// Attach a shared DB reference for write-through persistence.
    pub fn with_db(mut self, db: Rc<habit_slot::db::Db>) -> Self {
        self.db = Some(db);
        self
    }

    /// Persist new transactions to DB from the given index onward.
    fn persist_new_transactions(&self, from_index: usize) {
        if let Some(db) = &self.db {
            for tx in self.coin_balance.transactions.iter().skip(from_index) {
                let _ = db.insert_transaction(tx);
            }
        }
    }

    /// Persist the current pity counter to DB.
    fn persist_pity_counter(&self) {
        if let Some(db) = &self.db {
            let _ = db.save_pity_counter(self.pity_counter.consecutive_losses);
        }
    }

    /// Persist milestone tracker for a habit to DB.
    fn persist_milestone_tracker(&self, habit_id: Uuid) {
        if let Some(db) = &self.db {
            if let Some(tracker) = self.milestone_trackers.get(&habit_id) {
                let _ = db.save_milestone_tracker(habit_id, tracker);
            }
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
        let _created_at = habit.created_at;
        self.milestone_trackers
            .insert(id, MilestoneTracker::default());
        self.habits.push(habit);

        #[cfg(feature = "db")]
        if let Some(db) = &self.db {
            let _ = db.insert_habit(id, &self.habits.last().unwrap().name, created_at);
        }
    }

    pub fn remove_habit(&mut self, id: Uuid) {
        #[cfg(feature = "db")]
        if let Some(db) = &self.db {
            let _ = db.delete_habit(id);
        }

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
            #[cfg(feature = "db")]
            if let Some(db) = &self.db {
                let _ = db.delete_completion(habit_id, today);
            }
            self.completions
                .retain(|c| !(c.habit_id == habit_id && c.date == today));
            false
        } else {
            // Complete: record and award coins
            #[cfg(feature = "db")]
            if let Some(db) = &self.db {
                let _ = db.insert_completion(habit_id, today);
            }

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

            let _tx_count_before = self.coin_balance.transactions.len();
            economy::on_complete(&mut self.coin_balance, new_streak);
            #[cfg(feature = "db")]
            self.persist_new_transactions(tx_count_before);

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
                #[cfg(feature = "db")]
                self.persist_milestone_tracker(habit_id);
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
        let mut tracker = self
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

    /// Navigate to a page.
    pub fn navigate(&mut self, page: Page) {
        self.current_page = page;
    }

    /// Navigate back to home.
    pub fn go_home(&mut self) {
        self.current_page = Page::Home;
    }

    /// Check if a habit is completed today.
    pub fn is_completed_today(&self, habit_id: Uuid) -> bool {
        let today = chrono::Utc::now().naive_utc().date();
        self.completions
            .iter()
            .any(|c| c.habit_id == habit_id && c.date == today)
    }

    /// Push a new toast notification to the end of the queue (FIFO).
    pub fn push_toast(&mut self, symbol_name: String, payout: u32) {
        self.toasts.push(ToastMessage {
            symbol_name,
            payout,
            created_at: Instant::now(),
        });
    }

    /// Remove toasts older than the configured timeout. Called each render frame.
    pub fn dismiss_expired_toasts(&mut self) {
        let now = Instant::now();
        self.toasts.retain(|t| {
            now.duration_since(t.created_at)
                < Duration::from_millis(habit_slot::models::TOAST_TIMEOUT_MS)
        });
    }

    /// Spend coins for a slot spin bet. Returns true if successful.
    pub fn spend_coins(&mut self, amount: u32, note: String) -> bool {
        let success = economy::spend(&mut self.coin_balance, amount, note);
        #[cfg(feature = "db")]
        if success {
            self.persist_new_transactions(self.coin_balance.transactions.len().saturating_sub(1));
        }
        success
    }

    /// Credit coins to balance after winning spin.
    pub fn credit_coins(&mut self, amount: u32, note: String) {
        economy::earn(&mut self.coin_balance, amount, note);
        #[cfg(feature = "db")]
        self.persist_new_transactions(self.coin_balance.transactions.len().saturating_sub(1));
    }

    /// Execute a slot spin with integrated pity tracking and economy.
    /// Deducts bet, resolves spin, credits winnings. Returns the SpinResult.
    /// Prepares animation strips for reel animation.
    pub fn execute_spin(&mut self, bet: u32) -> Option<SpinResult> {
        let tx_before = self.coin_balance.transactions.len();
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

        #[cfg(feature = "db")]
        {
            self.persist_new_transactions(tx_before);
            self.persist_pity_counter();
        }

        // Prepare animation strips before revealing result
        self.prepare_animation(&result);

        self.last_spin_result = Some(result.clone());
        self.last_spin_result.clone()
    }

    /// Prepare animation strips for a spin result. Called before reels start animating.
    pub fn prepare_animation(&mut self, spin_result: &SpinResult) {
        self.animation_strips = Some(slot::generate_all_animation_strips(spin_result));
        self.is_spinning = true;
        self.reels_stopped = 0;
    }

    /// Mark one reel as stopped. When all 3 are done, clear animation state.
    pub fn stop_one_reel(&mut self) {
        if self.is_spinning {
            self.reels_stopped += 1;
            if self.reels_stopped >= 3 {
                // Push toast after animation finishes (only for wins)
                if let Some(ref result) = self.last_spin_result {
                    if result.payout_coins > 0 {
                        if let Some((symbol, count)) = result.symbols_matched {
                            let symbol_name =
                                habit_slot::sprites::symbol_display_name(&symbol).to_string();
                            self.push_toast(
                                format!("{} x{}", symbol_name, count),
                                result.payout_coins,
                            );
                        }
                    }
                }
                self.is_spinning = false;
                self.animation_strips = None;
                self.reels_stopped = 0;
            }
        }
    }

    /// Reset animation state (e.g., on new spin or cancel).
    pub fn reset_animation(&mut self) {
        self.is_spinning = false;
        self.animation_strips = None;
        self.reels_stopped = 0;
    }
}

/// Create a new writable signal for the app state.
pub fn use_app_state() -> Signal<AppState> {
    let mut s = AppState::default();
    // TEMP: give 7 coins for testing, remove before release
    s.coin_balance.balance = 7;
    use_signal(move || s.clone())
}

/// Create app state backed by a SQLite database (requires `db` feature).
#[cfg(feature = "db")]
pub fn use_app_state_with_db(db: Rc<habit_slot::db::Db>) -> Signal<AppState> {
    use_signal(|| AppState::from_db(&db).unwrap_or_default().with_db(db))
}

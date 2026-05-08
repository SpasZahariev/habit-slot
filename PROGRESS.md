## Issue #2: Project scaffolding and compile baseline ✓ DONE

- Fixed Dioxus launch pattern in `src/main.rs` (`dioxus::web::launch` → `launch` from prelude)
- `cargo check` now passes with no errors (only dead code warnings on stub modules)
- Added `.gitignore` to exclude `target/`
- All module stubs registered: slot, economy, streaks, rewards, components ✓
- state.rs has basic signal infrastructure ✓
- models.rs has all key types ✓

## Issue #3: Core slot engine and probability table ✓ DONE

- Implemented `spin(bet) -> SpinResult` with configurable symbol probability weights
- 3-of-a-kind payline matching across top/middle/bottom rows (reels[col][row] layout)
- Payout scales linearly with bet: Cherry=2x, Bell=5x, Diamond=10x, Seven=25x, Devil=50x
- RewardTier mapping: Small (Cherry/Bell), Medium (Diamond/Seven), Jackpot (Devil)
- 7 tests all pass: symbol distribution, tier mapping, payout scaling, 10k-spin validation
- Made dioxus optional dep so lib tests run without GTK deps on NixOS
- Created `src/lib.rs` for library target separation from binary/UI code

## Issue #4: Habit creation and list display ✓ DONE

- HabitForm component: text input + submit button, validates non-empty trimmed name
- HabitList component: displays habits with name, creation date (YYYY-MM-DD), delete button
- Empty state message when no habits exist
- AppState manages Vec<Habit> via Dioxus signals (use_app_state)
- add_habit() assigns UUID v4, current date, default RewardPool
- remove_habit() filters by UUID

## Issue #5: Habit completion tracking with streaks and coin earning ✓ DONE

- StreakCalculator (`src/streaks/mod.rs`): consecutive day counting, hard reset on missed days, max streak tracking
- CalendarColor enum with hex values for heatmap intensity mapping (1-3=Low, 4-9=Mid, 10+=High)
- Coin Economy (`src/economy/mod.rs`): balance tracking, earn/spend operations, immutable transaction log
- Streak bonus: +1 bonus coin per 7 consecutive days streak
- AppState integration: `toggle_completion()` records completion + awards coins via economy::on_complete()
- Habit list UI shows completion toggle ("Do it" / "Done") with visual state, streak count per habit
- Coin balance displayed prominently in main.rs (top of app)
- 15 tests across streaks (8) and economy (7): all pass

## Issue #6: Pity mechanic and near-miss programming ✓ DONE

- Pity mechanic: hidden loss counter increments on each losing spin, resets on any win
- On 5th consecutive loss, next spin guaranteed to land at least Small tier reward via `generate_pity_reels()`
- `spin_with_state(&mut consecutive_losses, bet)` for persistent pity tracking across spins
- Near-miss detection: `has_near_miss_pattern()` checks all paylines for 2+ matching symbols
- `is_near_miss` flag set only on losing spins with near-miss pattern (no impact on payout)
- PITY_THRESHOLD constant = 5 configurable at module level
- 11 new tests: pity trigger on 5th loss, reset after win, near-miss detection patterns, guaranteed pity wins

## Issue #7: Slot UI with bet selection and reel animation ✓ DONE

- `SlotMachine` component in `src/components/slot_machine.rs`
- BetSelector: 3-button toggle for 1/2/3 coin bets
- Reels display: 3x3 grid showing slot symbols (emoji: 🍒🔔💎7️⃣😈) from SpinResult
- SPIN button: calls `AppState.execute_spin(bet)` — deducts bet, resolves spin with pity tracking, credits winnings
- Button disabled when balance insufficient for selected bet
- Result display shows win amount ("Win! +N coins"), near-miss ("So close..."), or loss feedback
- `AppState.execute_spin()`: integrated method chaining spend → spin_with_state → earn winnings
- `AppState.last_spin_result` tracks latest outcome for reactive UI updates
- Coin balance updates in real-time (reflected in existing coin balance display)

## Issue #8: Calendar heatmap per habit ✓ DONE

- `CalendarHeatmap` component (`src/components/calendar_heatmap.rs`) with monthly grid view
- Month navigation (previous/next buttons) with year rollover handling
- Day cells colored by streak intensity at that point: Empty (gray), Low 1-3 (blue), Mid 4-9 (red), High 10+ (gold)
- Today highlighted with gold border in calendar grid
- Legend showing color mapping below the grid
- Expandable per habit via "Calendar"/"Hide" toggle button in HabitItem
- Uses existing `streaks::calendar_color()` function for per-day streak computation
- No new tests needed — relies on existing streaks test coverage for calendar_color logic

## Issue #9: Milestone tracking with claim-once rewards ✓ DONE

- `MilestoneTracker` in `src/rewards/mod.rs`: tracks claimed streak and completion tiers per habit via HashSet<usize>
- Two milestone tracks: STREAK_MILESTONES (7, 14, 30, 60, 90 days) and COMPLETION_MILESTONES (10, 25, 50, 100, 200 completions)
- `check_milestones()` returns newly claimed milestone + next active goals for both tracks
- Claim-once semantics: each tier index inserted into HashSet on claim, prevents duplicate rewards
- After claim, next tier automatically becomes the active goal via `next_streak_goal`/`next_completion_goal`
- `select_reward()`: random selection from appropriate RewardPool tier; returns None for empty pools
- AppState integration: `toggle_completion()` checks milestones after each completion, awards bonus coins (Small=5, Medium=10, Jackpot=25)
- UI display: HabitItem shows "Streak: X/Y | Tasks: A/B" progress line beneath habit name
- 11 new tests all pass: claim-once, tier advancement, empty pool, reward selection, tier mapping, format_progress

## Issue #11: Grayed-out high-tier symbols on low bets ✓ DONE

- `MAX_BET` constant (3) in `src/slot/mod.rs` — defines the threshold for full-color payouts
- `grayed_high_tier` flag on `SpinResult`: true when matched symbol is high-tier (Diamond+, tier order >= 2) AND bet < MAX_BET
- Reduced payout formula: `round(payout * bet/MAX_BET)` — proportional reduction creates FOMO at lower bets
- At max bet (3 coins): all symbols display full color with full multiplier, no graying applied
- `resolve_reels()` helper for testing grayed behavior with controlled reel data
- SlotMachine UI: winning cells rendered with CSS `grayscale(100%) brightness(50%)` when `grayed_high_tier` is true, plus "(Bet more for full payout)" hint text
- 5 new tests all pass (49 total): grayed at bet 1, grayed at bet 2, no gray at bet 3, low-tier not grayed, only matching symbols affected

## Issue #10: SQLite persistence layer — IN PROGRESS

### DB Module ✓ Done
- `rusqlite` with `bundled` feature as optional dependency behind `db` feature flag
- `Db` struct in `src/db.rs` with schema initialization and migration system
- Schema version tracking via metadata table (current: v1)
- 6 tables: `metadata`, `habits`, `completions`, `coin_balance`, `transactions`, `milestones`, `pity_counter`
- All queries parameterized — no raw SQL concatenation
- CRUD round-trips for all tables tested with in-memory DB + file persistence
- Graceful degradation: error handling returns defaults, app stays functional on read failures
- Migration upgrade path: version comparison at startup, auto-updates if behind
- 9 new tests (58 total with --features db): schema creation, habit CRUD, completion CRUD, coin balance persistence, transaction immutability, close/reopen persistence, milestone tracker CRUD, pity counter CRUD, migration version

### AppState Wiring ✓ Done
- `AppState.from_db(&Db) -> Option<Self>`: loads habits, completions, coin balance (from transactions), pity counter, milestone trackers
- `AppState.with_db(Rc<Db>) -> Self`: attaches shared DB reference for write-through
- All mutations persist to DB when `db` feature enabled:
  - `add_habit()` → `db.insert_habit()` + auto-creates milestone row
  - `remove_habit()` → `db.delete_habit()` (cascades completions + milestones)
  - `toggle_completion()` → `db.insert_completion()` / `db.delete_completion()` + persists new transactions + saves milestone tracker
  - `execute_spin()` → persists bet + win transactions + saves pity counter
- `use_app_state_with_db(Rc<Db>) -> Signal<AppState>`: Dioxus hook for DB-backed initialization
- Graceful degradation: all DB calls wrapped with `let _ = ...` — failures are logged silently, app stays functional without DB
- Both `cargo check` (no feature) and `cargo test --features db` (58 tests) pass

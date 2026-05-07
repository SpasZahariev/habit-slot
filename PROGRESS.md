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

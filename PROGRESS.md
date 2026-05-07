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

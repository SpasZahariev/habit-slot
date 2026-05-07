## Problem Statement

Building good habits is difficult because the reward is delayed and abstract. Traditional habit trackers rely on willpower and linear progress bars, which don't trigger the dopamine response needed for long-term adherence. Users need a habit tracker that makes daily completion feel exciting, unpredictable,and emotionally engaging — similar to the variable-ratio reinforcement schedule that makes slot machines addictive.

## Solution

A casino-themed mobile app where completing habits earns "soul coins." Soul coins are spent spinning a 3-reel slot machine with variable rewards. The unpredictability of the reward (sometimes nothing, sometimes small, rarely big) creates the "maybe effect" — the psychological hook that keeps users coming back daily. A vintage noir casino aesthetic and a "deal with the Devil" narrative frame the experience.

## User Stories

1. As a user, I want to create a habit with a name, so that I can track it
2. As a user, I want to mark a habit as completed for today, so that I earn rewards
3. As a user, I want to earn 1 soul coin per habit completion, so that I can spend them on spins
4. As a user, I want to earn bonus soul coins for maintaining streaks, so that consistency is rewarded
5. As a user, I want to see my total soul coin balance at all times, so that I know my spending power
6. As a user, I want to spin the slot machine by betting 1, 2, or 3 coins per spin, so that I can control my risk
7. As a user, I want to see 3 reels animate with symbols (Cherry, Bell, Diamond, Seven, Devil), so that the spin feels exciting
8. As a user, I want lower bets to still land high-tier symbols but pay out less (grayed out), so that I feel FOMO and am motivated to bet more
9. As a user, I want guaranteed small wins after 5 consecutive losses (pity mechanic), so that I don't feel completely cheated
10. As a user, I want to see a calendar view per habit with color intensity based on streak length, so that I can visualize my progress
11. As a user, I want streaks to reset if I miss a day, so that consistency matters
12. As a user, I want to hear pre-recorded sound effects during spins and wins, so that the experience is immersive
13. As a user, I want all data persisted in SQLite on-device, so that my progress survives app restarts
14. As a user, I want the app to work offline, so that I can use it anywhere
15. As a user, I want to see my current streak count displayed prominently, so that I know how many days I've maintained
16. As a user, I want milestone tracking (N days, N completions) to reset after one-time unlock, so that rewards are earned once
17. As a user, I want the visual theme to be vintage casino noir with sepia tones, gold accents, and dark wood textures, so that the aesthetic matches the Devil narrative
18. As a user, I want the app optimized for portrait mode only on mobile, so that the experience is focused and immersive

## Implementation Decisions

### Modules

**1. Slot Engine** (`src/slot/`)
- Pure Rust logic for spin resolution: reel generation, symbol matching, win calculation
- Configurable probability table per symbol tier (Cherry=common, Seven=rare/jackpot)
- Pity mechanic tracking: hidden counter guarantees small win after 5 consecutive losses
- Near-miss programming: intentional near-win patterns on losing spins to increase engagement
- Bet multiplier logic: higher bets scale payouts; lower bets landing high-tier symbols show grayed icons with reduced payout

**2. Coin Economy** (`src/economy/`)
- Soul coin ledger: balance tracking, earn/spend operations
- Earning rules: 1 coin per habit completion + streak bonus (configurable formula)
- Spending rules: validation against available balance for 1/2/3 coin bets
- Immutable transaction log for auditability

**3. Streak Calculator** (`src/streaks/`)
- Current streak computation from completion history (consecutive calendar days including today)
- Max streak tracking per habit
- Calendar heatmap data: color intensity mapping from streak length (e.g., 1-3=green, 4-9=orange, 10+=red)
- Hard reset on missed days

**4. Reward Pool Resolver** (`src/rewards/`)
- Tier-based reward pools: small, medium, jackpot — each with configurable items defined at habit creation
- Random selection within tier based on slot outcome
- Milestone resolver: tracks N-day and N-completion goals, marks as claimed once, then resets

**5. Database Layer** (`src/db.rs`)
- SQLite schema via `wasm-sqlite`: habits, completions, coins_balance, transactions, milestones, pity_counter
- Migration system for schema evolution
- All queries parameterized, no raw SQL concatenation

### Architecture

- Dioxus compiling to native Android and iOS apps via its built-in platform WebView targets — no Capacitor or third-party wrapper needed
- Mobile-first, portrait-only layout
- Module-separated structure: `src/components/`, `src/slot/`, `src/economy/`, `src/streaks/`, `src/rewards/`, `src/db.rs`, `src/state.rs`, `src/models.rs`
- `wasm-sqlite` for client-side persistence — no server required
- Pre-recorded SFX (WAV/OGG) triggered via Dioxus effects on spin states

### Key Types

```rust
pub enum SlotSymbol { Cherry, Bell, Diamond, Seven, Devil }

pub struct SpinResult {
    pub reels: [[SlotSymbol; 3]; 3],
    pub symbols_matched: Option<(SlotSymbol, u8)>,
    pub tier: RewardTier,
    pub payout_coins: u32,
    pub is_near_miss: bool,
}

pub enum RewardTier { None, Small, Medium, Jackpot }

pub struct Habit {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub reward_pool: RewardPool,
}

pub struct StreakData {
    pub current_streak_days: u32,
    pub max_streak_days: u32,
    pub last_completed_date: Option<NaiveDate>,
}
```

## Testing Decisions

### Philosophy
Test external behavior only — inputs and observable outputs. No private state assertions, no implementation details. Each module tested in isolation as pure logic where possible.

### Modules To Test

**1. Slot Engine Tests** — probability distribution verification (run 10k spins, assert tier frequencies within tolerance), pity mechanic triggers on 5th consecutive loss, near-miss generation on subset of losses, bet multiplier scales payout correctly, grayed-out high-tier on low bets.

**2. Coin Economy Tests** — earn on completion, streak bonus accumulation, spend validation (reject overdraw), transaction log immutability, balance consistency after mixed earn/spend sequences.

**3. Streak Calculator Tests** — consecutive day counting, hard reset on gap, calendar heatmap color mapping, max streak preservation across resets, edge cases (first completion, same-day multiple completions).

**4. Reward Pool Resolver Tests** — tier selection correctness, random within-tier selection, milestone claim-once semantics, reset after claim, empty pool handling.

**5. Database Layer Tests** — schema creation via migrations, CRUD round-trips for all tables, transaction integrity, persistence across connection lifecycle, migration upgrade path.

### Prior Art
No existing tests in codebase — this is greenfield. All 5 modules will have tests as part of MVP. Use `rstest` for parameterized tests and standard `assert!` / `assert_eq!` macros.

## Out of Scope

- PWA or standalone browser distribution
- User accounts, cloud sync, or multiplayer features
- Custom symbol art — placeholder pixel art sufficient for MVP
- Near-miss programming deep polish (basic implementation only)
- Social features, leaderboards, or sharing
- Advanced analytics or usage tracking
- Multiple currencies or complex economy mechanics

## Further Notes

- User is a novice Rust developer — code should be idiomatic but well-structured with clear module boundaries
- `dioxus-web` + `wasm-sqlite` combination requires careful async handling; use Dioxus signals for state management
- The slot animation will be CSS/DOM-based within Dioxus components, not a canvas/WebGL approach — keeps it simple and framework-native
- Sound effects will be pre-recorded assets loaded from the file system, triggered via `<audio>` elements in Dioxus

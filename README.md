# Habit Slot

A casino-themed habit tracker Android app built with **Rust** + **Dioxus**, where completing habits earns "soul coins" that are spent spinning a 3-reel slot machine for variable rewards. Leverages variable-ratio reinforcement scheduling (the same psychological mechanism behind slot machines) to make daily habit completion dopamine-triggering instead of willpower-dependent.

---

## Screenshots

<!-- Add screenshots from the Android app here -->

---

## Tech Stack

| Layer | Technology |
|---|---|
| UI Framework | [Dioxus](https://dioxuslabs.com) (Rust → native Android via built-in WebView) |
| Language | Rust 2021 edition |
| Styling | Tailwind CSS |
| Persistence | SQLite (rusqlite, bundled) — fully offline, no server |
| Build System | Dioxus CLI + Gradle for Android packaging |
| Testing | rstest for parameterized unit tests |

---

## Architecture

The app follows a clean module-separated architecture with pure logic decoupled from UI:

- **`src/slot/`** — Slot engine: reel generation, symbol matching, probability tables, pity mechanic (guaranteed small win after 5 consecutive losses), near-miss programming, bet multiplier logic
- **`src/economy/`** — Soul coin ledger: balance tracking, earn/spend operations, streak bonus formula, immutable transaction log
- **`src/streaks/`** — Streak calculator: consecutive-day counting, max streak tracking, calendar heatmap color mapping, hard-reset-on-miss semantics
- **`src/rewards/`** — Reward pool resolver: tier-based pools (small/medium/jackpot), milestone claim-once-and-reset tracking
- **`src/db.rs`** — SQLite database layer with parameterized queries and migration system
- **`src/state.rs`** — Dioxus signal-based state management
- **`src/components/`** — Reusable UI components (habit list, slot machine, streak calendar, etc.)

All business logic modules are tested in isolation as pure functions — input/output contracts only, no implementation detail assertions.

---

## Key Features

- Create and track habits with custom names
- Earn 1 soul coin per completed habit + streak bonuses
- Spend coins spinning a 3-reel slot machine (bet 1/2/3 coins)
- Variable-ratio reward scheduling — unpredictable payouts create the "maybe effect"
- Pity mechanic prevents extended losing streaks
- Calendar heatmap visualization with color intensity mapped to streak length
- Fully offline — all data persisted in on-device SQLite
- Optimized for portrait mode only

---

## Project Structure

```
habit-slot/
├── src/
│   ├── main.rs          # Dioxus entry point
│   ├── lib.rs           # Library crate root
│   ├── models.rs        # Core type definitions (Habit, SpinResult, etc.)
│   ├── state.rs         # App state management via Dioxus signals
│   ├── db.rs            # SQLite persistence layer
│   ├── slot/            # Slot engine logic
│   ├── economy/         # Coin ledger logic
│   ├── streaks/         # Streak calculation logic
│   ├── rewards/         # Reward pool resolution
│   └── components/      # Dioxus UI components
├── static/              # Static assets
├── scripts/             # Build and deployment scripts
└── Cargo.toml           # Rust dependencies
```

---

## Development

### Prerequisites

- Rust (2021 edition) with `cargo`
- Dioxus CLI: `cargo install dioxus-cli`
- Android NDK + platform tools (for native builds)
- Java JDK 17+ and Android SDK (for Gradle packaging)
- `.env` file with keystore password (see `.env.example`)

### Run on Desktop

```bash
dx serve --platform desktop
```

### Build Android Release APK

```bash
make release-android
```

This single command handles the full pipeline: Dioxus bundling for `aarch64-linux-android`, icon generation, Gradle build, keystore signing, and zipalign. The signed APK is output to `release-build/`.

---

## Testing

Run the full test suite:

```bash
cargo test
```

Each business logic module (slot engine, economy, streaks, rewards) has parameterized tests verifying probability distributions, edge cases, and invariant properties.

## Issue #2: Project scaffolding and compile baseline

- Fixed Dioxus launch pattern in `src/main.rs` (`dioxus::web::launch` → `launch` from prelude)
- `cargo check` now passes with no errors (only dead code warnings on stub modules)
- Added `.gitignore` to exclude `target/`

### Remaining for issue #2
- All module stubs registered but empty: slot, economy, streaks, rewards, components
- state.rs has basic signal infrastructure ✓
- models.rs has all key types ✓

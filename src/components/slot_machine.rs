use crate::state::AppState;
use dioxus::prelude::*;
use habit_slot::models::{SlotSymbol, SpinResult};

#[component]
pub fn SlotMachine() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let bet = use_signal(|| 1u32);

    let balance = app_state.read().coin_balance.balance;
    let can_spin = balance >= bet() as i64;
    let last_result = app_state.read().last_spin_result.clone();

    rsx! {
        div {
            class: "slot-machine",
            style: "margin-top: 32px; padding: 24px; background: #0d1b2a; border-radius: 12px; border: 2px solid #f5c518; width: 100%; max-width: 500px;",

            h2 {
                style: "text-align: center; color: #f5c518; margin-bottom: 16px;",
                "Soul Slot Machine"
            }

            BetSelector { bet: bet.clone() }

            Reels { spin_result: last_result.clone() }

            div {
                style: "display: flex; justify-content: center; margin-top: 16px;",
                button {
                    class: "spin-button",
                    disabled: !can_spin,
                    onclick: move |_| {
                        app_state.with_mut(|state| {
                            let b = *bet.read();
                            if b > 0 {
                                let _ = state.execute_spin(b);
                            }
                        });
                    },
                    style: if can_spin {
                        "background: #f5c518; color: #1a1a2e; border: none; padding: 12px 40px; border-radius: 8px; font-size: 1.2rem; font-weight: bold; cursor: pointer;"
                    } else {
                        "background: #333; color: #666; border: none; padding: 12px 40px; border-radius: 8px; font-size: 1.2rem; font-weight: bold; cursor: not-allowed;"
                    },
                    "SPIN"
                }
            }

            SpinResultDisplay { spin_result: last_result }
        }
    }
}

#[component]
fn BetSelector(bet: Signal<u32>) -> Element {
    rsx! {
        div {
            class: "bet-selector",
            style: "display: flex; justify-content: center; gap: 8px; margin-bottom: 16px;",
              for amount in [1, 2, 3] {
                button {
                    onclick: move |_| {
                        *bet.write() = amount;
                    },
                    style: if *bet.read() == amount {
                        "background: #e94560; color: white; border: none; padding: 8px 20px; border-radius: 6px; cursor: pointer; font-weight: bold;"
                    } else {
                        "background: #16213e; color: #f5c518; border: 1px solid #f5c518; padding: 8px 20px; border-radius: 6px; cursor: pointer;"
                    },
                    "{coin_label(amount)}"
                }
            }
        }
    }
}

#[component]
fn Reels(spin_result: Option<SpinResult>) -> Element {
    let default_reels: [[SlotSymbol; 3]; 3] = [
        [SlotSymbol::Cherry, SlotSymbol::Bell, SlotSymbol::Diamond],
        [SlotSymbol::Seven, SlotSymbol::Cherry, SlotSymbol::Devil],
        [SlotSymbol::Bell, SlotSymbol::Diamond, SlotSymbol::Cherry],
    ];

    let reels = spin_result
        .as_ref()
        .map(|r| r.reels)
        .unwrap_or(default_reels);

    rsx! {
        div {
            class: "reels-container",
            style: "display: flex; justify-content: center; gap: 8px; padding: 16px; background: #1a1a2e; border-radius: 8px;",
            for col in 0..3 {
                ReelColumn { col, reels: reels.clone(), spin_result: spin_result.clone() }
            }
        }
    }
}

#[component]
fn ReelColumn(col: usize, reels: [[SlotSymbol; 3]; 3], spin_result: Option<SpinResult>) -> Element {
    let is_grayed = spin_result
        .as_ref()
        .map(|r| r.grayed_high_tier)
        .unwrap_or(false);

    // Determine which row contains the winning symbols (if any)
    let winning_row = spin_result.as_ref().and_then(|r| {
        if let Some((matched_symbol, _)) = r.symbols_matched {
            for row in 0..3 {
                if reels[0][row] == matched_symbol
                    && reels[1][row] == matched_symbol
                    && reels[2][row] == matched_symbol
                {
                    return Some(row);
                }
            }
        }
        None
    });

    let cells: Vec<_> = (0..3)
        .map(|row| {
            let is_winning_cell = winning_row == Some(row);
            let cell_style = if is_grayed && is_winning_cell {
                "width: 80px; height: 60px; display: flex; align-items: center; justify-content: center; background: #16213e; border-radius: 6px; font-size: 2rem; filter: grayscale(100%) brightness(50%); opacity: 0.5;"
            } else {
                "width: 80px; height: 60px; display: flex; align-items: center; justify-content: center; background: #16213e; border-radius: 6px; font-size: 2rem;"
            };
            let emoji = symbol_to_emoji(&reels[col][row]).to_string();
            ReelCell { cell_style, emoji }
        })
        .collect();

    rsx! {
        div {
            class: "reel-column",
            style: "display: flex; flex-direction: column; gap: 4px;",
            for cell in cells {
                div {
                    class: "reel-symbol",
                    style: cell.cell_style,
                    "{cell.emoji}"
                }
            }
        }
    }
}

#[component]
fn SpinResultDisplay(spin_result: Option<SpinResult>) -> Element {
    rsx! {
        div {
            class: "spin-result",
            style: "text-align: center; margin-top: 16px; min-height: 24px;",
            match spin_result {
                Some(r) => if r.payout_coins > 0 {
                    rsx! {
                        p {
                            style: "color: #f5c518; font-size: 1.3rem; font-weight: bold;",
                            "Win! +{r.payout_coins} coins"
                        }
                        if r.grayed_high_tier {
                            p {
                                style: "color: #888; font-size: 0.85rem; margin-top: 4px;",
                                "(Bet more for full payout)"
                            }
                        }
                    }
                } else if r.is_near_miss {
                    rsx! {
                        p {
                            style: "color: #e94560; font-size: 1rem;",
                            "So close..."
                        }
                    }
                } else {
                    rsx! {
                        p {
                            style: "color: #666; font-size: 0.9rem;",
                            "No luck. Try again!"
                        }
                    }
                },
                None => rsx! {},
            }
        }
    }
}

struct ReelCell {
    cell_style: &'static str,
    emoji: String,
}

fn coin_label(amount: u32) -> String {
    format!("{} coin{}", amount, if amount > 1 { "s" } else { "" })
}

fn symbol_to_emoji(symbol: &SlotSymbol) -> &'static str {
    match symbol {
        SlotSymbol::Cherry => "🍒",
        SlotSymbol::Bell => "🔔",
        SlotSymbol::Diamond => "💎",
        SlotSymbol::Seven => "7️⃣",
        SlotSymbol::Devil => "😈",
    }
}

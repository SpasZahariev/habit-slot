use crate::state::AppState;
use dioxus::prelude::*;
use habit_slot::models::{SlotSymbol, SpinResult};

#[component]
pub fn SlotMachine() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    let balance = app_state.read().coin_balance.balance;
    let can_spin = balance >= 1;
    let last_result = app_state.read().last_spin_result.clone();

    rsx! {
        div {
            class: "slot-machine mt-8 p-4 bg-[#1a0a2e] rounded-xl border-2 border-[#ff2d78] w-[96%]",

            h2 {
                class: "text-center text-[#ff2d78] mb-4 drop-shadow-[0_0_8px_rgba(255,45,120,0.4)]",
                "Slot Machine"
            }

            Reels { spin_result: last_result.clone() }

            div {
                class: "flex justify-center mt-4",
                button {
                    class: format!("spin-button border-none rounded-lg cursor-pointer px-10 py-3 text-xl font-bold {}", if can_spin { "bg-[#ff2d78] text-[#f0e6ff] shadow-[0_0_15px_rgba(255,45,120,0.5)]" } else { "bg-[#3a2a5e] text-gray-600 cursor-not-allowed" }),
                    disabled: !can_spin,
                    onclick: move |_| {
                        app_state.with_mut(|state| {
                            let _ = state.execute_spin(1);
                        });
                    },
                    "SPIN"
                }
            }

            SpinResultDisplay { spin_result: last_result }
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
            class: "reels-container flex justify-center gap-2 p-4 bg-[#0f0520] rounded-lg",
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
            let cell_class = if is_grayed && is_winning_cell {
                "min-w-[70px] h-[50px] flex items-center justify-center bg-[#2a1a4e] rounded-md text-3xl grayscale brightness-50 opacity-50"
            } else {
                "min-w-[70px] h-[50px] flex items-center justify-center bg-[#2a1a4e] rounded-md text-3xl"
            };
            let emoji = symbol_to_emoji(&reels[col][row]).to_string();
            ReelCell { cell_class, emoji }
        })
        .collect();

    rsx! {
        div {
            class: "reel-column flex flex-col gap-1",
            for cell in cells {
                div {
                    class: format!("reel-symbol {}", cell.cell_class),
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
            class: "spin-result text-center mt-4 min-h-[24px]",
            match spin_result {
                Some(r) => if r.payout_coins > 0 {
                    rsx! {
                        p {
                            class: "text-[#00f5d4] text-xl font-bold",
                            "Win! +{r.payout_coins} coins"
                        }
                        if r.grayed_high_tier {
                            p {
                                class: "text-gray-500 text-sm mt-1",
                                "(Bet more for full payout)"
                            }
                        }
                    }
                } else if r.is_near_miss {
                    rsx! {
                        p {
                            class: "text-[#ff2d78] text-base",
                            "So close..."
                        }
                    }
                } else {
                    rsx! {
                        p {
                            class: "text-[#7a6a9e] text-sm",
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
    cell_class: &'static str,
    emoji: String,
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

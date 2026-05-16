use super::LeverSlider;
use crate::state::AppState;
use dioxus::prelude::*;
use habit_slot::models::{SlotSymbol, SpinResult};
use habit_slot::sprites::symbol_sprite_uri;

#[component]
pub fn SlotMachine() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    let last_result = app_state.read().last_spin_result.clone();
    let is_spinning = app_state.read().is_spinning;
    let animation_strips = app_state.read().animation_strips.clone();
    let reels_stopped = app_state.read().reels_stopped;

    // Spawn staggered reel-stop timers when spin starts with no reels stopped yet.
    if is_spinning && reels_stopped == 0 {
        let state = use_context::<Signal<AppState>>().clone();
        spawn(async move {
            Timer::after_millis(1000).await;
            state.with_mut(|s| s.stop_one_reel());

            Timer::after_millis(600).await;
            state.with_mut(|s| s.stop_one_reel());

            Timer::after_millis(600).await;
            state.with_mut(|s| s.stop_one_reel());
        });
    }

    rsx! {
        style { include_str!("slot_styles.css") }

        div {
            class: "slot-machine mt-8 p-4 bg-[#1a0a2e] rounded-xl border-2 border-[#ff2d78] w-[96%]",

            if is_spinning && animation_strips.is_some() {
                AnimatedReels {
                    strips: animation_strips.unwrap(),
                    reels_stopped,
                    spin_result: last_result.clone()
                }
            } else {
                Reels { spin_result: last_result.clone() }
            }

            div {
                class: "flex justify-center mt-4",
                LeverSlider {
                    is_disabled: is_spinning,
                    on_trigger: Callback::from(move |_| {
                        app_state.with_mut(|state| {
                            let _ = state.execute_spin(1);
                        });
                    })
                }
            }

            SpinResultDisplay { spin_result: last_result }
        }
    }
}

/// Animated reels during spin — shows full strip with CSS translateY animation.
#[component]
fn AnimatedReels(
    strips: [Vec<SlotSymbol>; 3],
    reels_stopped: u8,
    spin_result: Option<SpinResult>,
) -> Element {
    rsx! {
        div {
            class: "reels-container flex justify-center gap-2 p-4 bg-[#0f0520] rounded-lg",
            for col in 0..3 {
                AnimatedReelColumn {
                    col,
                    strip: strips[col].clone(),
                    is_stopped: col as u8 < reels_stopped,
                    result_col: strips[col],
                    spin_result: spin_result.clone(),
                }
            }
        }
    }
}

#[component]
fn AnimatedReelColumn(
    col: usize,
    strip: Vec<SlotSymbol>,
    is_stopped: bool,
    result_col: Vec<SlotSymbol>,
    spin_result: Option<SpinResult>,
) -> Element {
    let strip_height = strip.len() * 52; // 50px per cell + 2px gap
    let visible_height = 156; // 3 cells * 50px + 4 gaps = ~156px

    let translate_y = if is_stopped {
        // Show last 3 symbols (the result)
        format!("translateY(-{}px)", strip_height.saturating_sub(visible_height))
    } else {
        "translateY(0px)"
    };

    let is_grayed = spin_result
        .as_ref()
        .map(|r| r.grayed_high_tier)
        .unwrap_or(false);

    let winning_row = spin_result.as_ref().and_then(|r| {
        if let Some((matched_symbol, _)) = r.symbols_matched {
            let reels = r.reels;
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

    // Determine the winning global row index within the strip.
    // The last 3 symbols are at indices strip.len()-3, strip.len()-2, strip.len()-1
    let result_offset = strip.len().saturating_sub(3);
    let winning_global_row = winning_row.map(|r| result_offset + r);

    rsx! {
        div {
            class: "reel-column-viewport overflow-hidden min-w-[70px] h-[150px]",
            div {
                class: format!("reel-strip {} {}", if is_stopped { "" } else { "blur-[2px]" }, if col == 2 && reels_stopped == 3 { "ease-out-slow" } else { "" }),
                style: format!(
                    "transform: {}; transition: transform {}ms cubic-bezier(0.25, 0.1, 0.25, 1);",
                    translate_y,
                    if col == 2 && is_stopped { 600 } else { 400 }
                ),
                for (idx, symbol) in strip.iter().enumerate() {
                    ReelSymbolCell {
                        symbol,
                        is_winning: winning_global_row == Some(idx),
                        is_grayed,
                    }
                }
            }
        }
    }
}

#[component]
fn ReelSymbolCell(symbol: SlotSymbol, is_winning: bool, is_grayed: bool) -> Element {
    let cell_class = if is_grayed && is_winning {
        "min-w-[70px] h-[50px] flex items-center justify-center bg-[#2a1a4e] rounded-md grayscale brightness-50 opacity-50"
    } else if is_winning {
        "min-w-[70px] h-[50px] flex items-center justify-center bg-[#2a1a4e] rounded-md ring-2 ring-[#00f5d4]"
    } else {
        "min-w-[70px] h-[50px] flex items-center justify-center bg-[#2a1a4e] rounded-md"
    };

    rsx! {
        div {
            class: format!("reel-symbol {}", cell_class),
            img {
                src: symbol_sprite_uri(&symbol),
                class: "w-[28px] h-[24px] object-contain",
            }
        }
    }
}

#[component]
fn Reels(spin_result: Option<SpinResult>) -> Element {
    let default_reels: [[SlotSymbol; 3]; 3] = [
        [SlotSymbol::Kebab, SlotSymbol::Taco, SlotSymbol::Pizza],
        [SlotSymbol::Sushi, SlotSymbol::Kebab, SlotSymbol::Pancake],
        [SlotSymbol::Taco, SlotSymbol::Sashimi, SlotSymbol::Kebab],
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

struct ReelCellData {
    uri: &'static str,
    cell_class: String,
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

    let cells: Vec<ReelCellData> = (0..3)
        .map(|row| {
            let is_winning_cell = winning_row == Some(row);
            let cell_class = if is_grayed && is_winning_cell {
                "min-w-[70px] h-[50px] flex items-center justify-center bg-[#2a1a4e] rounded-md grayscale brightness-50 opacity-50".to_string()
            } else if is_winning_cell {
                "min-w-[70px] h-[50px] flex items-center justify-center bg-[#2a1a4e] rounded-md ring-2 ring-[#00f5d4]".to_string()
            } else {
                "min-w-[70px] h-[50px] flex items-center justify-center bg-[#2a1a4e] rounded-md".to_string()
            };
            ReelCellData {
                uri: symbol_sprite_uri(&reels[col][row]),
                cell_class,
            }
        })
        .collect();

    rsx! {
        div {
            class: "reel-column flex flex-col gap-1",
            for cell in cells {
                div {
                    class: format!("reel-symbol {}", cell.cell_class),
                    img {
                        src: cell.uri,
                        class: "w-[28px] h-[24px] object-contain",
                    }
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

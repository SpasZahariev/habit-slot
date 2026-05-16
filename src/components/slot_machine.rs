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

    if is_spinning && reels_stopped == 0 {
        let mut state_clone = use_context::<Signal<AppState>>().clone();
        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            state_clone.with_mut(|s| s.stop_one_reel());

            tokio::time::sleep(std::time::Duration::from_millis(600)).await;
            state_clone.with_mut(|s| s.stop_one_reel());

            tokio::time::sleep(std::time::Duration::from_millis(600)).await;
            state_clone.with_mut(|s| s.stop_one_reel());
        });
    }

    rsx! {
        style { r"
            .reel-column-viewport {{
                position: relative;
                overflow: hidden;
                width: 70px;
                height: 150px;
                flex-shrink: 0;
            }}

            .reel-strip {{
                display: flex;
                flex-direction: column;
            }}

            @keyframes reel-spin-fast {{
                0% {{
                    transform: translateY(0);
                    filter: blur(3px);
                }}
                85% {{
                    filter: blur(2px);
                }}
                100% {{
                    transform: translateY(-600px);
                    filter: blur(0px);
                }}
            }}

            @keyframes reel-spin-medium {{
                0% {{
                    transform: translateY(0);
                    filter: blur(3px);
                }}
                85% {{
                    filter: blur(2px);
                }}
                100% {{
                    transform: translateY(-600px);
                    filter: blur(0px);
                }}
            }}

            @keyframes reel-spin-slow {{
                0% {{
                    transform: translateY(0);
                    filter: blur(3px);
                }}
                85% {{
                    filter: blur(2px);
                }}
                100% {{
                    transform: translateY(-600px);
                    filter: blur(0px);
                }}
            }}

            .reel-strip-anim-0 {{
                animation: reel-spin-fast 1.0s cubic-bezier(0.15, 0.80, 0.30, 1.0) forwards;
            }}

            .reel-strip-anim-1 {{
                animation: reel-spin-medium 1.6s cubic-bezier(0.15, 0.80, 0.30, 1.0) forwards;
            }}

            .reel-strip-anim-2 {{
                animation: reel-spin-slow 2.2s cubic-bezier(0.15, 0.80, 0.30, 1.0) forwards;
            }}

            .reel-strip-static {{
                transform: translateY(-600px);
            }}
        " }

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
                    on_trigger: Callback::new(move |_| {
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

#[component]
fn AnimatedReels(
    strips: [Vec<SlotSymbol>; 3],
    reels_stopped: u8,
    spin_result: Option<SpinResult>,
) -> Element {
    rsx! {
        div {
            class: "reels-container flex justify-center gap-2 p-4 bg-[#0f0520] rounded-lg",
            ReelColumnAnimated { col: 0, strip: strips[0].clone(), is_stopped: reels_stopped >= 1, spin_result: spin_result.clone() },
            ReelColumnAnimated { col: 1, strip: strips[1].clone(), is_stopped: reels_stopped >= 2, spin_result: spin_result.clone() },
            ReelColumnAnimated { col: 2, strip: strips[2].clone(), is_stopped: reels_stopped >= 3, spin_result },
        }
    }
}

#[component]
fn ReelColumnAnimated(
    col: usize,
    strip: Vec<SlotSymbol>,
    is_stopped: bool,
    spin_result: Option<SpinResult>,
) -> Element {
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

    let result_offset = strip.len().saturating_sub(3);
    let winning_global_row = winning_row.map(|r| result_offset + r);

    let anim_class = if is_stopped {
        "reel-strip-static"
    } else {
        match col {
            0 => "reel-strip-anim-0",
            1 => "reel-strip-anim-1",
            _ => "reel-strip-anim-2",
        }
    };

    rsx! {
        div {
            class: "reel-column-viewport",
            div {
                class: format!("reel-strip {}", anim_class),
                for (idx, symbol) in strip.iter().enumerate() {
                    ReelSymbolCell {
                        symbol: *symbol,
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
        "w-[70px] h-[50px] flex items-center justify-center bg-[#2a1a4e] rounded-md grayscale brightness-50 opacity-50"
    } else if is_winning {
        "w-[70px] h-[50px] flex items-center justify-center bg-[#2a1a4e] rounded-md ring-2 ring-[#00f5d4]"
    } else {
        "w-[70px] h-[50px] flex items-center justify-center bg-[#2a1a4e] rounded-md"
    };

    rsx! {
        div {
            class: cell_class,
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
                "w-[70px] h-[50px] flex items-center justify-center bg-[#2a1a4e] rounded-md grayscale brightness-50 opacity-50".to_string()
            } else if is_winning_cell {
                "w-[70px] h-[50px] flex items-center justify-center bg-[#2a1a4e] rounded-md ring-2 ring-[#00f5d4]".to_string()
            } else {
                "w-[70px] h-[50px] flex items-center justify-center bg-[#2a1a4e] rounded-md".to_string()
            };
            ReelCellData {
                uri: symbol_sprite_uri(&reels[col][row]),
                cell_class,
            }
        })
        .collect();

    rsx! {
        div {
            class: "reel-column flex flex-col",
            for cell in cells {
                div {
                    class: cell.cell_class,
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
    let display = match &spin_result {
        Some(r) if r.payout_coins > 0 => {
            let win_text = format!("Win! +{} coins", r.payout_coins);
            let grayed_note = r.grayed_high_tier;
            rsx! {
                p {
                    class: "text-[#00f5d4] text-xl font-bold",
                    "{win_text}"
                }
                if grayed_note {
                    p {
                        class: "text-gray-500 text-sm mt-1",
                        "(Bet more for full payout)"
                    }
                }
            }
        }
        Some(r) if r.is_near_miss => {
            rsx! {
                p {
                    class: "text-[#ff2d78] text-base",
                    "So close..."
                }
            }
        }
        Some(_) => {
            rsx! {
                p {
                    class: "text-[#7a6a9e] text-sm",
                    "No luck. Try again!"
                }
            }
        }
        None => rsx! {},
    };

    rsx! {
        div {
            class: "spin-result text-center mt-4 min-h-[24px]",
            {display}
        }
    }
}

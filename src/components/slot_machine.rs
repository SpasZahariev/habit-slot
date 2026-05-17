use super::LeverSlider;
use crate::state::AppState;
use dioxus::prelude::*;
use habit_slot::models::{SlotSymbol, SpinResult};
use habit_slot::sprites::symbol_sprite_uri;

/// Cell dimensions for 49x49 PNG icons with padding. Square cells.
const CELL_W: u32 = 86;
const CELL_H: u32 = 86;

/// Gap between cells in a reel strip.
const CELL_GAP: u32 = 5;

/// Total height of one cell + gap for animation calculation.
const CELL_STEP: u32 = CELL_H + CELL_GAP; // 78

/// Top/bottom padding inside the reel strip container.
const STRIP_PADDING: u32 = 10;

/// Viewport height: 3 visible cells + 2 gaps + padding.
const VIEWPORT_HEIGHT: u32 = CELL_H * 3 + CELL_GAP * 2 + STRIP_PADDING * 2; // 200

/// Viewport width matches cell width.
const VIEWPORT_WIDTH: u32 = CELL_W;

/// Animation translate distance: scrolls past all filler symbols to land on result.
/// With 12 filler + 3 result = 15 cells. Final position shows last 3, so scroll = 12 * step - padding.
fn animation_translate_distance() -> i32 {
    // Strip has ANIMATION_FILLER_COUNT + 3 symbols (15 total).
    // At rest position, viewport shows the last 3 cells.
    // Distance to scroll = (filler_count) * cell_step
    12 * CELL_STEP as i32 - STRIP_PADDING as i32
}

#[component]
pub fn SlotMachine() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

           let last_result = app_state.read().last_spin_result.clone();
    let is_spinning = app_state.read().is_spinning;
    let animation_strips = app_state.read().animation_strips.clone();
    let reels_stopped = app_state.read().reels_stopped;
    let coin_balance = app_state.read().coin_balance.balance;

    if is_spinning && reels_stopped == 0 {
        let mut state_clone = use_context::<Signal<AppState>>().clone();
          spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(2500)).await;
            state_clone.with_mut(|s| s.stop_one_reel());

            tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
            state_clone.with_mut(|s| s.stop_one_reel());

            tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
            state_clone.with_mut(|s| s.stop_one_reel());
        });
    }

    let anim_dist = animation_translate_distance();

    rsx! {
        style { r"
            .reel-column-viewport {{
                position: relative;
                overflow: hidden;
                width: {VIEWPORT_WIDTH}px;
                height: {VIEWPORT_HEIGHT}px;
                flex-shrink: 0;
            }}

            .reel-strip {{
                display: flex;
                flex-direction: column;
                gap: {CELL_GAP}px;
                padding-top: {STRIP_PADDING}px;
                padding-bottom: {STRIP_PADDING}px;
            }}

            @keyframes reel-spin-0 {{
                0% {{
                    transform: translateY(0);
                    filter: blur(1.5px);
                }}
                80% {{
                    filter: blur(1px);
                }}
                100% {{
                    transform: translateY({anim_dist}px);
                    filter: blur(0px);
                }}
            }}

            @keyframes reel-spin-1 {{
                0% {{
                    transform: translateY(0);
                    filter: blur(1.5px);
                }}
                80% {{
                    filter: blur(1px);
                }}
                100% {{
                    transform: translateY({anim_dist}px);
                    filter: blur(0px);
                }}
            }}

            @keyframes reel-spin-2 {{
                0% {{
                    transform: translateY(0);
                    filter: blur(1.5px);
                }}
                80% {{
                    filter: blur(1px);
                }}
                100% {{
                    transform: translateY({anim_dist}px);
                    filter: blur(0px);
                }}
            }}

            .reel-strip-anim-0 {{
                animation: reel-spin-0 2.5s cubic-bezier(0.2, 0.8, 0.3, 1) forwards;
            }}

            .reel-strip-anim-1 {{
                animation: reel-spin-1 3.7s cubic-bezier(0.2, 0.8, 0.3, 1) forwards;
            }}

            .reel-strip-anim-2 {{
                animation: reel-spin-2 4.9s cubic-bezier(0.2, 0.8, 0.3, 1) forwards;
            }}

            .reel-strip-static {{
                transform: translateY({anim_dist}px);
            }}

            .slot-cell {{
                flex-shrink: 0;
                width: {CELL_W}px;
                height: {CELL_H}px;
                display: flex;
                align-items: center;
                justify-content: center;
                background-color: #2a1a4e;
                border-radius: 6px;
            }}

            .slot-cell-img {{
                width: {(CELL_W as f32 * 0.57).round() as u32}px;
                height: {(CELL_W as f32 * 0.57).round() as u32}px;
                object-fit: contain;
            }}

            .slot-cell--winning {{
                box-shadow: 0 0 0 2px #00f5d4;
            }}

            .slot-cell--grayed {{
                filter: grayscale(1) brightness(0.5);
                opacity: 0.5;
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
                    is_disabled: is_spinning || coin_balance < 1,
                    on_trigger: Callback::new(move |_| {
                        app_state.with_mut(|state| {
                            let _ = state.execute_spin(1);
                        });
                    })
                }
            }

            if !is_spinning {
                SpinResultDisplay { spin_result: last_result }
            } else {
                div { class: "min-h-[24px]" }
            }
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
        "slot-cell slot-cell--grayed"
    } else if is_winning {
        "slot-cell slot-cell--winning"
    } else {
        "slot-cell"
    };

    rsx! {
        div {
            class: cell_class,
            img {
                src: symbol_sprite_uri(&symbol),
                class: "slot-cell-img",
            }
        }
    }
}

#[component]
fn Reels(spin_result: Option<SpinResult>) -> Element {
    let default_reels: [[SlotSymbol; 3]; 3] = [
        [SlotSymbol::Low0, SlotSymbol::Low1, SlotSymbol::Low2],
        [SlotSymbol::Mid0, SlotSymbol::Low0, SlotSymbol::High0],
        [SlotSymbol::Low1, SlotSymbol::Mid1, SlotSymbol::Low0],
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
    cell_class: &'static str,
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
                "slot-cell slot-cell--grayed"
            } else if is_winning_cell {
                "slot-cell slot-cell--winning"
            } else {
                "slot-cell"
            };
            ReelCellData {
                uri: symbol_sprite_uri(&reels[col][row]),
                cell_class,
            }
        })
        .collect();

      rsx! {
        div {
            class: "reel-column-viewport",
            div {
                class: "reel-strip",
                for cell in cells {
                    div {
                        class: cell.cell_class,
                        img {
                            src: cell.uri,
                            class: "slot-cell-img",
                        }
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

use super::LeverSlider;
use crate::state::AppState;
use dioxus::prelude::*;
use habit_slot::models::{SlotSymbol, SpinResult};
use habit_slot::sprites::{display_names_match, symbol_sprite_uri, winning_border_color};

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

/// Absolute pixel distance to translate the strip upward so the last 3 cells (result symbols) align with the viewport.
fn animation_translate_distance() -> i32 {
    // Strip has ANIMATION_FILLER_COUNT + 3 symbols (15 total).
    // At rest position, viewport shows the last 3 cells.
    // Distance to scroll = (filler_count) * cell_step - padding_offset
    12 * CELL_STEP as i32 - STRIP_PADDING as i32
}

#[component]
pub fn SlotMachine() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut spin_bet = use_signal(|| 1u32);

    let last_result = app_state.read().last_spin_result.clone();
    let is_spinning = app_state.read().is_spinning;
    let animation_strips = app_state.read().animation_strips.clone();
    let reels_stopped = app_state.read().reels_stopped;
    let balance_u32 = app_state.read().coin_balance.balance as u32;
    let has_low_rewards = app_state.read().has_any_low_tier_rewards();

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

    // Negative translateY value for the end position of the animation.
    let anim_dist = -animation_translate_distance();

    rsx! {
        style { r"
            @keyframes reel-spin-0 {{
                0% {{
                    transform: translateY({anim_dist}px);
                    filter: blur(1.5px);
                }}
                80% {{
                    filter: blur(1px);
                }}
                100% {{
                    transform: translateY(0);
                    filter: blur(0px);
                }}
            }}

            @keyframes reel-spin-1 {{
                0% {{
                    transform: translateY({anim_dist}px);
                    filter: blur(1.5px);
                }}
                80% {{
                    filter: blur(1px);
                }}
                100% {{
                    transform: translateY(0);
                    filter: blur(0px);
                }}
            }}

            @keyframes reel-spin-2 {{
                0% {{
                    transform: translateY({anim_dist}px);
                    filter: blur(1.5px);
                }}
                80% {{
                    filter: blur(1px);
                }}
                100% {{
                    transform: translateY(0);
                    filter: blur(0px);
                }}
            }}

            .reel-strip-anim-0 {{
                will-change: transform, filter;
                animation: reel-spin-0 2.5s cubic-bezier(0.2, 0.8, 0.3, 1) forwards;
            }}

            .reel-strip-anim-1 {{
                will-change: transform, filter;
                animation: reel-spin-1 3.7s cubic-bezier(0.2, 0.8, 0.3, 1) forwards;
            }}

            .reel-strip-anim-2 {{
                will-change: transform, filter;
                animation: reel-spin-2 4.9s cubic-bezier(0.2, 0.8, 0.3, 1) forwards;
            }}

            .reel-strip-static {{
                transform: translateY(0);
            }}
        " }

       if !has_low_rewards {
            div {
                p {
                    style: "color: #ff2d78; font-family: Silkscreen; font-size: 0.85rem; text-align: center; margin-bottom: 15px",
                    "Add Low-tier rewards on the Rewards page to start spinning."
                }
            }
        } else {
            div {
                p {
                    style: "color: rgba(240,230,255,0.3); font-family: Silkscreen; font-size: 0.85rem; text-align: center; margin-bottom: 15px",
                    "Spend more coins to qualify for Higher Tier rewards."
                }
            }
        }

       div {
            class: "flex flex-col items-center w-full relative",

            div {
                class: "bet-selector flex justify-center gap-2 mb-3",
                for i in 1..=3u32 {
                    button {
                        class: format!("px-4 py-2 rounded-lg font-bold text-sm transition-all {}",
                            if spin_bet() == i {
                                "bg-[#ff2d78] text-white shadow-lg shadow-[#ff2d78]/30"
                            } else {
                               if balance_u32 >= i && !is_spinning {
                                    "bg-[#2a1a4e] text-[#7a6a9e] hover:bg-[#3a2a5e] hover:text-white"
                                } else {
                                    "bg-[#1a0a2e] text-[#3a2a5e] cursor-not-allowed"
                                }
                            }
                        ),
                        disabled: balance_u32 < i || is_spinning,
                        onclick: move |_| { spin_bet.set(i) },
                        { format!("{} Coin{}", i, if i == 1 { "" } else { "s" }) }
                    }
                }
            }

           div {
                class: "slot-machine relative p-4 bg-[#1a0a2e] rounded-xl border-2 border-[#ff2d78] w-[96%]",

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
                        is_disabled: is_spinning || balance_u32 < spin_bet() || !app_state.read().has_any_low_tier_rewards(),
                        on_trigger: Callback::new(move |_| {
                            let bet = spin_bet();
                            app_state.with_mut(|state| {
                                let _ = state.execute_spin(bet);
                            });
                        })
                    }
                }

            }

           if !is_spinning {
                SpinResultDisplay { spin_result: last_result }
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
    let is_grayed: bool = false;

    // Find ALL winning rows, each with their own tier color
    let all_winning_rows: Vec<(usize, SlotSymbol)> = spin_result.as_ref().map(|r| {
        let reels = r.reels;
        let mut wins = Vec::new();
        for row in 0..3 {
            let a = reels[0][row];
            let b = reels[1][row];
            let c = reels[2][row];
            if display_names_match(a, b, c) {
                wins.push((row, a));
            }
        }
        wins
    }).unwrap_or_default();

    let result_symbols = strip.iter().skip(strip.len().saturating_sub(3)).cloned().collect::<Vec<_>>();

    let anim_strip: Vec<SlotSymbol> = [result_symbols.clone(), strip].concat();

    let strip_len = anim_strip.len();

    let anim_class = if is_stopped {
        "reel-strip-static"
    } else {
        match col {
            0 => "reel-strip-anim-0",
            1 => "reel-strip-anim-1",
            _ => "reel-strip-anim-2",
        }
    };

    let viewport_style = format!(
        "position:relative;overflow:hidden;width:{}px;height:{}px;flex-shrink:0",
        VIEWPORT_WIDTH, VIEWPORT_HEIGHT
    );
    let strip_layout = format!(
        "display:flex;flex-direction:column;gap:{}px;padding-top:{}px;padding-bottom:{}px",
        CELL_GAP, STRIP_PADDING, STRIP_PADDING
    );

    let result_start = strip_len.saturating_sub(3);
    let cell_colors: Vec<Option<&'static str>> = anim_strip.iter().enumerate().map(|(idx, _symbol)| {
        if idx >= result_start {
            let row = idx - result_start;
            all_winning_rows.iter()
                .find(|(r, _)| *r == row)
                .map(|(_, sym)| winning_border_color(*sym))
        } else {
            None
        }
    }).collect();

    rsx! {
        div {
            style: viewport_style,
            div {
                class: anim_class,
                style: strip_layout,
                for (idx, symbol) in anim_strip.iter().enumerate() {
                    ReelSymbolCell {
                        symbol: *symbol,
                        winning_color: cell_colors[idx],
                        is_grayed,
                    }
                }
            }
        }
    }
}

#[component]
fn ReelSymbolCell(symbol: SlotSymbol, winning_color: Option<&'static str>, is_grayed: bool) -> Element {
    let cell_style = format!(
        "flex-shrink:0;width:{}px;height:{}px;display:flex;align-items:center;justify-content:center;background-color:#2a1a4e;border-radius:6px",
        CELL_W, CELL_H
    );

    let is_winning = winning_color.is_some();

    let wrapper_style = if is_winning {
        let color = winning_color.unwrap();
        format!("box-shadow:0 0 0 3px {color};box-shadow:0 0 8px {color}")
    } else {
        String::new()
    };

    let img_filter = if is_grayed && is_winning {
        "filter:grayscale(1) brightness(0.5);opacity:0.5"
    } else {
        ""
    };

    let img_style = format!(
        "width:{}px;height:{}px;object-fit:contain;{}",
        (CELL_W as f32 * 0.57).round() as u32,
        (CELL_W as f32 * 0.57).round() as u32,
        img_filter
    );

    rsx! {
        div {
            style: format!("{};{}", cell_style, wrapper_style),
            img {
                src: symbol_sprite_uri(&symbol),
                style: img_style,
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
    border_css: String,
    img_filter_css: String,
}

#[component]
fn ReelColumn(col: usize, reels: [[SlotSymbol; 3]; 3], spin_result: Option<SpinResult>) -> Element {
    let is_grayed: bool = false;

    // Find ALL winning rows, each with their own tier color
    let all_winning_rows: Vec<(usize, SlotSymbol)> = spin_result.as_ref().map(|r| {
        let reels = r.reels;
        let mut wins = Vec::new();
        for row in 0..3 {
            let a = reels[0][row];
            let b = reels[1][row];
            let c = reels[2][row];
            if display_names_match(a, b, c) {
                wins.push((row, a));
            }
        }
        wins
    }).unwrap_or_default();

    let viewport_style = format!(
        "position:relative;overflow:hidden;width:{}px;height:{}px;flex-shrink:0",
        VIEWPORT_WIDTH, VIEWPORT_HEIGHT
    );
    let strip_layout = format!(
        "display:flex;flex-direction:column;gap:{}px;padding-top:{}px;padding-bottom:{}px",
        CELL_GAP, STRIP_PADDING, STRIP_PADDING
    );

  let cells: Vec<ReelCellData> = (0..3)
        .map(|row| {
            let winning_color = all_winning_rows.iter()
                .find(|(r, _)| *r == row)
                .map(|(_, sym)| winning_border_color(*sym));
            let is_winning_cell = winning_color.is_some();

            let border_css = if is_winning_cell {
                let color = winning_color.unwrap();
                format!("box-shadow:0 0 0 3px {color};box-shadow:0 0 8px {color}")
            } else {
                String::new()
            };

            let img_filter_css = if is_grayed && is_winning_cell {
                "filter:grayscale(1) brightness(0.5);opacity:0.5".to_string()
            } else {
                String::new()
            };

            ReelCellData {
                uri: symbol_sprite_uri(&reels[col][row]),
                border_css,
                img_filter_css,
            }
        })
        .collect();

    let cell_base_style = format!(
        "flex-shrink:0;width:{}px;height:{}px;display:flex;align-items:center;justify-content:center;background-color:#2a1a4e;border-radius:6px",
        CELL_W, CELL_H
    );
    let img_style = format!(
        "width:{}px;height:{}px;object-fit:contain",
        (CELL_W as f32 * 0.57).round() as u32,
        (CELL_W as f32 * 0.57).round() as u32
    );

              rsx! {
        div {
            style: viewport_style,
            div {
                style: strip_layout,
                for cell in cells {
                    div {
                        style: format!("{};{}", cell_base_style, cell.border_css),
                        img {
                            src: cell.uri,
                            style: format!("{};{}", img_style.clone(), cell.img_filter_css),
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
        Some(r) if !r.reward_note.is_empty() => {
            rsx! {
                p {
                    class: "text-[#00f5d4] text-xl font-bold",
                    "{r.reward_note}"
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

use dioxus::prelude::*;

use crate::state::AppState;
use habit_slot::models::GlobalRewardTier;

#[component]
pub fn RewardModal() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut name = use_signal(|| String::new());
    let mut tier = use_signal(|| GlobalRewardTier::Low);

    rsx! {
        if app_state.read().global_rewards_modal_open {
            div {
                onclick: move |_| {
                    app_state.write().global_rewards_modal_open = false;
                    name.set(String::new());
                    tier.set(GlobalRewardTier::Low);
                },
                style: "position: fixed; inset: 0; background: rgba(10,5,20,0.85); display: flex; align-items: center; justify-content: center; z-index: 100;",

                div {
                    onclick: |e| e.stop_propagation(),
                    style: "background: #1a0a2e; border: 2px solid rgba(255,45,120,0.4); border-radius: 16px; padding: 24px; width: 90%; max-width: 360px;",

                    form {
                        onsubmit: move |e| {
                            e.prevent_default();
                            let trimmed = name.read().trim().to_string();
                            if !trimmed.is_empty() {
                                let selected_tier = tier.read().clone();
                                app_state.write().add_global_reward(trimmed, selected_tier);
                                app_state.write().global_rewards_modal_open = false;
                                name.set(String::new());
                                tier.set(GlobalRewardTier::Low);
                            }
                        },
                        style: "display: flex; flex-direction: column; gap: 16px;",

                        div {
                            style: "display: flex; flex-direction: column; gap: 6px;",
                            label { style: "color: #f0e6ff; font-size: 0.85rem;", "Reward Name" }
                            input {
                                type: "text",
                                value: name.read().to_string(),
                                oninput: move |e| name.set(e.value().to_string()),
                                placeholder: "e.g. Chocolate Snack",
                                style: "background: #2a1a4e; border: 1px solid rgba(0,245,212,0.3); border-radius: 8px; padding: 10px 12px; color: #f0e6ff; font-family: 'Pixelify Sans', monospace; font-size: 0.95rem; outline: none;",
                            }
                        },

                        div {
                            style: "display: flex; flex-direction: column; gap: 8px;",
                            label { style: "color: #f0e6ff; font-size: 0.85rem;", "Tier" }
                            div {
                                style: "display: flex; gap: 8px;",

                                button {
                                    type: "button",
                                    onclick: move |_| tier.set(GlobalRewardTier::Low),
                                    style: format_tier_button(GlobalRewardTier::Low, tier.read().clone(), "#4ade80"),
                                    "Low"
                                }
                                button {
                                    type: "button",
                                    onclick: move |_| tier.set(GlobalRewardTier::Medium),
                                    style: format_tier_button(GlobalRewardTier::Medium, tier.read().clone(), "#c084fc"),
                                    "Medium"
                                }
                                button {
                                    type: "button",
                                    onclick: move |_| tier.set(GlobalRewardTier::Jackpot),
                                    style: format_tier_button(GlobalRewardTier::Jackpot, tier.read().clone(), "#fb923c"),
                                    "Jackpot"
                                }
                            }
                        },

                        div {
                            style: "display: flex; gap: 8px; margin-top: 8px;",
                            button {
                                type: "submit",
                                class: "flex-1",
                                style: "background: #00f5d4; color: #1a0a2e; border: none; border-radius: 8px; padding: 10px 16px; font-family: 'Pixelify Sans', monospace; font-size: 0.95rem; cursor: pointer;",
                                "Add"
                            }
                            button {
                                type: "button",
                                onclick: move |_| {
                                    app_state.write().global_rewards_modal_open = false;
                                    name.set(String::new());
                                    tier.set(GlobalRewardTier::Low);
                                },
                                style: "background: #2a1a4e; color: #f0e6ff; border: 1px solid rgba(255,45,120,0.3); border-radius: 8px; padding: 10px 16px; font-family: 'Pixelify Sans', monospace; font-size: 0.95rem; cursor: pointer;",
                                "Cancel"
                            }
                        }
                    }
                }
            }
        }
    }
}

fn format_tier_button(
    tier_value: GlobalRewardTier,
    selected: GlobalRewardTier,
    color: &str,
) -> String {
    let is_selected = tier_value == selected;
    let bg = if is_selected { color } else { "#2a1a4e" };
    let fg = if is_selected { "#1a0a2e" } else { color };
    format!(
        "flex:1; padding:8px 4px; border-radius:8px; font-family:'Pixelify Sans',monospace; font-size:0.8rem; cursor:pointer; text-align:center; background:{}; color:{}; border:2px solid {};",
        bg, fg, color
    )
}

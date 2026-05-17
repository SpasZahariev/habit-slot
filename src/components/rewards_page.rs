use dioxus::prelude::*;

use crate::components::RewardModal;
use crate::models::GlobalRewardTier;
use crate::state::AppState;
use uuid::Uuid;

#[component]
pub fn RewardsPage() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let rewards = app_state.read().global_rewards.clone();

    let mut sorted: Vec<_> = rewards;
    sorted.sort_by(|a, b| b.id.cmp(&a.id));

    rsx! {
        RewardModal {}

        div {
            style: "display: flex; justify-content: flex-end; margin-bottom: 12px;",
            button {
                onclick: move |_| app_state.write().global_rewards_modal_open = true,
                style: "background: linear-gradient(135deg, #ff2d78, #c9464f); color: #f0e6ff; border: none; border-radius: 8px; padding: 8px 16px; font-family: 'Pixelify Sans', monospace; font-size: 0.9rem; cursor: pointer;",
                "Add Reward"
            }
        }

        if sorted.is_empty() {
            div {
                class: "flex flex-col items-center flex-1 justify-center py-8 text-center opacity-70",
                p { "No rewards yet." }
                p { "Tap \"Add Reward\" to create your first reward goal." }
            }
        } else {
            ul {
                class: "list-none w-full gap-2 flex flex-col",
                for reward in sorted {
                    li {
                        class: "p-4 mb-2 bg-[#2a1a4e] rounded-lg border border-[rgba(255,45,120,0.2)]",
                        div {
                            class: "flex justify-between items-center gap-2",
                            strong {
                                class: "text-[1.1rem] text-[#00f5d4]",
                                "{&reward.name}"
                            }
                            div {
                                style: "display: flex; align-items: center; gap: 8px;",

                                span {
                                    class: format!("text-sm px-2 py-1 rounded-md {}", match reward.tier {
                                        GlobalRewardTier::Low => "bg-[#2a1a4e] text-[#4ade80] border border-[#4ade80]",
                                        GlobalRewardTier::Medium => "bg-[#2a1a4e] text-[#c084fc] border border-[#c084fc]",
                                        GlobalRewardTier::Jackpot => "bg-[#2a1a4e] text-[#fb923c] border border-[#fb923c]",
                                    }),
                                    match reward.tier {
                                        GlobalRewardTier::Low => "Low",
                                        GlobalRewardTier::Medium => "Medium",
                                        GlobalRewardTier::Jackpot => "Jackpot",
                                    }
                                }

                                DeleteRewardButton { reward_id: reward.id }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DeleteRewardButton(reward_id: Uuid) -> Element {
    let app_state = use_context::<Signal<AppState>>();

    rsx! {
        button {
            type: "button",
            onclick: move |_| {
                app_state.write().remove_global_reward(reward_id);
            },
            style: "
                background: transparent;
                color: rgba(240, 230, 255, 0.4);
                border: none;
                cursor: pointer;
                font-size: 1.1rem;
                padding: 4px 8px;
                line-height: 1;
                transition: color 0.15s;
            ",
            onmouseenter: move |_| {},
            "×"
        }
    }
}

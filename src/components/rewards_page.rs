use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn RewardsPage() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let rewards = app_state.read().global_rewards.clone();

    if rewards.is_empty() {
        return rsx! {
            div {
                class: "flex flex-col items-center flex-1 justify-center py-8 text-center opacity-70",
                p { "No rewards yet." }
                p { "Rewards earned from the slot machine will appear here." }
            }
        };
    }

    rsx! {
        ul {
            class: "list-none w-full gap-2 flex flex-col",
            for reward in rewards {
                li {
                    class: "p-4 mb-2 bg-[#2a1a4e] rounded-lg border border-[rgba(255,45,120,0.2)]",
                    div {
                        class: "flex justify-between items-center",
                        strong {
                            class: "text-[1.1rem] text-[#00f5d4]",
                            "{&reward.name}"
                        }
                        span {
                            class: format!("text-sm px-2 py-1 rounded-md {}", match reward.tier {
                                crate::models::GlobalRewardTier::Low => "bg-[#2a1a4e] text-[#b8a9d4]",
                                crate::models::GlobalRewardTier::Medium => "bg-[#ff2d78] text-[#f0e6ff]",
                                crate::models::GlobalRewardTier::Jackpot => "bg-[#00f5d4] text-[#1a0a2e]",
                            }),
                            match reward.tier {
                                crate::models::GlobalRewardTier::Low => "Low",
                                crate::models::GlobalRewardTier::Medium => "Medium",
                                crate::models::GlobalRewardTier::Jackpot => "Jackpot",
                            }
                        }
                    }
                }
            }
        }
    }
}

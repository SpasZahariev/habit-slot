use crate::components::AgisAnimation;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn HomePage() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let balance = app_state.read().coin_balance.balance;

    rsx! {
        div {
            class: "flex flex-col items-center flex-1 w-full justify-between py-8",

            div {
                class: "flex flex-col items-center justify-center flex-1",

                h1 {
                    class: "text-title mb-2 text-[#ff2d78] drop-shadow-[0_0_10px_rgba(255,45,120,0.5)]",
                    "Habit Slot"
                }

                AgisAnimation {}

                button {
                    class: "nav-button w-full max-w-xs py-btn-padding text-btn-lg font-bold bg-[#ff2d78] text-[#f0e6ff] rounded-xl mb-4 shadow-[0_0_15px_rgba(255,45,120,0.4)] border-none cursor-pointer",
                    onclick: move |_| {
                        app_state.with_mut(|state| state.navigate(crate::state::Page::SlotMachine));
                    },
                    "Slot Machine"
                }

                button {
                    class: "nav-button w-full max-w-xs py-btn-padding text-btn-lg font-bold bg-[#2a1a4e] text-[#00f5d4] rounded-xl border-2 border-[#ff2d78] cursor-pointer",
                    onclick: move |_| {
                        app_state.with_mut(|state| state.navigate(crate::state::Page::Habits));
                    },
                    "Habits"
                }
            }

            div {
                class: "text-2xl text-[#00f5d4]",
                "{balance} coins"
            }
        }
    }
}

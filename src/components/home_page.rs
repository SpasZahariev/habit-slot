use crate::components::AgisAnimation;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn HomePage() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    rsx! {
        div {
            class: "flex flex-col items-center flex-1 w-full justify-center py-8",

            h1 {
                class: "text-title mb-2 text-[#ff2d78] drop-shadow-[0_0_10px_rgba(255,45,120,0.5)]",
                "Habit Slot"
            }

            AgisAnimation {}

            button {
                class: "pixel-btn-base pixel-btn-filled nav-button w-full max-w-xs py-btn-padding text-btn-lg font-bold bg-[#ff2d78] text-[#f0e6ff] mb-4 cursor-pointer",
                onclick: move |_| {
                    app_state.with_mut(|state| state.navigate(crate::state::Page::SlotMachine));
                },
                "Slot Machine"
            }

            button {
                class: "pixel-btn-base pixel-btn-outlined nav-button w-full max-w-xs py-btn-padding text-btn-lg font-bold bg-[#2a1a4e] text-[#00f5d4] mb-4 cursor-pointer",
                onclick: move |_| {
                    app_state.with_mut(|state| state.navigate(crate::state::Page::Rewards));
                },
                "Rewards"
            }

            button {
                class: "pixel-btn-base pixel-btn-outlined nav-button w-full max-w-xs py-btn-padding text-btn-lg font-bold bg-[#2a1a4e] text-[#00f5d4] cursor-pointer",
                onclick: move |_| {
                    app_state.with_mut(|state| state.navigate(crate::state::Page::Habits));
                },
                "Habits"
            }
        }
    }
}

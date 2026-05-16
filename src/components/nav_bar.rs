use crate::state::{AppState, Page};
use dioxus::prelude::*;

#[component]
pub fn NavBar() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let current_page = app_state.read().current_page;

    let title = match current_page {
        Page::Home => "Habit Slot",
        Page::SlotMachine => "Slot Machine",
        Page::Habits => "Habits",
        Page::CreateHabit => "Create Habit",
    };

    rsx! {
        div {
            class: "flex items-center gap-4 w-full mb-6 pb-3 border-b border-[#ff2d78]/30",
            button {
                class: "text-5xl font-bold text-[#ff2d78] cursor-pointer bg-none border-none p-0 leading-none",
                onclick: move |_| {
                    app_state.with_mut(|state| state.go_home());
                },
                "\u{2190}"
            }
            h2 {
                class: "text-xl text-[#f0e6ff]",
                "{title}"
            }
        }
    }
}

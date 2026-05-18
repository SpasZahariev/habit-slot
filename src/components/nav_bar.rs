use crate::state::{AppState, Page};
use dioxus::prelude::*;
use habit_slot::sprites::back_arrow_uri;

#[component]
pub fn NavBar() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let current_page = app_state.read().current_page.clone();

    let title: String = match &current_page {
        Page::Home => "Habit Slot".to_string(),
        Page::SlotMachine => "Slot Machine".to_string(),
        Page::Habits => "Habits".to_string(),
        Page::HabitDetail(id_str) => app_state.read().get_habit_name(id_str),
        Page::Rewards => "Rewards".to_string(),
    };

    let back_action = move |_| {
        let _ = app_state.with_mut(|state| state.handle_back());
    };

    rsx! {
        div {
            class: "flex items-center gap-3 w-full mb-6 pb-3 border-b border-[#ff2d78]/30",
            button {
                class: "cursor-pointer bg-none border-none p-0 flex items-center",
                onclick: back_action,
                img {
                    src: back_arrow_uri(),
                    class: "h-[27.2px] w-auto object-contain",
                    alt: "Back",
                }
            }
            h2 {
                class: "text-xl font-semibold text-[#f0e6ff] leading-none self-center",
                "{title}"
            }
        }
    }
}

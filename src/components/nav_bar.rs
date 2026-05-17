use crate::state::{AppState, Page};
use dioxus::prelude::*;
use habit_slot::sprites::back_arrow_uri;

#[component]
pub fn NavBar() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let current_page = app_state.read().current_page;

    let title = match &current_page {
        Page::Home => "Habit Slot",
        Page::SlotMachine => "Slot Machine",
        Page::Habits => "Habits",
        Page::HabitDetail(id_str) => app_state.read().get_habit_name(id_str),
        Page::Rewards => "Rewards",
    };

    let back_action = move |_| {
        app_state.with_mut(|state| match state.current_page {
            Page::HabitDetail(_) => state.go_habits(),
            _ => state.go_home(),
        });
    };

    rsx! {
        div {
            class: "flex items-center gap-3 w-full mb-6 pb-3 border-b border-[#ff2d78]/30",
            button {
                class: "cursor-pointer bg-none border-none p-0 flex items-center",
                onclick: back_action,
                img {
                    src: back_arrow_uri(),
                    class: "h-4 w-auto object-contain",
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

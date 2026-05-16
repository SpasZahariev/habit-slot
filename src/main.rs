mod components;
mod state;

use crate::components::{HabitList, HomePage, NavBar, SlotMachine};
use crate::state::{use_app_state, Page};
use dioxus::prelude::*;

const TAILWIND_CSS: &str = include_str!("./css/tailwind.css");

fn main() {
    launch(App);
}

fn App() -> Element {
    let app_state = use_app_state();

    provide_context(app_state.clone());

    let current_page = app_state.read().current_page;

    rsx! {
        Meta { name: "viewport", content: "width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no" }
        Stylesheet { integrity: "", href: "https://fonts.googleapis.com/css2?family=Pixelify+Sans:wght@400..700&display=swap" }
        style { "{TAILWIND_CSS}" }

        div {
            class: "flex flex-col items-center min-h-[100dvh] w-full overflow-x-hidden px-4 py-3 font-['Pixelify_Sans'] bg-gradient-to-b from-[#1a0a2e] to-[#2d1b69] text-[#f0e6ff]",

            if current_page != Page::Home {
                NavBar {}
            }

            match current_page {
                Page::Home => rsx! { HomePage {} },
                Page::SlotMachine => rsx! { SlotMachine {} },
                Page::Habits => rsx! { HabitList {} },
                Page::CreateHabit => rsx! { NavBar {} },
            }
        }
    }
}

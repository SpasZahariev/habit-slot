mod components;
mod state;

use crate::components::{CoinFooterBar, HabitList, HomePage, NavBar, SlotMachine, ToastContainer};
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
        style { "@import url('https://fonts.googleapis.com/css2?family=Pixelify+Sans:wght@400..700&display=swap'); {TAILWIND_CSS} @keyframes toast-slide-in {{ from {{ transform: translateY(-100%); opacity: 0; }} to {{ transform: translateY(0); opacity: 1; }} }}" }

        div {
            class: "flex flex-col min-h-full-vh w-full overflow-x-hidden px-4 py-3 font-pixel bg-gradient-to-b from-[#1a0a2e] to-[#2d1b69] text-[#f0e6ff] pb-16",

            ToastContainer {}

            if current_page != Page::Home {
                NavBar {}
            }

            div {
                class: "flex-1",
                match current_page {
                    Page::Home => rsx! { HomePage {} },
                    Page::SlotMachine => rsx! { SlotMachine {} },
                    Page::Habits => rsx! { HabitList {} },
                    Page::CreateHabit => rsx! { NavBar {} },
                }
            }

            CoinFooterBar {}
        }
    }
}

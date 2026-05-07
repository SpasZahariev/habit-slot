mod components;
mod state;

use crate::components::{HabitForm, HabitList};
use crate::state::use_app_state;
use dioxus::prelude::*;

fn main() {
    launch(App);
}

fn App() -> Element {
    let app_state = use_app_state();

    provide_context(app_state.clone());

    rsx! {
        div {
            class: "app",
            style: "display: flex; flex-direction: column; align-items: center; min-height: 100vh; padding: 24px; background: #1a1a2e; color: #f5c518; font-family: sans-serif;",

            h1 {
                style: "font-size: 2rem; margin-bottom: 24px;",
                "Spas Slot"
            }

            div {
                class: "coin-balance",
                style: "font-size: 1.2rem; margin-bottom: 24px;",
                "{app_state.read().coin_balance.balance} Soul Coins"
            }

            HabitForm {}
            HabitList {}
        }
    }
}

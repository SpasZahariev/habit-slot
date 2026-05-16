mod components;
mod state;

use crate::components::{HabitForm, HabitList, SlotMachine};
use crate::state::use_app_state;
use dioxus::prelude::*;

fn main() {
    launch(App);
}

const GLOBAL_CSS: &str = "html,body{margin:0!important;padding:0!important;overflow:hidden!important;width:100%;height:100%}@import url('https://fonts.googleapis.com/css2?family=Pixelify+Sans:wght@400..700&display=swap');body{font-family:'Pixelify Sans',sans-serif!important}*{box-sizing:border-box}";

fn App() -> Element {
    let app_state = use_app_state();

    provide_context(app_state.clone());

    rsx! {
        style { content: GLOBAL_CSS }

        div {
            class: "app",
            style: "display: flex; flex-direction: column; align-items: center; min-height: 100vh; width: 100vw; margin: 0; padding: 24px; background: linear-gradient(180deg, #1a0a2e, #2d1b69); color: #f0e6ff; font-family: 'Pixelify Sans', sans-serif;",

            h1 {
                style: "font-size: 2rem; margin-bottom: 24px; color: #ff2d78; text-shadow: 0 0 10px rgba(255,45,120,0.5);",
                "Habit Slot"
            }

            div {
                class: "coin-balance",
                style: "font-size: 1.2rem; margin-bottom: 24px; color: #00f5d4;",
                "{app_state.read().coin_balance.balance} coins"
            }

            HabitForm {}
            HabitList {}
            SlotMachine {}
        }
    }
}

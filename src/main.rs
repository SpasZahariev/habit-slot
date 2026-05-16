mod components;
mod state;

use crate::components::{HabitForm, HabitList, SlotMachine};
use crate::state::use_app_state;
use dioxus::prelude::*;

fn main() {
    launch(App);
}

fn App() -> Element {
    let app_state = use_app_state();

    provide_context(app_state.clone());

    inject_global_styles();

    rsx! {
        style { r#"
            html, body {
                margin: 0 !important;
                padding: 0 !important;
                overflow: hidden !important;
                width: 100%;
                height: 100%;
            }
            @import url('https://fonts.googleapis.com/css2?family=Pixelify+Sans:wght@400..700&display=swap');
            body {
                font-family: 'Pixelify Sans', sans-serif !important;
            }
        "# }

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

fn inject_global_styles() {
    #[cfg(feature = "dioxus")]
    use_effect(move || {
        let document = dioxus::web::document();
        let style = document.create_element("style").unwrap();
        style.set_text_content(Some(
            r#"* { box-sizing: border-box; } html, body { margin: 0; padding: 0; overflow: hidden; }"#,
        ));
        if let Some(head) = document.query_selector("head").ok().flatten() {
            let _ = head.append_child(&style);
        }
    });
}

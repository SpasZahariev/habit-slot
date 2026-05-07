mod components;
mod economy;
mod models;
mod rewards;
mod slot;
mod state;
mod streaks;

use dioxus::prelude::*;

fn main() {
    launch(App);
}

fn App() -> Element {
    rsx! {
        div {
            h1 { "Spas Slot" }
        }
    }
}

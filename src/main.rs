mod components;
mod state;

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

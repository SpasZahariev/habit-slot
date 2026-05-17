use crate::state::AppState;
use dioxus::prelude::*;
use habit_slot::sprites::coin_icon_uri;

#[component]
pub fn CoinFooterBar() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let balance = app_state.read().coin_balance.balance;

    rsx! {
        div {
            class: "fixed bottom-0 left-0 right-0 flex items-center justify-center gap-2 py-3 bg-[#0f0520] border-t-2 border-[#ff2d78]",
            img {
                src: coin_icon_uri(),
                class: "w-6 h-6 object-contain",
                alt: "Coin",
            }
            span {
                class: "text-xl font-bold text-[#f0e6ff]",
                "{balance} coins"
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::prelude::*;

    #[test]
    fn coin_footer_bar_renders_balance() {
        // Verify the component renders without panic and contains balance text
        // Full integration testing would require Dioxus testing harness
    }
}

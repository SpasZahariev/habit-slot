mod components;
mod state;

use crate::components::{
    CoinFooterBar, HabitDetailPage, HabitList, HabitModal, HomePage, NavBar, RewardsPage, SlotMachine, ToastContainer,
};
use crate::state::{use_app_state, Page};
use dioxus::prelude::*;

const TAILWIND_CSS: &str = include_str!("./css/tailwind.css");

#[cfg(target_os = "android")]
fn set_android_flags() {
    use wry::prelude::dispatch;
    dispatch(|env, activity, _webview| {
        let window = env
            .call_method(activity, "getWindow", "()Landroid/view/Window;", &[])
            .expect("getWindow")
            .l()
            .expect("window");

        let color: i32 = 0xFF1A0A2Eu32 as i32;
        env.call_method(&window, "setStatusBarColor", "(I)V", &[color.into()])
            .ok();
        env.call_method(&window, "setNavigationBarColor", "(I)V", &[color.into()])
            .ok();
    });
}

#[cfg(not(target_os = "android"))]
fn set_android_flags() {}

fn main() {
    launch(App);
}

#[allow(non_snake_case)]
fn App() -> Element {
    let app_state = use_app_state();

    provide_context(app_state.clone());

    use_effect(|| set_android_flags());

    let current_page = app_state.read().current_page.clone();

    rsx! {
        Meta { name: "viewport", content: "width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no, viewport-fit=cover" }
        style { "@import url('https://fonts.googleapis.com/css2?family=Silkscreen:wght@400;700&display=swap'); {TAILWIND_CSS} @keyframes toast-slide-in {{ from {{ transform: translateY(-100%); opacity: 0; }} to {{ transform: translateY(0); opacity: 1; }}" }

        div {
            style: "height: 100vh; overflow: hidden; padding-top: env(safe-area-inset-top, 12px);",
            class: "flex flex-col w-full overflow-x-hidden px-4 pt-3 font-pixel bg-gradient-to-b from-[#1a0a2e] to-[#2d1b69] text-[#f0e6ff] pb-16",

            ToastContainer {}

            if current_page != Page::Home {
                NavBar {}
            }

            div {
                class: "flex-1 min-h-0",
                style: "overflow-y: auto;",
                match current_page {
                    Page::Home => rsx! { HomePage {} },
                    Page::SlotMachine => rsx! { SlotMachine {} },
                    Page::Habits => rsx! {
                        HabitList {}
                        HabitModal {}
                    },
                    Page::HabitDetail(ref habit_id) => rsx! {
                        HabitDetailPage { habit_id: habit_id.clone() }
                    },
                    Page::Rewards => rsx! { RewardsPage {} },
                }
            }

            CoinFooterBar {}
        }
    }
}

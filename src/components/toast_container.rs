use crate::state::AppState;
use dioxus::prelude::*;
use std::time::Duration;

#[component]
pub fn ToastContainer() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let mut task_running = use_signal(|| false);

    if !*task_running.read() {
        *task_running.write() = true;
        spawn(async move {
            loop {
                app_state.with_mut(|s| s.dismiss_expired_toasts());
                tokio::time::sleep(Duration::from_millis(300)).await;
            }
        });
    }

    let toasts = app_state.read().toasts.clone();

    if toasts.is_empty() {
        return None;
    }

    rsx! {
        div {
            class: "toast-container fixed top-0 left-0 right-0 z-50 flex flex-col items-center pointer-events-none px-4 pt-2",
            for (toast, _index) in toasts.iter().enumerate() {
                ToastBanner { toast }
            }
        }
    }
}

#[component]
fn ToastBanner(toast: crate::state::ToastMessage) -> Element {
    rsx! {
        div {
            class: "toast-banner mt-2 px-6 py-3 bg-[#0f0520] border border-[#ff2d78] rounded-xl shadow-[0_0_20px_rgba(255,45,120,0.4)] flex items-center gap-3 pointer-events-auto",
            style: "animation: toast-slide-in 0.3s ease-out;",
            span {
                class: "text-[#00f5d4] text-lg font-bold",
                "{toast.symbol_name}"
            }
            span {
                class: "text-[#ff2d78] text-base font-bold",
                "+{toast.payout} coins"
            }
        }
    }
}

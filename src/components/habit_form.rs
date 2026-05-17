use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn HabitForm() -> Element {
    let mut name = use_signal(|| String::new());
    let mut app_state = use_context::<Signal<AppState>>();

    rsx! {
        form {
            onsubmit: move |e| {
                e.prevent_default();
                let trimmed = name.read().trim().to_string();
                if !trimmed.is_empty() {
                    app_state.write().add_habit(trimmed, 0);
                    name.set(String::new());
                }
            },
            class: "habit-form flex gap-2 mb-6 w-[96%]",

            input {
                r#type: "text",
                placeholder: "New habit...",
                value: &*name.read(),
                oninput: move |e| name.set(e.value().to_string()),
                class: "habit-input flex-1 px-3 py-3 border border-[#ff2d78] rounded-lg bg-[#2a1a4e] text-[#f0e6ff] text-base",
            }

            button {
                r#type: "submit",
                disabled: name.read().trim().is_empty(),
                class: "habit-submit px-6 py-3 border-none rounded-lg bg-[#ff2d78] text-[#f0e6ff] font-bold cursor-pointer text-base",
                "Add"
            }
        }
    }
}

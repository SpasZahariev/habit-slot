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
                    app_state.write().add_habit(trimmed);
                    name.set(String::new());
                }
            },
            class: "habit-form",
            style: "display: flex; gap: 8px; margin-bottom: 24px; width: 100%; max-width: 90vw;",

            input {
                r#type: "text",
                placeholder: "New habit...",
                value: &*name.read(),
                oninput: move |e| name.set(e.value().to_string()),
                class: "habit-input",
                style: "flex: 1; padding: 12px; border: 1px solid #ff2d78; border-radius: 8px; background: #2a1a4e; color: #f0e6ff; font-size: 1rem;",
            }

            button {
                r#type: "submit",
                disabled: name.read().trim().is_empty(),
                class: "habit-submit",
                style: "padding: 12px 24px; border: none; border-radius: 8px; background: #ff2d78; color: #f0e6ff; font-weight: bold; cursor: pointer; font-size: 1rem;",
                "Add"
            }
        }
    }
}

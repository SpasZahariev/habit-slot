use dioxus::prelude::*;

use crate::state::AppState;

#[component]
pub fn HabitModal() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut name = use_signal(|| String::new());
    let mut target_days = use_signal(|| String::new());
    let mut coin_reward = use_signal(|| 1u32);

    let is_name_valid = {
        let n = name.read().trim().to_string();
        !n.is_empty()
            && n.chars()
                .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '\'' || c == '.')
    };

    let submit = move |e: FormEvent| {
        e.prevent_default();
        let trimmed = name.read().trim().to_string();
        if !trimmed.is_empty()
            && trimmed
                .chars()
                .all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '\'' || c == '.')
        {
            let target: u32 = target_days.read().parse().unwrap_or(0);
            let reward = *coin_reward.read();
            app_state.write().add_habit(trimmed, target, reward);
            app_state.write().habit_modal_open = false;
            name.set(String::new());
            target_days.set(String::new());
            coin_reward.set(1);
        }
    };

    let close = move |_| {
        app_state.write().habit_modal_open = false;
        name.set(String::new());
        target_days.set(String::new());
        coin_reward.set(1);
    };

    rsx! {
        if app_state.read().habit_modal_open {
            div {
                onclick: close,
                style: "position: fixed; inset: 0; background: rgba(10,5,20,0.85); display: flex; align-items: center; justify-content: center; z-index: 100;",

                div {
                    onclick: |e| e.stop_propagation(),
                    style: "background: #1a0a2e; border: 2px solid rgba(255,45,120,0.4); border-radius: 16px; padding: 24px; width: 90%; max-width: 360px;",

                    form {
                        onsubmit: submit,
                        style: "display: flex; flex-direction: column; gap: 16px;",

                        div {
                            style: "display: flex; flex-direction: column; gap: 6px;",
                            label { style: "color: #f0e6ff; font-family: Silkscreen; font-size: 0.85rem;", "Habit Name" }
                            input {
                                r#type: "text",
                                value: name.read().to_string(),
                                oninput: move |e| name.set(e.value().to_string()),
                                placeholder: "e.g. Touch Grass today",
                                style: "background: #2a1a4e; border: 1px solid rgba(0,245,212,0.3); border-radius: 8px; padding: 10px 12px; color: #f0e6ff; font-family: Silkscreen; font-size: 0.95rem; outline: none;",
                            }
                        },

                        div {
                            style: "display: flex; flex-direction: column; gap: 6px;",
                            label { style: "color: #f0e6ff; font-family: Silkscreen; font-size: 0.85rem;", "Target Days (optional)" }
                            input {
                                r#type: "number",
                                value: target_days.read().to_string(),
                                oninput: move |e| target_days.set(e.value().to_string()),
                                placeholder: "0",
                                min: "0",
                                style: "background: #2a1a4e; border: 1px solid rgba(0,245,212,0.3); border-radius: 8px; padding: 10px 12px; color: #f0e6ff; font-family: Silkscreen; font-size: 0.95rem; outline: none;",
                            }
                        },

                        div {
                            style: "display: flex; flex-direction: column; gap: 6px;",
                            label { style: "color: #f0e6ff; font-family: Silkscreen; font-size: 0.85rem;", "Coin Reward" }
                            div {
                                style: "display: flex; gap: 8px;",
                                for val in [1u32, 3, 5] {
                                    button {
                                        r#type: "button",
                                        onclick: move |_| coin_reward.set(val),
                                        style: format!(
                                            "flex: 1; border-radius: 8px; padding: 8px; font-family: Silkscreen; font-size: 0.95rem; cursor: pointer; border: 2px solid {}; background: {}; color: #1a0a2e;",
                                            if *coin_reward.read() == val { "rgba(255,255,255,0.3)" } else { "transparent" },
                                            match val {
                                                1 => "#4ade80",
                                                3 => "#a855f7",
                                                5 => "#f97316",
                                                _ => "#4ade80",
                                            }
                                        ),
                                        { format!("{} coin{}", val, if val > 1 { "s" } else { "" }) }
                                    }
                                }
                            }
                        },

                        div {
                            style: "display: flex; gap: 8px; margin-top: 8px;",
                            button {
                                r#type: "submit",
                                class: "flex-1",
                                disabled: !is_name_valid,
                                style: format!("background: {}; color: #1a0a2e; border: none; border-radius: 8px; padding: 10px 16px; font-family: Silkscreen; font-size: 0.95rem; cursor: pointer;", if is_name_valid { "#00f5d4" } else { "#3a2a5e" }),
                                "Add Habit"
                            }
                            button {
                                r#type: "button",
                                onclick: close,
                                style: "background: #2a1a4e; color: #f0e6ff; border: 1px solid rgba(255,45,120,0.3); border-radius: 8px; padding: 10px 16px; font-family: Silkscreen; font-size: 0.95rem; cursor: pointer;",
                                "Cancel"
                            }
                        }
                    }
                }
            }
        }
    }
}

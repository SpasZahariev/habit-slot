use crate::state::AppState;
use dioxus::prelude::*;
use habit_slot::models::Habit;
use std::time::Duration;

#[component]
pub fn HabitList() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let habits = app_state.read().habits.clone();

    rsx! {
        button {
            onclick: move |_| {
                app_state.write().habit_modal_open = true;
            },
            class: "w-[96%] mt-2 mb-3 rounded-lg font-pixel",
            style: "padding: 12px 16px; font-size: 0.95rem; background: transparent; border: 2px solid rgba(255,45,120,0.4); color: #f0e6ff; cursor: pointer;",
            "Add Habit"
        }

        if habits.is_empty() {
            div {
                class: "empty-state text-center px-4 py-12 opacity-70 text-[#f0e6ff]",
                p { "No habits yet. Tap 'Add Habit' to create one." }
            }
        } else {
            ul {
                class: "habit-list list-none w-[96%] gap-2 flex flex-col",
                for habit in habits {
                    HabitItem { habit }
                }
            }
        }
    }
}

#[component]
pub fn HabitItem(habit: Habit) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut pulsing = use_signal(|| false);

    let habit_id = habit.id;

    rsx! {
        li {
            class: "habit-item flex items-center justify-between p-4 mb-2 bg-[#2a1a4e] rounded-lg border border-[rgba(255,45,120,0.2)]",

            div {
                class: "flex items-center gap-2 font-pixel flex-1 min-w-0",

                span {
                    style: "color: #00f5d4; font-size: 0.95rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                    "{&habit.name}"
                }

                span {
                    style: "color: rgba(240,230,255,0.3); font-size: 0.85rem;",
                    "|"
                }

                span {
                    style: "color: #ff2d78; font-size: 0.85rem;",
                    "🔥 {app_state.read().get_streak(habit_id).current_streak_days}"
                }

                span {
                    style: "color: rgba(240,230,255,0.4); font-size: 0.85rem;",
                    "|"
                }

                span {
                    style: "color: rgba(240,230,255,0.6); font-size: 0.85rem;",
                    "today: {app_state.read().get_today_count(habit_id)}"
                }
            }

            button {
                class: format!("tick-btn {}" , if *pulsing.read() { "tick-pulse" } else { "" }),
                style: "width: 36px; height: 36px; border-radius: 50%; background: #4ade80; border: none; color: white; font-size: 1.2rem; cursor: pointer; display: flex; align-items: center; justify-content: center; flex-shrink: 0;",
                onclick: move |_| {
                    app_state.with_mut(|s| {
                        s.increment_habit_completion(habit_id);
                        s.push_toast("✓ Habit ticked".to_string(), 1);
                    });
                    *pulsing.write() = true;
                    spawn(async move {
                        tokio::time::sleep(Duration::from_millis(300)).await;
                        pulsing.with_mut(|v| *v = false);
                    });
                },

                span { "✓" }
            }
        }
    }
}

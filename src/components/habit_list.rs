use crate::state::AppState;
use dioxus::prelude::*;
use habit_slot::models::Habit;
use habit_slot::sprites::{check_gray_uri, check_green_uri};
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
                class: "habit-list list-none w-[96%] gap-3 flex flex-col",
                for habit in habits {
                    HabitRow { habit }
                }
            }
        }
    }
}

#[component]
pub fn HabitRow(habit: Habit) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let mut pulsing = use_signal(|| false);

    let habit_id = habit.id;
    let today_count = app_state.read().get_today_count(habit_id);
    let completed = today_count > 0;
    let check_image = if completed {
        check_green_uri()
    } else {
        check_gray_uri()
    };

    rsx! {
        div {
            class: "habit-row flex items-center gap-3",

            div {
                class: "habit-card flex-1 min-w-0 p-4 bg-[#2a1a4e] rounded-lg border border-[rgba(255,45,120,0.2)]",

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
                        "today: {today_count}"
                    }
                }
            }

            button {
                class: format!("tick-btn {}" , if *pulsing.read() { "tick-pulse" } else { "" }),
                style: "width: 40px; height: 40px; border: none; background: transparent; cursor: pointer; display: flex; align-items: center; justify-content: center; flex-shrink: 0; padding: 0;",
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

                img {
                    src: check_image,
                    style: "width: 28px; height: 28px; image-rendering: pixelated;",
                }
            }
        }
    }
}

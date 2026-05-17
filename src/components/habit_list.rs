use crate::state::AppState;
use dioxus::prelude::*;
use habit_slot::models::Habit;

#[component]
pub fn HabitList() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let habits = app_state.read().habits.clone();

    rsx! {
        button {
            onclick: move |_| {
                app_state.write().habit_modal_open = true;
            },
            class: "pixel-btn-base pixel-btn-outlined w-[96%] mt-4 mb-2",
            style: "padding: 12px 16px; font-size: 0.95rem;",
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
    let app_state = use_context::<Signal<AppState>>();
    let today_count = app_state.read().get_today_count(habit.id);
    let streak = app_state.read().get_streak(habit.id).current_streak_days;

    rsx! {
        li {
            class: "habit-item flex items-center justify-between p-4 mb-2 bg-[#2a1a4e] rounded-lg border border-[rgba(255,45,120,0.2)] cursor-pointer",
            onclick: move |_| {},

            div {
                class: "flex items-center gap-2 font-pixel",

                span {
                    style: "color: #00f5d4; font-size: 0.95rem;",
                    "{&habit.name}"
                }

                span {
                    style: "color: rgba(240,230,255,0.3); font-size: 0.85rem;",
                    "|"
                }

                span {
                    style: "color: #ff2d78; font-size: 0.85rem;",
                    "🔥 {streak}"
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

            div {
                style: "width: 36px; height: 36px;",
            }
        }
    }
}

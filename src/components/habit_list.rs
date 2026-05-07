use crate::state::AppState;
use chrono::Datelike;
use dioxus::prelude::*;
use habit_slot::models::Habit;

fn format_date(d: &chrono::NaiveDate) -> String {
    format!("{:04}-{:02}-{:02}", d.year(), d.month(), d.day())
}

#[component]
pub fn HabitList() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    let habits = app_state.read().habits.clone();

    if habits.is_empty() {
        return rsx! {
            div {
                class: "empty-state",
                style: "text-align: center; padding: 48px 24px; opacity: 0.6;",
                p { "No habits yet." }
                p { "Add your first habit above to start earning soul coins." }
            }
        };
    }

    rsx! {
        ul {
            class: "habit-list",
            style: "list-style: none; padding: 0; margin: 0; width: 100%; max-width: 500px;",
            for habit in habits {
                HabitItem { habit }
            }
        }
    }
}

#[component]
pub fn HabitItem(habit: Habit) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let completed = app_state.read().is_completed_today(habit.id);
    let streak = app_state.read().get_streak(habit.id).current_streak_days;
    let btn_label = if completed { "Done" } else { "Do it" };

    rsx! {
        li {
            class: "habit-item",
            style: "display: flex; justify-content: space-between; align-items: center; padding: 16px; margin-bottom: 8px; background: #16213e; border-radius: 8px;",

            div {
                strong {
                    style: "font-size: 1.1rem; color: #f5c518;",
                    "{&habit.name}"
                }
                br {}
                span {
                    class: "habit-date",
                    style: "font-size: 0.85rem; opacity: 0.5;",
                    "Created {format_date(&habit.created_at)}"
                }
            }

            div {
                style: "display: flex; gap: 8px; align-items: center;",

                span {
                    class: "habit-streak",
                    style: "font-size: 0.9rem; color: #e94560;",
                    "{streak} fire"
                }

                button {
                    class: "habit-toggle",
                    onclick: move |_| {
                        let _ = app_state.write().toggle_completion(habit.id);
                    },
                    style: if completed {
                        "background: #e94560; border: none; color: white; padding: 8px 16px; border-radius: 6px; cursor: pointer;"
                    } else {
                        "background: none; border: 1px solid #f5c518; color: #f5c518; padding: 8px 16px; border-radius: 6px; cursor: pointer;"
                    },
                    "{btn_label}"
                }

                button {
                    class: "habit-delete",
                    onclick: move |_| {
                        app_state.write().remove_habit(habit.id);
                    },
                    style: "background: none; border: 1px solid #e94560; color: #e94560; padding: 4px 12px; border-radius: 6px; cursor: pointer;",
                    "X"
                }
            }
        }
    }
}

use std::collections::HashMap;

use crate::components::CalendarHeatmap;
use crate::state::AppState;
use chrono::Datelike;
use dioxus::prelude::*;
use habit_slot::models::Habit;
use uuid::Uuid;

fn format_date(d: &chrono::NaiveDate) -> String {
    format!("{:04}-{:02}-{:02}", d.year(), d.month(), d.day())
}

#[component]
pub fn HabitList() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let expanded_calendars = use_signal(|| HashMap::<Uuid, bool>::new());

    let habits = app_state.read().habits.clone();

    if habits.is_empty() {
        return rsx! {
            div {
                class: "empty-state",
                style: "text-align: center; padding: 48px 16px; opacity: 0.7; color: #f0e6ff;",
                p { "No habits yet." }
                p { "Add your first habit above to start earning soul coins." }
            }
        };
    }

    rsx! {
          ul {
                class: "habit-list",
                style: "list-style: none; padding: 0; margin: 0; width: 96%; gap: 8px; display: flex; flex-direction: column;",
            for habit in habits {
                HabitItem {
                    habit,
                    expanded_calendars: expanded_calendars.clone(),
                }
            }
        }
    }
}

#[component]
pub fn HabitItem(habit: Habit, expanded_calendars: Signal<HashMap<Uuid, bool>>) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let completed = app_state.read().is_completed_today(habit.id);
    let streak = app_state.read().get_streak(habit.id).current_streak_days;
    let btn_label = if completed { "Done" } else { "Do it" };

    let is_expanded = expanded_calendars
        .read()
        .get(&habit.id)
        .copied()
        .unwrap_or(false);

    let toggle_calendar = move |_| {
        let mut map = expanded_calendars.write();
        map.insert(habit.id, !is_expanded);
    };

    let milestone_progress = app_state.read().get_milestone_progress(habit.id);
    let streak_goal_text = format!(
        "Streak: {}/{}",
        streak, milestone_progress.next_streak_goal.0
    );
    let total_completions = app_state
        .read()
        .completions
        .iter()
        .filter(|c| c.habit_id == habit.id)
        .count();
    let completion_goal_text = format!(
        "Tasks: {}/{}",
        total_completions, milestone_progress.next_completion_goal.0
    );

    rsx! {
        li {
            class: "habit-item",
            style: "display: flex; flex-direction: column; justify-content: space-between; padding: 16px; margin-bottom: 8px; background: #2a1a4e; border-radius: 8px; border: 1px solid rgba(255,45,120,0.2);",

            div {
                style: "display: flex; justify-content: space-between; align-items: center;",

                div {
                    strong {
                        style: "font-size: 1.1rem; color: #00f5d4;",
                        "{&habit.name}"
                    }
                    br {}
                    span {
                        class: "habit-date",
                        style: "font-size: 0.85rem; opacity: 0.6; color: #f0e6ff;",
                        "Created {format_date(&habit.created_at)}"
                    }
                    br {}
                    span {
                        class: "milestone-progress",
                        style: "font-size: 0.75rem; color: #b8a9d4; margin-top: 4px;",
                        "{streak_goal_text} | {completion_goal_text}"
                    }
                }

                div {
                    style: "display: flex; gap: 8px; align-items: center;",

                    span {
                        class: "habit-streak",
                        style: "font-size: 0.9rem; color: #ff2d78;",
                        "{streak} fire"
                    }

                    button {
                        class: "habit-toggle",
                        onclick: move |_| {
                            let _ = app_state.write().toggle_completion(habit.id);
                        },
                        style: if completed {
                            "background: #ff2d78; border: none; color: #f0e6ff; padding: 8px 16px; border-radius: 6px; cursor: pointer;"
                        } else {
                            "background: none; border: 1px solid #00f5d4; color: #00f5d4; padding: 8px 16px; border-radius: 6px; cursor: pointer;"
                        },
                        "{btn_label}"
                    }

                    button {
                        class: "habit-calendar-toggle",
                        onclick: toggle_calendar,
                        style: "background: none; border: 1px solid #7a6a9e; color: #b8a9d4; padding: 4px 8px; border-radius: 6px; cursor: pointer; font-size: 0.8rem;",
                        if is_expanded { "Hide" } else { "Calendar" }
                    }

                    button {
                        class: "habit-delete",
                        onclick: move |_| {
                            app_state.write().remove_habit(habit.id);
                        },
                        style: "background: none; border: 1px solid #ff2d78; color: #ff2d78; padding: 4px 12px; border-radius: 6px; cursor: pointer;",
                        "X"
                    }
                }
            }

            if is_expanded {
                CalendarHeatmap { habit: habit.clone() }
            }
        }
    }
}

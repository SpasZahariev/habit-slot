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
                class: "empty-state text-center px-4 py-12 opacity-70 text-[#f0e6ff]",
                p { "No habits yet." }
                p { "Add your first habit above to start earning coins." }
            }
        };
    }

    rsx! {
        ul {
            class: "habit-list list-none w-[96%] gap-2 flex flex-col",
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
            class: "habit-item flex flex-col justify-between p-4 mb-2 bg-[#2a1a4e] rounded-lg border border-[rgba(255,45,120,0.2)]",

            div {
                class: "flex justify-between items-center",

                div {
                    strong {
                        class: "text-[1.1rem] text-[#00f5d4]",
                        "{&habit.name}"
                    }
                    br {}
                    span {
                        class: "habit-date text-[0.85rem] opacity-60 text-[#f0e6ff]",
                        "Created {format_date(&habit.created_at)}"
                    }
                    br {}
                    span {
                        class: "milestone-progress text-[0.75rem] text-[#b8a9d4] mt-1",
                        "{streak_goal_text} | {completion_goal_text}"
                    }
                }

                div {
                    class: "flex gap-2 items-center",

                    span {
                        class: "habit-streak text-sm text-[#ff2d78]",
                        "{streak} fire"
                    }

                    button {
                        class: format!("habit-toggle border-none rounded-md cursor-pointer px-4 py-2 {}", if completed { "bg-[#ff2d78] text-[#f0e6ff]" } else { "border border-[#00f5d4] text-[#00f5d4] bg-transparent" }),
                        onclick: move |_| {
                            let _ = app_state.write().toggle_completion(habit.id);
                        },
                        "{btn_label}"
                    }

                    button {
                        class: "habit-calendar-toggle border rounded-md cursor-pointer text-xs px-2 py-1 border-[#7a6a9e] text-[#b8a9d4] bg-transparent",
                        onclick: toggle_calendar,
                        if is_expanded { "Hide" } else { "Calendar" }
                    }

                    button {
                        class: "habit-delete border rounded-md cursor-pointer px-3 py-1 border-[#ff2d78] text-[#ff2d78] bg-transparent",
                        onclick: move |_| {
                            app_state.write().remove_habit(habit.id);
                        },
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

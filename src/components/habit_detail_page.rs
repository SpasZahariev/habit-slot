use chrono::{Datelike, NaiveDate};
use dioxus::prelude::*;
use uuid::Uuid;

use crate::state::AppState;

#[component]
pub fn HabitDetailPage(habit_id: String) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    let habit = match Uuid::parse_str(&habit_id) {
        Ok(id) => app_state.read().habits.iter().find(|h| h.id == id).cloned(),
        Err(_) => None,
    };

    if let Some(habit) = habit {
        let habit_uuid = habit.id;
        let streak = app_state.read().get_streak(habit_uuid);
        let total_days_done = app_state.read().get_total_days_done(habit_uuid);
        let total_completions = app_state.read().get_total_completions(habit_uuid);

        rsx! {
            div {
                class: "habit-detail-page w-full max-w-[420px] mx-auto",
                style: "padding: 16px; padding-bottom: 32px;",

                div {
                    style: "background: #2a1a4e; border-radius: 12px; padding: 20px; margin-bottom: 12px; border: 1px solid rgba(255,45,120,0.2);",

                    div {
                        style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px;",

                        StatBox { label: "Current Streak", value: format!("🔥 {} days", streak.current_streak_days) }
                        StatBox { label: "Longest Streak", value: format!("🏆 {} days", habit.longest_streak) }
                        StatBox { label: "Total Days", value: format!("{} days", total_days_done) }
                        StatBox { label: "Total Ticks", value: format!("{}", total_completions) }
                    }
                }

                CoinRewardSelector { habit_id: habit_uuid, current_reward: habit.coin_reward }

                div {
                    style: "background: #0f0520; border-radius: 8px; padding: 12px;",
                    StreakCalendarInner { habit }
                }

                div {
                    style: "margin-top: 16px; margin-bottom: 24px;",
                    button {
                        onclick: move |_| {
                            app_state.with_mut(|s| s.open_delete_confirm(habit_uuid));
                        },
                        class: "w-full font-pixel",
                        style: "padding: 14px; background: transparent; border: 2px solid #ff2d78; color: #ff2d78; border-radius: 10px; font-size: 0.95rem; cursor: pointer;",
                        "Delete Habit"
                    }
                }
            }

            DeleteConfirmationModal {}
        }
    } else {
        rsx! {
            div {
                class: "text-center py-12 opacity-70",
                p { "Habit not found." }
                button {
                    onclick: move |_| {
                        app_state.with_mut(|s| s.go_habits());
                    },
                    style: "margin-top: 12px; background: #ff2d78; color: #1a0a2e; border: none; border-radius: 8px; padding: 10px 20px; cursor: pointer;",
                    "Back to Habits"
                }
            }
        }
    }
}

#[component]
fn StatBox(label: &'static str, value: String) -> Element {
    rsx! {
        div {
            style: "background: #1a0a2e; border-radius: 8px; padding: 12px;",
            div {
                style: "font-size: 0.7rem; color: rgba(240,230,255,0.5); margin-bottom: 4px;",
                "{label}"
            }
            div {
                style: "color: #f0e6ff; font-size: 1rem;",
                "{value}"
            }
        }
    }
}

/// Horizontal coin reward selector with colored buttons.
#[component]
fn CoinRewardSelector(habit_id: Uuid, current_reward: u32) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    rsx! {
        div {
            style: "background: #2a1a4e; border-radius: 12px; padding: 16px; margin-bottom: 8px; border: 1px solid rgba(255,45,120,0.2);",
            div {
                style: "display: flex; flex-direction: column; gap: 6px; margin-bottom: 3px;",
                label { style: "color: #f0e6ff; font-family: Silkscreen; font-size: 0.85rem;", "Coin Reward" }
                div {
                    style: "display: flex; gap: 6px; max-width: 280px; margin: 0 auto;",
                    for val in [1u32, 3, 5] {
                        button {
                            onclick: move |_| {
                                app_state.with_mut(|s| s.update_habit_coin_reward(habit_id, val));
                            },
                            style: format!(
                                "flex: 1; border-radius: 8px; padding: 2px 6px; font-family: Silkscreen; font-size: 0.95rem; cursor: pointer; border: 2px solid {}; background: #1a0a2e; color: {}; min-width: 90px;",
                                if current_reward == val {
                                    match val { 1 => "#4ade80", 3 => "#a855f7", 5 => "#f97316", _ => "#4ade80" }
                                } else { "transparent" },
                                match val { 1 => "#4ade80", 3 => "#a855f7", 5 => "#f97316", _ => "#4ade80" }
                            ),
                            { format!("{} coin{}", val, if val > 1 { "s" } else { "" }) }
                        }
                    }
                }
            }
        }
    }
}

/// Binary calendar heatmap inner content: green for completed days, dark gray otherwise.
/// Cyan border for today's cell.
#[component]
fn StreakCalendarInner(habit: habit_slot::models::Habit) -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let mut selected_month = use_signal(|| None);

    let today = chrono::Utc::now().naive_utc().date();
    let year = today.year();
    let month = today.month() as u32;

    let (current_year, current_month) = match selected_month.read().as_ref() {
        Some((y, m)) => (*y, *m),
        None => (year, month),
    };

    let days_in_month = get_days_in_month(current_year, current_month);
    let first_day_of_month =
        NaiveDate::from_ymd_opt(current_year, current_month, 1).unwrap_or(today);
    let weekday_offset = first_day_of_weekday(first_day_of_month) as usize;

    let month_name_str = month_name(current_month);
    let is_today_month = current_year == year && current_month == month;
    let today_day = today.day();

    let habit_id = habit.id;
    let completions = app_state.read().completions.clone();

    let completed_dates: std::collections::HashSet<NaiveDate> = completions
        .iter()
        .filter(|c| c.habit_id == habit_id)
        .map(|c| c.date)
        .collect();

    let prev_month = move |_| {
        let (y, m) = match *selected_month.read() {
            Some((y, 1)) => (y - 1, 12),
            Some((y, m)) => (y, m - 1),
            None => (year - 1, 12),
        };
        *selected_month.write() = Some((y, m));
    };

    let next_month = move |_| {
        let (y, m) = match *selected_month.read() {
            Some((y, 12)) => (y + 1, 1),
            Some((y, m)) => (y, m + 1),
            None => (year + 1, 1),
        };
        *selected_month.write() = Some((y, m));
    };

    let cells: Vec<CellStyle> = (0..days_in_month)
        .map(|i| {
            let day = (i + 1) as u32;
            let date = NaiveDate::from_ymd_opt(current_year, current_month, day).unwrap_or(today);
            let is_today = is_today_month && day == today_day;
            let is_future = date > today;
            let completed = !is_future && completed_dates.contains(&date);

            CellStyle {
                bg: if completed { "#4ade80" } else { "#1a1a2e" },
                border: if is_today {
                    "border: 2px solid #00f5d4;"
                } else {
                    ""
                },
            }
        })
        .collect();

    rsx! {
        div {
            style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; gap: 8px;",
            button {
                onclick: prev_month.clone(),
                style: "background: none; border: 1px solid #ff2d78; color: #ff2d78; padding: 4px 12px; border-radius: 4px; cursor: pointer; text-align: left;",
                "Prev"
            }

            span {
                style: "font-size: 0.95rem; color: #ff2d78; font-weight: bold; flex: 1; text-align: center; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                "{month_name_str} {current_year}"
            }

            button {
                onclick: next_month.clone(),
                style: "background: none; border: 1px solid #ff2d78; color: #ff2d78; padding: 4px 12px; border-radius: 4px; cursor: pointer; text-align: right;",
                "Next"
            }
        }

        div {
            style: "display: grid; grid-template-columns: repeat(7, 1fr); gap: 3px;",

            for day_name in ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"] {
                div {
                    style: "text-align: center; font-size: 0.7rem; color: #b8a9d4; padding: 4px 0;",
                    "{day_name}"
                }
            }

            for _ in 0..weekday_offset {
                div {
                    style: "aspect-ratio: 1; border-radius: 3px; background: transparent;",
                }
            }

            for (cell, day_num) in cells.iter().zip(1..=days_in_month) {
                div {
                    style: format!(
                        "aspect-ratio: 1; border-radius: 3px; background: {}; display: flex; align-items: center; justify-content: center; font-size: 0.7rem; color: #ccc; cursor: default; {}",
                        cell.bg, cell.border
                    ),
                    "{day_num}"
                }
            }
        }

        div {
            style: "display: flex; gap: 8px; margin-top: 8px; font-size: 0.7rem; color: #b8a9d4;",

            ColorSwatch { color: "#4ade80", label: "Completed" }
            ColorSwatch { color: "#1a1a2e", label: "Not done" }
        }
    }
}

struct CellStyle {
    bg: &'static str,
    border: &'static str,
}

#[component]
fn ColorSwatch(color: &'static str, label: &'static str) -> Element {
    rsx! {
        span {
            style: "display: inline-flex; align-items: center; gap: 2px;",
            span {
                style: format!(
                    "display: inline-block; width: 10px; height: 10px; border-radius: 2px; background: {};",
                    color
                ),
            }
            "{label}"
        }
    }
}

/// Delete confirmation modal following the RewardModal pattern.
#[component]
pub fn DeleteConfirmationModal() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();

    rsx! {
        if app_state.read().delete_confirm_open {
            div {
                onclick: move |_| {
                    app_state.with_mut(|s| s.close_delete_confirm());
                },
                style: "position: fixed; inset: 0; background: rgba(10,5,20,0.85); display: flex; align-items: center; justify-content: center; z-index: 100;",

                div {
                    onclick: |e| e.stop_propagation(),
                    style: "background: #1a0a2e; border: 2px solid rgba(255,45,120,0.4); border-radius: 16px; padding: 24px; width: 90%; max-width: 360px;",

                    div {
                        style: "display: flex; flex-direction: column; gap: 16px;",

                        p {
                            style: "color: #f0e6ff; font-size: 0.95rem; text-align: center; line-height: 1.4; margin: 0;",
                            "Delete this habit? This will remove all completion history."
                        }

                        div {
                            style: "display: flex; gap: 8px;",

                            button {
                                onclick: move |_| {
                                    app_state.with_mut(|s| s.confirm_delete_habit());
                                },
                                class: "flex-1 font-pixel",
                                style: "background: #ff2d78; color: #1a0a2e; border: none; border-radius: 8px; padding: 10px 16px; font-size: 0.95rem; cursor: pointer;",
                                "Delete"
                            }

                            button {
                                onclick: move |_| {
                                    app_state.with_mut(|s| s.close_delete_confirm());
                                },
                                class: "flex-1 font-pixel",
                                style: "background: #2a1a4e; color: #f0e6ff; border: 1px solid rgba(255,45,120,0.3); border-radius: 8px; padding: 10px 16px; font-size: 0.95rem; cursor: pointer;",
                                "Cancel"
                            }
                        }
                    }
                }
            }
        }
    }
}

fn get_days_in_month(year: i32, month: u32) -> usize {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn first_day_of_weekday(date: NaiveDate) -> u32 {
    date.weekday().num_days_from_sunday()
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    }
}

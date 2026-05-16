use chrono::{Datelike, NaiveDate};
use dioxus::prelude::*;

use habit_slot::models::{CalendarColor, Habit};
use habit_slot::streaks;

use crate::state::AppState;

struct CellData {
    day_text: String,
    style: String,
}

/// Calendar heatmap showing completion history for a single habit.
/// Displays one month at a time with prev/next navigation.
#[component]
pub fn CalendarHeatmap(habit: Habit) -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let mut selected_month = use_signal(|| None);

    let today = chrono::Utc::now().naive_utc().date();
    let year = today.year();
    let month = today.month() as u32;

    let (current_year, current_month) = match selected_month.read().as_ref() {
        Some((y, m)) => (*y, *m),
        None => (year, month),
    };

    let completions = app_state.read().completions.clone();

    let days_in_month = get_days_in_month(current_year, current_month);
    let first_day_of_month =
        NaiveDate::from_ymd_opt(current_year, current_month, 1).unwrap_or(today);
    let weekday_offset = first_day_of_weekday(first_day_of_month) as usize;

    let month_name = month_name(current_month);
    let is_today_month = current_year == year && current_month == month;
    let today_day = today.day();

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

    let cells: Vec<CellData> = (0..days_in_month)
        .map(|i| {
            let day = (i + 1) as u32;
            let date = NaiveDate::from_ymd_opt(current_year, current_month, day).unwrap_or(today);
            let color = streaks::calendar_color(date, &completions, habit.id);
            let is_today = is_today_month && day == today_day;
            CellData {
                day_text: format!("{}", day),
                style: format!(
                    "aspect-ratio: 1; border-radius: 3px; background: {}; \
                     display: flex; align-items: center; justify-content: center; \
                     font-size: 0.7rem; color: #ccc; cursor: default; {}",
                    color.hex(),
                    if is_today {
                        "border: 2px solid #f5c518;"
                    } else {
                        ""
                    }
                ),
            }
        })
        .collect();

    let spacer_count = weekday_offset;

    rsx! {
        div {
            class: "calendar-heatmap",
            style: "margin-top: 12px; background: #0d1117; border-radius: 8px; padding: 12px;",

            div {
                style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;",
                button {
                    onclick: prev_month.clone(),
                    style: "background: none; border: 1px solid #f5c518; color: #f5c518; padding: 4px 12px; border-radius: 4px; cursor: pointer;",
                    "← Prev"
                }

                span {
                    style: "font-size: 0.95rem; color: #f5c518; font-weight: bold;",
                    "{month_name} {current_year}"
                }

                button {
                    onclick: next_month.clone(),
                    style: "background: none; border: 1px solid #f5c518; color: #f5c518; padding: 4px 12px; border-radius: 4px; cursor: pointer;",
                    "Next →"
                }
            }

            div {
                class: "calendar-grid",
                style: "display: grid; grid-template-columns: repeat(7, 1fr); gap: 3px;",

                for day_name in ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"] {
                    div {
                        style: "text-align: center; font-size: 0.7rem; color: #888; padding: 4px 0;",
                        "{day_name}"
                    }
                }

                for _ in 0..spacer_count {
                    div {
                        style: "aspect-ratio: 1; border-radius: 3px; background: transparent;",
                    }
                }

                for cell in cells {
                    div {
                        style: cell.style,
                        "{cell.day_text}"
                    }
                }
            }

            div {
                class: "calendar-legend",
                style: "display: flex; gap: 8px; margin-top: 8px; font-size: 0.7rem; color: #888;",

                span { "Streak:" }
                ColorSwatch { color: CalendarColor::Empty, label: "-" }
                ColorSwatch { color: CalendarColor::Low, label: "1-3" }
                ColorSwatch { color: CalendarColor::Mid, label: "4-9" }
                ColorSwatch { color: CalendarColor::High, label: "10+" }
            }
        }
    }
}

#[component]
fn ColorSwatch(color: CalendarColor, label: &'static str) -> Element {
    rsx! {
        span {
            style: format!(
                "display: inline-flex; align-items: center; gap: 2px;",
            ),
            span {
                style: format!(
                    "display: inline-block; width: 10px; height: 10px; border-radius: 2px; background: {};",
                    color.hex()
                ),
            }
            "{label}"
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

/// Get day of week: 0=Sunday, 1=Monday, ..., 6=Saturday
fn first_day_of_weekday(date: NaiveDate) -> u32 {
    let dow = date.weekday().num_days_from_sunday();
    dow
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

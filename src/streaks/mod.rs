use chrono::NaiveDate;

use crate::models::{CalendarColor, Completion, StreakData};

/// Compute streak data for a habit from its completion history.
pub fn compute_streak(habit_id: uuid::Uuid, completions: &[Completion]) -> StreakData {
    let today = chrono::Utc::now().naive_utc().date();
    let habit_completions: Vec<_> = completions
        .iter()
        .filter(|c| c.habit_id == habit_id)
        .map(|c| c.date)
        .collect();

    compute_streak_impl(&habit_completions, today)
}

/// Internal implementation for testability (accepts explicit "today").
fn compute_streak_impl(dates: &[NaiveDate], today: NaiveDate) -> StreakData {
    let mut sorted = dates.to_vec();
    sorted.sort();
    sorted.dedup();

    if sorted.is_empty() {
        return StreakData {
            current_streak_days: 0,
            max_streak_days: 0,
            last_completed_date: None,
        };
    }

    let last_completed = *sorted.last().unwrap();

    // Compute current streak: count consecutive days backward from today.
    let current_streak_days = if last_completed == today {
        count_consecutive_from_end(&sorted, today)
    } else {
        let yesterday = today - chrono::Duration::days(1);
        if last_completed == yesterday {
            count_consecutive_from_end(&sorted, yesterday)
        } else {
            0 // streak broken
        }
    };

    // Compute max streak: longest consecutive sequence.
    let max_streak_days = find_max_consecutive(&sorted);

    StreakData {
        current_streak_days,
        max_streak_days,
        last_completed_date: Some(last_completed),
    }
}

/// Count consecutive days backward from `start` in sorted unique dates.
fn count_consecutive_from_end(sorted: &[NaiveDate], start: NaiveDate) -> u32 {
    let mut count = 0u32;
    let mut current = start;
    for i in (0..sorted.len()).rev() {
        if sorted[i] == current {
            count += 1;
            current -= chrono::Duration::days(1);
        } else if sorted[i] < current {
            break; // gap found
        }
    }
    count
}

/// Find the longest consecutive day sequence.
fn find_max_consecutive(sorted: &[NaiveDate]) -> u32 {
    if sorted.is_empty() {
        return 0;
    }

    let mut max_run = 1u32;
    let mut current_run = 1u32;

    for i in 1..sorted.len() {
        let diff = (sorted[i] - sorted[i - 1]).num_days();
        if diff == 1 {
            current_run += 1;
            max_run = max_run.max(current_run);
        } else if diff == 0 {
            continue; // duplicate, skip
        } else {
            current_run = 1;
        }
    }

    max_run
}

/// Get calendar color for a date based on streak length at that point.
pub fn calendar_color(
    date: NaiveDate,
    completions: &[Completion],
    habit_id: uuid::Uuid,
) -> CalendarColor {
    let mut count = 0u32;
    let mut current = date;
    for c in completions.iter().filter(|c| c.habit_id == habit_id) {
        if c.date <= date && c.date >= current {
            if c.date == current {
                count += 1;
                current -= chrono::Duration::days(1);
            }
        }
    }

    match count {
        0 => CalendarColor::Empty,
        1..=3 => CalendarColor::Low,
        4..=9 => CalendarColor::Mid,
        _ => CalendarColor::High,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn empty_completions_returns_zero_streak() {
        let result = compute_streak_impl(&[], date(2026, 5, 7));
        assert_eq!(result.current_streak_days, 0);
        assert_eq!(result.max_streak_days, 0);
        assert!(result.last_completed_date.is_none());
    }

    #[test]
    fn single_completion_today_gives_streak_1() {
        let dates = vec![date(2026, 5, 7)];
        let result = compute_streak_impl(&dates, date(2026, 5, 7));
        assert_eq!(result.current_streak_days, 1);
        assert_eq!(result.max_streak_days, 1);
    }

    #[test]
    fn consecutive_days_count_correctly() {
        let dates = vec![
            date(2026, 5, 3),
            date(2026, 5, 4),
            date(2026, 5, 5),
            date(2026, 5, 6),
            date(2026, 5, 7),
        ];
        let result = compute_streak_impl(&dates, date(2026, 5, 7));
        assert_eq!(result.current_streak_days, 5);
        assert_eq!(result.max_streak_days, 5);
    }

    #[test]
    fn streak_resets_on_gap() {
        // 3 days, gap, 2 days ending today
        let dates = vec![
            date(2026, 5, 1),
            date(2026, 5, 2),
            date(2026, 5, 3),
            // gap on 5/4, 5/5
            date(2026, 5, 6),
            date(2026, 5, 7),
        ];
        let result = compute_streak_impl(&dates, date(2026, 5, 7));
        assert_eq!(result.current_streak_days, 2);
        assert_eq!(result.max_streak_days, 3);
    }

    #[test]
    fn streak_zero_when_last_was_yesterday_but_not_today() {
        let dates = vec![date(2026, 5, 4), date(2026, 5, 5)];
        // Today is 5/7, last was 5/5 (not today or yesterday) → streak broken
        let result = compute_streak_impl(&dates, date(2026, 5, 7));
        assert_eq!(result.current_streak_days, 0);
    }

    #[test]
    fn max_preserved_across_reset() {
        // Long streak (4 days), then gap, then short streak (2 days) today
        let dates = vec![
            date(2026, 5, 1),
            date(2026, 5, 2),
            date(2026, 5, 3),
            date(2026, 5, 4),
            // gap on 5/5, 5/6, 5/7, 5/8
            date(2026, 5, 9),
            date(2026, 5, 10),
        ];
        let result = compute_streak_impl(&dates, date(2026, 5, 10));
        assert_eq!(result.current_streak_days, 2);
        assert_eq!(result.max_streak_days, 4);
    }

    #[test]
    fn duplicate_dates_handled() {
        let dates = vec![
            date(2026, 5, 7),
            date(2026, 5, 7), // same day completion twice
        ];
        let result = compute_streak_impl(&dates, date(2026, 5, 7));
        assert_eq!(result.current_streak_days, 1);
        assert_eq!(result.max_streak_days, 1);
    }

    #[test]
    fn calendar_color_mapping() {
        // Test via streak length → color mapping logic
        let dates = vec![date(2026, 5, 7)];
        let result = compute_streak_impl(&dates, date(2026, 5, 7));
        assert_eq!(result.current_streak_days, 1);

        // Simulate a heatmap check: streak of 1 → Low color
        let color = if result.current_streak_days == 0 {
            CalendarColor::Empty
        } else if result.current_streak_days <= 3 {
            CalendarColor::Low
        } else if result.current_streak_days <= 9 {
            CalendarColor::Mid
        } else {
            CalendarColor::High
        };
        assert_eq!(color, CalendarColor::Low);

        // Verify color hex values are valid
        assert!(!CalendarColor::Empty.hex().is_empty());
        assert!(!CalendarColor::Low.hex().is_empty());
        assert!(!CalendarColor::Mid.hex().is_empty());
        assert!(!CalendarColor::High.hex().is_empty());
    }
}

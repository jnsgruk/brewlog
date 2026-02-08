use chrono::{DateTime, Datelike, Utc};

/// Format a datetime as a human-readable relative time string.
///
/// Examples: "Just now", "5m ago", "3h ago", "Yesterday", "4d ago", "2w ago",
/// "Mar 15" (same year), "Mar 15, 2024" (different year).
pub fn format_relative_time(dt: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let delta = now.signed_duration_since(dt);
    let secs = delta.num_seconds();

    if secs < 60 {
        return "Just now".to_string();
    }

    let mins = delta.num_minutes();
    if mins < 60 {
        return format!("{mins}m ago");
    }

    let hours = delta.num_hours();
    if hours < 24 {
        return format!("{hours}h ago");
    }

    let days = delta.num_days();
    if days == 1 {
        return "Yesterday".to_string();
    }
    if days < 7 {
        return format!("{days}d ago");
    }

    let weeks = days / 7;
    if days < 30 {
        return format!("{weeks}w ago");
    }

    if dt.year() == now.year() {
        dt.format("%b %d").to_string()
    } else {
        dt.format("%b %d, %Y").to_string()
    }
}

/// Format a weight in grams for display. Uses "g" up to 999g, "kg" for 1000g+.
///
/// Whole-gram values omit the decimal ("250g"), fractional values show one
/// decimal place ("15.5g"). Kilogram values always show one decimal ("1.5kg").
pub fn format_weight(grams: f64) -> String {
    if grams >= 1000.0 {
        format!("{:.1}kg", grams / 1000.0)
    } else if (grams - grams.round()).abs() < 0.05 {
        format!("{grams:.0}g")
    } else {
        format!("{grams:.1}g")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn utc(y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, min, s).unwrap()
    }

    #[test]
    fn just_now_zero_seconds() {
        let now = utc(2025, 6, 1, 12, 0, 0);
        assert_eq!(format_relative_time(now, now), "Just now");
    }

    #[test]
    fn just_now_59_seconds() {
        let dt = utc(2025, 6, 1, 12, 0, 0);
        let now = utc(2025, 6, 1, 12, 0, 59);
        assert_eq!(format_relative_time(dt, now), "Just now");
    }

    #[test]
    fn minutes_boundary_60_seconds() {
        let dt = utc(2025, 6, 1, 12, 0, 0);
        let now = utc(2025, 6, 1, 12, 1, 0);
        assert_eq!(format_relative_time(dt, now), "1m ago");
    }

    #[test]
    fn minutes_ago() {
        let dt = utc(2025, 6, 1, 12, 0, 0);
        let now = utc(2025, 6, 1, 12, 45, 0);
        assert_eq!(format_relative_time(dt, now), "45m ago");
    }

    #[test]
    fn hours_boundary_60_minutes() {
        let dt = utc(2025, 6, 1, 12, 0, 0);
        let now = utc(2025, 6, 1, 13, 0, 0);
        assert_eq!(format_relative_time(dt, now), "1h ago");
    }

    #[test]
    fn hours_ago() {
        let dt = utc(2025, 6, 1, 6, 0, 0);
        let now = utc(2025, 6, 1, 12, 0, 0);
        assert_eq!(format_relative_time(dt, now), "6h ago");
    }

    #[test]
    fn yesterday_exactly_24h() {
        let dt = utc(2025, 6, 1, 12, 0, 0);
        let now = utc(2025, 6, 2, 12, 0, 0);
        assert_eq!(format_relative_time(dt, now), "Yesterday");
    }

    #[test]
    fn yesterday_36h() {
        let dt = utc(2025, 6, 1, 0, 0, 0);
        let now = utc(2025, 6, 2, 12, 0, 0);
        assert_eq!(format_relative_time(dt, now), "Yesterday");
    }

    #[test]
    fn days_ago_2() {
        let dt = utc(2025, 6, 1, 12, 0, 0);
        let now = utc(2025, 6, 3, 12, 0, 0);
        assert_eq!(format_relative_time(dt, now), "2d ago");
    }

    #[test]
    fn days_ago_6() {
        let dt = utc(2025, 6, 1, 12, 0, 0);
        let now = utc(2025, 6, 7, 12, 0, 0);
        assert_eq!(format_relative_time(dt, now), "6d ago");
    }

    #[test]
    fn weeks_ago_1() {
        let dt = utc(2025, 6, 1, 12, 0, 0);
        let now = utc(2025, 6, 8, 12, 0, 0);
        assert_eq!(format_relative_time(dt, now), "1w ago");
    }

    #[test]
    fn weeks_ago_3() {
        let dt = utc(2025, 6, 1, 12, 0, 0);
        let now = utc(2025, 6, 22, 12, 0, 0);
        assert_eq!(format_relative_time(dt, now), "3w ago");
    }

    #[test]
    fn same_year_absolute_date() {
        let dt = utc(2025, 3, 15, 10, 0, 0);
        let now = utc(2025, 6, 1, 12, 0, 0);
        assert_eq!(format_relative_time(dt, now), "Mar 15");
    }

    #[test]
    fn different_year_absolute_date() {
        let dt = utc(2024, 3, 15, 10, 0, 0);
        let now = utc(2025, 6, 1, 12, 0, 0);
        assert_eq!(format_relative_time(dt, now), "Mar 15, 2024");
    }

    #[test]
    fn boundary_at_30_days() {
        let dt = utc(2025, 5, 2, 12, 0, 0);
        let now = utc(2025, 6, 1, 12, 0, 0);
        assert_eq!(format_relative_time(dt, now), "May 02");
    }

    #[test]
    fn future_timestamp_returns_just_now() {
        let dt = utc(2025, 6, 1, 13, 0, 0);
        let now = utc(2025, 6, 1, 12, 0, 0);
        assert_eq!(format_relative_time(dt, now), "Just now");
    }

    // --- format_weight tests ---

    #[test]
    fn weight_zero() {
        assert_eq!(format_weight(0.0), "0g");
    }

    #[test]
    fn weight_whole_grams() {
        assert_eq!(format_weight(15.0), "15g");
        assert_eq!(format_weight(250.0), "250g");
    }

    #[test]
    fn weight_fractional_grams() {
        assert_eq!(format_weight(15.5), "15.5g");
        assert_eq!(format_weight(234.3), "234.3g");
    }

    #[test]
    fn weight_boundary_999() {
        assert_eq!(format_weight(999.0), "999g");
        assert_eq!(format_weight(999.9), "999.9g");
    }

    #[test]
    fn weight_boundary_1000() {
        assert_eq!(format_weight(1000.0), "1.0kg");
    }

    #[test]
    fn weight_kilograms() {
        assert_eq!(format_weight(1500.0), "1.5kg");
        assert_eq!(format_weight(2345.6), "2.3kg");
        assert_eq!(format_weight(10000.0), "10.0kg");
    }
}

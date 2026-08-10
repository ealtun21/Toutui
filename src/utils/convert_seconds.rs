pub fn convert_seconds(vec_seconds: Vec<f64>) -> Vec<String> {
    vec_seconds
        .iter()
        .map(|&s| {
            let total_minutes = (s / 60.0).round() as i64;
            let hours = total_minutes / 60;
            let minutes = total_minutes % 60;

            if hours == 0 {
                format!("{}m", minutes)
            } else if minutes == 0 {
                format!("{}h", hours)
            } else {
                format!("{}h{}m", hours, minutes)
            }
        })
        .collect()
}

/// Writes a place in a media, with the seconds. See T-24.
///
/// `convert_seconds` rounds to the minute, therefore a bookmark at 12 minutes
/// 30 seconds and a bookmark at 12 minutes 45 seconds give one text. A place
/// must name the second: the user goes back to it.
///
/// A media of less than one hour gives `MM:SS`, and a longer media gives
/// `H:MM:SS`.
pub fn clock(seconds: f64) -> String {
    let whole = if seconds.is_finite() && seconds > 0.0 {
        seconds.round() as i64
    } else {
        0
    };

    let hours = whole / 3600;
    let minutes = (whole % 3600) / 60;
    let rest = whole % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, rest)
    } else {
        format!("{:02}:{:02}", minutes, rest)
    }
}

pub fn convert_seconds_for_prg(duration: f64, current_time: f64) -> String {
    let time_left_s = duration - current_time;
    let total_minutes = (time_left_s / 60.0).round() as i64;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;

    if current_time == 0.0 {
        String::new()
    } else if hours == 0 {
        format!("{}m left,", minutes)
    } else if minutes == 0 {
        format!("{}h left,", hours)
    } else {
        format!("{}h{}m left,", hours, minutes)
    }
}

#[cfg(test)]
mod tests_of_the_clock {
    use super::clock;

    #[test]
    fn a_short_place_gives_the_minutes_and_the_seconds() {
        assert_eq!(clock(0.0), "00:00");
        assert_eq!(clock(5.0), "00:05");
        assert_eq!(clock(750.0), "12:30");
        assert_eq!(clock(3599.0), "59:59");
    }

    #[test]
    fn a_long_place_gives_the_hours() {
        assert_eq!(clock(3600.0), "1:00:00");
        assert_eq!(clock(7845.0), "2:10:45");
    }

    /// Two places of the same minute must give two texts. The user goes back
    /// to a place, therefore the second matters.
    #[test]
    fn two_places_of_one_minute_give_two_texts() {
        assert_ne!(clock(750.0), clock(765.0));
    }

    #[test]
    fn a_place_that_is_not_valid_gives_zero() {
        assert_eq!(clock(-10.0), "00:00");
        assert_eq!(clock(f64::NAN), "00:00");
        assert_eq!(clock(f64::INFINITY), "00:00");
    }
}

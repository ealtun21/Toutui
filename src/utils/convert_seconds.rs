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

/// The time that is left of a media, for the panel of a line. See T-289.
///
/// **A time that is left of less than zero is no time at all** (T-289). The
/// function took the two numbers of its caller and it wrote the difference of
/// them, and each of the two roads below gave a negative difference:
///
/// - **a length that the server did not give is not a length of 0** (T-180):
///   the three callers of `src/app.rs` read the length of the media with
///   `length.unwrap_or(0.0)`, therefore a media of no length gave
///   `0 - the place of the user`. The measurement of 2026-08-16, of the real
///   program v0.8.117 against the sandbox, with the `audioFile` of the first
///   episode of `Arthur Gordon Pym` away: the panel of that line said
///   `Duration: N/A` and `Progress: 22%, -1m left, Not finished`, and the panel
///   therefore said a time of a length that it named a value it does not have;
/// - **the place of the user can stand past the length of the media**: the
///   server holds a `duration` of its own beside the `duration` of the audio
///   file, and the two of them do not agree. The same measurement, with the
///   place of `Chapter 02` at 6000 seconds of a file of 2336 seconds: the panel
///   said `Duration: 39m` and `Progress: 100%, -1h-1m left, Finished`.
///
/// The second road holds a fault of the form too: `/` and `%` of Rust go toward
/// zero, therefore a difference of -61 minutes gave `-1h` and `-1m` together,
/// and **the program says a time in one form** (the rule of T-284).
///
/// **The two neighbours of this function hold the rule already**:
/// `crate::player::integrated::player_info::the_left_of_the_row` takes
/// `saturating_sub` of two whole numbers and it says `N/A` for a length of 0,
/// and `crate::logic::queue::the_time_of_the_line` writes the difference inside
/// a guard of `place < length`.
///
/// The rules of this function now:
///
/// - a length that is not more than 0 is a length that the program does not
///   have, therefore it says no time at all, as a place of 0 says none;
/// - a place that stands at the length of the media or past it says `0m left,`,
///   because the time that is left of a media is never less than zero.
pub fn convert_seconds_for_prg(duration: f64, current_time: f64) -> String {
    // **A length that the server did not give is a length of 0** (T-180), and
    // the program can make no time that is left of it.
    if duration.is_nan() || duration <= 0.0 {
        return String::new();
    }

    // **The time that is left is never less than zero** (T-289): the place of
    // the user can stand past the length of the media.
    let time_left_s = (duration - current_time).max(0.0);
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

use crate::api::me::get_media_progress::Root;

// no need to handle null values here (there are handeled in `app.rs`) //

pub async fn collect_progress_percentage_book(root: &Root) -> String {
    format!("{}", (root.progress * 100.0).round() as i64)
}

pub async fn collect_is_finished_book(item: &Root) -> String {
    if item.is_finished {
        "Finished".to_string()
    } else {
        "Not finished".to_string()
    }
}

pub async fn collect_current_time_prg(item: &Root) -> f64 {
    item.current_time
}

/// The day when the user started this media, in the words of the design. See
/// T-328.
///
/// **The answer of the account holds the day of the start of every media**
/// (`startedAt` of a row of `mediaProgress` of `GET /api/me`), and the panel 5
/// of the design names it: `Started  14 Aug 2026`. The measurement of the
/// sandbox of 2026-08-16 gave `1786905843790` for `A Long Test Book`, and no
/// view of the program said one day of one media.
///
/// **A media that the user never started holds no day**: the server gives 0 for
/// such a row, and the rule of the panel is the rule of T-325 — a fact that the
/// server did not give takes no line at all.
///
/// The day comes in the time of the machine of the user, because the user reads
/// the day that they started the media and not the day of the server.
pub fn the_day_of_the_start(item: &Root) -> String {
    the_day_of(item.started_at)
}

/// The words of a moment of the server, in milliseconds. See T-328.
///
/// The function is pure, therefore a test needs no server.
fn the_day_of(millis: i64) -> String {
    use chrono::TimeZone;

    if millis <= 0 {
        return String::new();
    }

    match chrono::Local.timestamp_millis_opt(millis).single() {
        Some(moment) => moment.format("%-d %b %Y").to_string(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The day of the start says the day, the month, and the year, and a media
    /// with no day of a start says nothing at all.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_day_of_the_start_says_the_day_of_the_user() {
        // The measurement of the sandbox of 2026-08-16, of `A Long Test Book`.
        let words = the_day_of(1786905843790);
        assert!(
            words.ends_with(" Aug 2026"),
            "the day of the start says the month and the year: {words}"
        );
        assert!(
            !words.starts_with('0'),
            "the day of the start holds no zero at its start: {words}"
        );

        // A media that the user never started, and a value that no moment of
        // the machine holds.
        assert_eq!(the_day_of(0), "");
        assert_eq!(the_day_of(-1), "");
        assert_eq!(the_day_of(i64::MAX), "");
    }
}

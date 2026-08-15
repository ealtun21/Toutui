//! The place of the user of the panel of a line. See T-239.
//!
//! **A panel that says a value of a media that plays reads the engine of this
//! program** (T-238 gave that rule to the line of the view of the queue). The
//! panel of a line takes its percent, its time that is left, and its mark of
//! the end from the box of a request of the server, and the playback of this
//! same program moves the media away from that answer at each second.
//!
//! The measurement of 2026-08-15: the server held `A Book Of Many Hours` at
//! 10800 seconds of 28800, the user played it with the key `l` of the Home
//! view, and 75 seconds later one frame of one screen said two places of that
//! one media:
//!
//! ```text
//! ➤ ▶   A Book Of Many Hours
//! Author: Many Hours Author - Year: N/A - Duration: 8h
//! Progress: 37%, 5h left, Not finished
//!            ▶ 4:13:12 / 8:00:00 | Elapsed: 4:13:12 | Left: 3:46:48 (53%)
//! ```
//!
//! **A live message of the server is the second road to a newer place**
//! (T-240). The measurement of 2026-08-15: the server held that same book at
//! 10800 seconds of 28800 with the percent 52, no playback of the program held
//! it, and a second client of the account moved it to 21600 seconds with the
//! percent 75. The line took the message at the next frame (T-47), and the
//! panel of that same line kept the answer of the request of the start:
//!
//! ```text
//! ➤ 75% A Book Of Many Hours
//! Author: Many Hours Author - Year: N/A - Duration: 8h
//! Progress: 52%, 5h left, Not finished
//! ```

/// The three values of the panel of a line that names a place of the user.
///
/// The panel of a book of the Home view says the three of them, and the panel
/// of an episode of a podcast says the percent and the mark of the end alone:
/// the format of that panel names no time that is left (T-228 and T-229).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThePlaceOfThePanel {
    /// The percent of the media, with no sign of the percent.
    pub percent: String,

    /// The time that is left of the media, in the words of
    /// `crate::utils::convert_seconds::convert_seconds_for_prg`.
    pub the_time_that_is_left: String,

    /// "Finished" or "Not finished", in the words of
    /// `crate::api::utils::collect_get_media_progress::collect_is_finished_book`.
    pub the_end: String,
}

/// Gives the place of the panel of one line. See T-239.
///
/// **The place of the row is the place of the moment of the request of the
/// view**, and the playback of this same program takes no road back to it: the
/// program writes the place of the user to the server, and the server sends no
/// message of that place to the client that wrote it (T-235). Therefore the
/// engine of this program is the truth of the place of the media that plays,
/// and it costs no request at all.
///
/// **A place of 0 is a playback that did not begin**: the media loads in that
/// moment, and the panel then keeps the place of the row (T-238).
///
/// **A length of 0, and a length that the server did not give, are one thing**
/// (T-180): the program cannot make a percent or a time that is left of a place
/// with no length, therefore the panel keeps the values of the row.
///
/// **The mark of the end of the row belongs to the place of the row** (T-238):
/// a media that plays stands at the place of the engine, therefore a media that
/// the user finished and that plays again says the percent and the time of that
/// place, and not the mark of its end.
///
/// **A live message of the server is newer than the request of the view**
/// (T-240): a different client of the same account moved in a media that no
/// playback of this program holds, and the server sends the message after it
/// wrote the value (T-47). The line of that media shows the new percent at the
/// next frame already, therefore the panel of that same line must not say the
/// value of the request.
///
/// The function is pure, therefore a test needs no server and no screen.
pub fn the_place_of_the_panel(
    plays_now: bool,
    the_place_of_the_playback: Option<f64>,
    the_message_of_the_server: Option<&crate::api::live::Progress>,
    the_length_of_the_media: Option<f64>,
    the_percent_of_the_row: &str,
    the_time_of_the_row: &str,
    the_end_of_the_row: &str,
) -> ThePlaceOfThePanel {
    let of_the_row = || match the_message_of_the_server {
        Some(live) => ThePlaceOfThePanel {
            percent: live.percent.clone(),
            the_time_that_is_left: the_time_of_a_message(live, the_length_of_the_media)
                .unwrap_or_else(|| the_time_of_the_row.to_string()),
            the_end: live.finished.clone(),
        },
        None => ThePlaceOfThePanel {
            percent: the_percent_of_the_row.to_string(),
            the_time_that_is_left: the_time_of_the_row.to_string(),
            the_end: the_end_of_the_row.to_string(),
        },
    };

    let place = match the_place_of_the_playback {
        Some(place) if plays_now && place > 0.0 => place,
        _ => return of_the_row(),
    };

    let length = match the_length_of_the_media {
        Some(length) if length > 0.0 => length,
        _ => return of_the_row(),
    };

    ThePlaceOfThePanel {
        percent: format!("{}", (place / length * 100.0).round() as i64),
        the_time_that_is_left: crate::utils::convert_seconds::convert_seconds_for_prg(
            length, place,
        ),
        the_end: "Not finished".to_string(),
    }
}

/// The time that is left of the place of a live message of the server.
/// See T-240.
///
/// The message holds the place of the user in seconds, as a text (T-235). A
/// place that is no number, and a length that the server did not give, each
/// give nothing at all, and the panel then keeps the time of the row.
///
/// **A place of 0 is the start of the media here**, and not a value that the
/// message did not give: the line of the view of the queue reads that same
/// value in that same way (T-234). The panel of a media that the user did not
/// begin names no time that is left, as `convert_seconds_for_prg` says.
fn the_time_of_a_message(
    live: &crate::api::live::Progress,
    the_length_of_the_media: Option<f64>,
) -> Option<String> {
    let place = live.place.trim().parse::<f64>().ok()?;
    let length = the_length_of_the_media.filter(|length| *length > 0.0)?;

    Some(crate::utils::convert_seconds::convert_seconds_for_prg(
        length, place,
    ))
}

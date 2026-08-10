//! The statistics of the user. See T-24.
//!
//! `GET /api/me/listening-stats` gives every number that the screen needs, and
//! the program sends one request only. A measurement against an Audiobookshelf
//! 2.36.0 on 2026-08-11 gives this shape:
//!
//! ```json
//! {
//!   "totalTime": 281,
//!   "items": { "<id>": { "id": "<id>", "timeListening": 276,
//!                        "mediaMetadata": { "title": "A Long Test Book",
//!                                           "authors": [ { "name": "..." } ] } } },
//!   "days": { "2026-08-10": 281 },
//!   "dayOfWeek": { "Monday": 281 },
//!   "today": 281,
//!   "recentSessions": [ { "displayTitle": "...", "displayAuthor": "...",
//!                         "date": "2026-08-10", "timeListening": 5 } ]
//! }
//! ```
//!
//! **`items` is a map, and not a list.** The key is the identity of the media,
//! and the same identity comes again inside the value.
//!
//! Every time is a number of seconds. The functions of this file make the text
//! for the screen, and they are pure. Therefore a test examines them with no
//! terminal and with no server.

use crate::api::client::error::ApiError;
use crate::api::client::ApiClient;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

/// The seven days of the week, in the sequence of the server.
///
/// The server writes the name in English, and it writes it in this form. The
/// screen shows the seven days always, therefore a day with no time keeps its
/// line and the user sees the shape of their week.
pub const DAYS_OF_THE_WEEK: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListeningStats {
    /// The time of every session of this account, in seconds.
    #[serde(default)]
    pub total_time: f64,
    /// The time of this day, in seconds.
    #[serde(default)]
    pub today: f64,
    /// The time of each day. The key is a date of the form `2026-08-10`.
    ///
    /// A `BTreeMap` keeps the keys in sequence. A date of that form goes in
    /// the sequence of the time when it goes in the sequence of the letters,
    /// therefore the program needs no calendar.
    #[serde(default)]
    pub days: BTreeMap<String, f64>,
    /// The time of each day of the week. The key is a name of
    /// `DAYS_OF_THE_WEEK`.
    #[serde(default)]
    pub day_of_week: HashMap<String, f64>,
    /// The time of each media. The key is the identity of the media.
    #[serde(default)]
    pub items: HashMap<String, ItemStat>,
    #[serde(default)]
    pub recent_sessions: Vec<Session>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemStat {
    #[serde(default)]
    pub time_listening: f64,
    #[serde(default)]
    pub media_metadata: Option<MediaMetadata>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaMetadata {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub authors: Vec<Author>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Author {
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    #[serde(default)]
    pub display_title: Option<String>,
    #[serde(default)]
    pub display_author: Option<String>,
    /// The date of the session, of the form `2026-08-10`.
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub time_listening: f64,
}

/// Asks the server for the statistics of this account.
pub async fn get_listening_stats(client: &ApiClient) -> Result<ListeningStats, ApiError> {
    client.get_json("/api/me/listening-stats").await
}

/// One media of the list of the media that the user played most.
#[derive(Debug, Clone, PartialEq)]
pub struct TopItem {
    pub title: String,
    pub author: String,
    pub seconds: f64,
}

/// Gives the media that the user played most, the largest time first.
///
/// A map has no sequence. Therefore this function makes one, and the sequence
/// is the same for each frame: the time first, and the title after it. Two
/// media of the same time then keep one place, and the screen does not move.
pub fn top_items(stats: &ListeningStats, count: usize) -> Vec<TopItem> {
    let mut all: Vec<TopItem> = stats
        .items
        .values()
        .map(|item| {
            let metadata = item.media_metadata.as_ref();

            let title = metadata
                .and_then(|one| one.title.clone())
                .unwrap_or_else(|| "A media with no title".to_string());

            let author = metadata
                .map(|one| {
                    one.authors
                        .iter()
                        .filter_map(|author| author.name.clone())
                        .collect::<Vec<String>>()
                        .join(", ")
                })
                .unwrap_or_default();

            TopItem {
                title,
                author,
                seconds: item.time_listening,
            }
        })
        .collect();

    all.sort_by(|a, b| {
        b.seconds
            .partial_cmp(&a.seconds)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.title.cmp(&b.title))
    });

    all.truncate(count);
    all
}

/// Gives the last days that the server knows, the oldest first.
///
/// The server gives a day only when the user played a media on that day.
/// Therefore this function does not make an empty day: it gives the days that
/// exist, and no more than `count` of them.
pub fn last_days(stats: &ListeningStats, count: usize) -> Vec<(String, f64)> {
    let mut all: Vec<(String, f64)> = stats
        .days
        .iter()
        .map(|(day, seconds)| (day.clone(), *seconds))
        .collect();

    if all.len() > count {
        all.drain(..all.len() - count);
    }

    all
}

/// Gives the seven days of the week, Monday first.
///
/// A day that the server does not name gives zero. The screen then shows the
/// seven lines always.
pub fn week(stats: &ListeningStats) -> Vec<(&'static str, f64)> {
    DAYS_OF_THE_WEEK
        .iter()
        .map(|day| (*day, stats.day_of_week.get(*day).copied().unwrap_or(0.0)))
        .collect()
}

/// Gives the largest value of a group of times.
///
/// The bar of each line uses this value as its full width. A group with no
/// value, and a group of zeros, give zero.
pub fn largest(values: &[f64]) -> f64 {
    values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .fold(0.0_f64, f64::max)
}

/// Writes a time for a person.
///
/// The value is a number of seconds. A time of more than one hour shows the
/// hours and the minutes. A shorter time shows the minutes and the seconds.
pub fn human_time(seconds: f64) -> String {
    let seconds = if seconds.is_finite() && seconds > 0.0 {
        seconds.round() as i64
    } else {
        0
    };

    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let rest = seconds % 60;

    if hours > 0 {
        format!("{} h {:02} min", hours, minutes)
    } else if minutes > 0 {
        format!("{} min {:02} s", minutes, rest)
    } else {
        format!("{} s", rest)
    }
}

/// The eight parts of a full block, from one eighth to seven eighths.
const PARTS: [char; 7] = ['▏', '▎', '▍', '▌', '▋', '▊', '▉'];

/// Makes a bar of characters.
///
/// `width` is the number of columns of a full bar. The bar uses a part of a
/// block for the remainder, therefore a small value is still visible. A value
/// above zero always gives one mark at least.
///
/// The program needs no new crate for this work.
pub fn bar(value: f64, largest: f64, width: usize) -> String {
    if width == 0 || largest <= 0.0 || !value.is_finite() || value <= 0.0 {
        return String::new();
    }

    let part = (value / largest).clamp(0.0, 1.0);
    let mut eighths = (part * (width * 8) as f64).round() as usize;

    // A value above zero must give a mark. A bar of nothing would say that
    // the user played nothing on that day, and that is not true.
    if eighths == 0 {
        eighths = 1;
    }

    let full = (eighths / 8).min(width);
    let rest = eighths % 8;

    let mut out = "█".repeat(full);

    if rest > 0 && full < width {
        out.push(PARTS[rest - 1]);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The answer of the sandbox of `docs/TEST-SERVER.md`, measured on
    /// 2026-08-11 against an Audiobookshelf 2.36.0.
    fn the_answer_of_the_server() -> ListeningStats {
        serde_json::from_value(serde_json::json!({
            "totalTime": 281,
            "items": {
                "ac365248": {
                    "id": "ac365248",
                    "timeListening": 5,
                    "mediaMetadata": {
                        "title": "Multi File Test Book",
                        "authors": [ { "id": "f49b0437", "name": "Test Author" } ],
                        "narrators": [],
                        "series": []
                    }
                },
                "9a671047": {
                    "id": "9a671047",
                    "timeListening": 276,
                    "mediaMetadata": {
                        "title": "A Long Test Book",
                        "authors": [ { "id": "cc5891d3", "name": "Long Author" } ]
                    }
                }
            },
            "days": { "2026-08-10": 281 },
            "dayOfWeek": { "Monday": 281 },
            "today": 281,
            "recentSessions": [ {
                "id": "6a8b29f8",
                "displayTitle": "Multi File Test Book",
                "displayAuthor": "Test Author",
                "date": "2026-08-10",
                "dayOfWeek": "Monday",
                "timeListening": 5,
                "startTime": 30,
                "currentTime": 5
            } ]
        }))
        .expect("the answer of the server must read")
    }

    #[test]
    fn the_answer_of_a_real_server_reads() {
        let stats = the_answer_of_the_server();

        assert_eq!(stats.total_time, 281.0);
        assert_eq!(stats.today, 281.0);
        assert_eq!(stats.days.get("2026-08-10"), Some(&281.0));
        assert_eq!(stats.day_of_week.get("Monday"), Some(&281.0));
        assert_eq!(stats.items.len(), 2);
        assert_eq!(stats.recent_sessions.len(), 1);
        assert_eq!(
            stats.recent_sessions[0].display_title.as_deref(),
            Some("Multi File Test Book")
        );
    }

    #[test]
    fn an_answer_with_no_field_gives_no_fault() {
        let stats: ListeningStats =
            serde_json::from_value(serde_json::json!({})).expect("an answer must read");

        assert_eq!(stats.total_time, 0.0);
        assert!(stats.days.is_empty());
        assert!(top_items(&stats, 5).is_empty());
        assert!(last_days(&stats, 14).is_empty());
        assert_eq!(week(&stats).len(), 7);
    }

    #[test]
    fn the_media_of_the_largest_time_comes_first() {
        let top = top_items(&the_answer_of_the_server(), 5);

        assert_eq!(top.len(), 2);
        assert_eq!(top[0].title, "A Long Test Book");
        assert_eq!(top[0].author, "Long Author");
        assert_eq!(top[0].seconds, 276.0);
        assert_eq!(top[1].title, "Multi File Test Book");
    }

    /// A map has no sequence. Two media of the same time must not change
    /// their place from one frame to the next one.
    #[test]
    fn two_media_of_the_same_time_keep_one_sequence() {
        let stats: ListeningStats = serde_json::from_value(serde_json::json!({
            "items": {
                "b": { "timeListening": 60, "mediaMetadata": { "title": "Beta" } },
                "a": { "timeListening": 60, "mediaMetadata": { "title": "Alpha" } },
                "c": { "timeListening": 60, "mediaMetadata": { "title": "Gamma" } }
            }
        }))
        .expect("an answer must read");

        for _ in 0..20 {
            let top = top_items(&stats, 3);
            assert_eq!(top[0].title, "Alpha");
            assert_eq!(top[1].title, "Beta");
            assert_eq!(top[2].title, "Gamma");
        }
    }

    #[test]
    fn a_media_with_no_title_gives_a_line() {
        let stats: ListeningStats = serde_json::from_value(serde_json::json!({
            "items": { "x": { "timeListening": 12 } }
        }))
        .expect("an answer must read");

        let top = top_items(&stats, 5);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].title, "A media with no title");
        assert_eq!(top[0].author, "");
    }

    #[test]
    fn the_list_of_the_media_stops_at_the_count() {
        let stats: ListeningStats = serde_json::from_value(serde_json::json!({
            "items": {
                "a": { "timeListening": 5, "mediaMetadata": { "title": "A" } },
                "b": { "timeListening": 4, "mediaMetadata": { "title": "B" } },
                "c": { "timeListening": 3, "mediaMetadata": { "title": "C" } }
            }
        }))
        .expect("an answer must read");

        assert_eq!(top_items(&stats, 2).len(), 2);
        assert_eq!(top_items(&stats, 0).len(), 0);
    }

    #[test]
    fn the_days_come_in_the_sequence_of_the_time() {
        let stats: ListeningStats = serde_json::from_value(serde_json::json!({
            "days": {
                "2026-08-09": 20,
                "2026-07-31": 10,
                "2026-08-10": 30
            }
        }))
        .expect("an answer must read");

        let days = last_days(&stats, 14);
        assert_eq!(
            days,
            vec![
                ("2026-07-31".to_string(), 10.0),
                ("2026-08-09".to_string(), 20.0),
                ("2026-08-10".to_string(), 30.0),
            ]
        );
    }

    #[test]
    fn the_last_days_keep_the_newest_days() {
        let stats: ListeningStats = serde_json::from_value(serde_json::json!({
            "days": {
                "2026-08-08": 1,
                "2026-08-09": 2,
                "2026-08-10": 3
            }
        }))
        .expect("an answer must read");

        let days = last_days(&stats, 2);
        assert_eq!(days.len(), 2);
        assert_eq!(days[0].0, "2026-08-09");
        assert_eq!(days[1].0, "2026-08-10");
    }

    #[test]
    fn the_week_holds_the_seven_days_always() {
        let week = week(&the_answer_of_the_server());

        assert_eq!(week.len(), 7);
        assert_eq!(week[0], ("Monday", 281.0));
        assert_eq!(week[6], ("Sunday", 0.0));
    }

    #[test]
    fn a_time_of_more_than_one_hour_shows_the_hours() {
        assert_eq!(human_time(3600.0), "1 h 00 min");
        assert_eq!(human_time(7845.0), "2 h 10 min");
    }

    #[test]
    fn a_short_time_shows_the_minutes_and_the_seconds() {
        assert_eq!(human_time(281.0), "4 min 41 s");
        assert_eq!(human_time(45.0), "45 s");
    }

    #[test]
    fn a_time_that_is_not_valid_gives_zero() {
        assert_eq!(human_time(0.0), "0 s");
        assert_eq!(human_time(-10.0), "0 s");
        assert_eq!(human_time(f64::NAN), "0 s");
        assert_eq!(human_time(f64::INFINITY), "0 s");
    }

    #[test]
    fn the_largest_value_of_a_group() {
        assert_eq!(largest(&[1.0, 9.0, 4.0]), 9.0);
        assert_eq!(largest(&[]), 0.0);
        assert_eq!(largest(&[-5.0]), 0.0);
        assert_eq!(largest(&[f64::NAN, 3.0]), 3.0);
    }

    #[test]
    fn a_full_bar_holds_the_width() {
        assert_eq!(bar(10.0, 10.0, 5).chars().count(), 5);
        assert!(bar(10.0, 10.0, 5).chars().all(|one| one == '█'));
    }

    #[test]
    fn a_half_bar_holds_a_half_of_the_width() {
        assert_eq!(bar(5.0, 10.0, 8), "████");
    }

    /// A bar of nothing would say that the user played nothing on that day.
    #[test]
    fn a_very_small_value_still_gives_a_mark() {
        let mark = bar(1.0, 100000.0, 10);
        assert_eq!(mark.chars().count(), 1);
        assert_eq!(mark, "▏");
    }

    #[test]
    fn a_value_of_zero_gives_no_bar() {
        assert_eq!(bar(0.0, 10.0, 8), "");
        assert_eq!(bar(5.0, 0.0, 8), "");
        assert_eq!(bar(5.0, 10.0, 0), "");
        assert_eq!(bar(f64::NAN, 10.0, 8), "");
    }

    /// A value above the largest value must not draw outside the area.
    #[test]
    fn a_bar_never_becomes_wider_than_the_width() {
        for width in 0..40usize {
            for value in [0.5, 1.0, 3.0, 1000.0] {
                assert!(bar(value, 1.0, width).chars().count() <= width.max(1));
                assert!(bar(value, 1.0, width).chars().count() <= width || width == 0);
            }
        }
    }
}

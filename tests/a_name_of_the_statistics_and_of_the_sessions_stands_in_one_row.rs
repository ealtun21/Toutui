//! A name of the server stands in one row of the view of the statistics and
//! of the view of the sessions. See T-377.
//!
//! **A title of the server can hold an end of a line**, and the two views of
//! a text of a scroll held two roads of that end, with a fault of its own on
//! each of them. The view of the statistics builds most of its lines with
//! `Line::from(String)`, and ratatui removes every `\n` of such a text with
//! no space in its place: the words of the two lines then glue together. The
//! view of the sessions puts the name into a `Span`, which keeps the `\n`,
//! and the wrap of a line then breaks the row at it: the second row stands at
//! the indent of a wrap, and it reads as a session of its own, with no time
//! at all.
//!
//! The measurement of the real program v0.8.207 inside tmux, against the
//! sandbox (podman on :13399): `PATCH /api/items/:id/media` gave the book
//! `A Long Test Book` the title `Alpha\nOMEGAEND`, a playback wrote a
//! listening session of that title, and the key `T` then gave
//! `3. AlphaOMEGAEND — Long Author  (1 h 28 min)` — the two words glued —
//! while the key `W` gave `1 min 16 s  Alpha` and, on a row of its own,
//! `OMEGAEND — Long Author  [92% of the media]`.
//!
//! The correction collapses every name of the server at the composition of
//! the line, because the road of the statistics loses its `\n` inside
//! `Line::from(String)` before any later collapse could see it.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a
//! fault that nextest hides (T-144 and T-157).
//!
//! **The build of the fault fails it**: with the collapse removed from the
//! two views, the row of the statistics glues the two words together, and no
//! row of the sessions holds the whole name.

use ratatui::text::Line;
use toutui::api::me::sessions::{PlaySession, SessionPage};
use toutui::logic::sessions_view::Loaded;
use toutui::logic::stats::Statistics;

/// The text of one row of a view.
fn the_text_of(line: &Line<'static>) -> String {
    line.spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect()
}

#[test]
fn a_name_of_the_statistics_and_of_the_sessions_stands_in_one_row() {
    // The view of the statistics: the media that the user played most, the
    // last sessions, and the name of the library each hold a name of the
    // server.
    let listening = serde_json::from_value(serde_json::json!({
        "totalTime": 281,
        "items": {
            "9a671047": {
                "timeListening": 276,
                "mediaMetadata": {
                    "title": "Alpha\nOMEGAEND",
                    "authors": [ { "name": "Long Author" } ]
                }
            }
        },
        "days": { "2026-08-10": 281 },
        "dayOfWeek": { "Monday": 281 },
        "today": 281,
        "recentSessions": [ {
            "displayTitle": "Beta\nGAMMAEND",
            "displayAuthor": "Long Author",
            "date": "2026-08-10",
            "timeListening": 5
        } ]
    }))
    .expect("the answer of the server reads");

    let state = toutui::logic::stats::State::Ready(Box::new(Statistics {
        listening,
        library: None,
        library_name: "Delta\nEPSILONEND".to_string(),
        year: None,
        year_number: 2026,
    }));

    let rows = toutui::ui::stats_tui::lines(&state, 200);
    let rows_of_the_statistics: Vec<String> = rows.iter().map(the_text_of).collect();

    for name in [
        "Alpha OMEGAEND — Long Author",
        "Beta GAMMAEND",
        "The library Delta EPSILONEND",
    ] {
        assert!(
            rows_of_the_statistics.iter().any(|row| row.contains(name)),
            "no row of the statistics holds {name:?}: {rows_of_the_statistics:?}"
        );
    }

    // The view of the sessions: the name of the row and the heading of the
    // day each hold a name of the server.
    let page = SessionPage {
        total: 1,
        num_pages: 1,
        page: 0,
        items_per_page: 25,
        sessions: vec![PlaySession {
            id: Some("one".to_string()),
            display_title: Some("Alpha\nOMEGAEND".to_string()),
            display_author: Some("Long Author".to_string()),
            date: Some("2026-08-17".to_string()),
            day_of_week: Some("Mon\nday".to_string()),
            time_listening: 76.0,
            current_time: 90.0,
            duration: 1800.0,
            media_player: None,
        }],
    };

    let state = toutui::logic::sessions_view::State::Ready(Box::new(Loaded::first(page)));
    let rows = toutui::ui::sessions_tui::lines(&state, 200);
    let rows_of_the_sessions: Vec<String> = rows.iter().map(the_text_of).collect();

    for name in ["Alpha OMEGAEND — Long Author", "2026-08-17 — Mon day"] {
        assert!(
            rows_of_the_sessions.iter().any(|row| row.contains(name)),
            "no row of the sessions holds {name:?}: {rows_of_the_sessions:?}"
        );
    }
}

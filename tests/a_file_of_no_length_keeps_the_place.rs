//! A book whose audio file holds no length keeps the place of the user. See
//! T-180.
//!
//! **A length of 0 of `media.audioFiles[].duration` is a length that the server
//! did not give.** `track_from` reads that field with `unwrap_or(0.0)`, and the
//! shape of T-179 asks the next question: what does the program **do** with
//! that default?
//!
//! A measurement of 2026-08-14 against the sandbox with
//! `docs/harness/a_field_of_the_answer_goes_away.py`, which took the field
//! `duration` out of the answer of
//! `GET /api/items/6ba57b9a-acb5-44f9-b2b6-39ad9107b420` alone:
//!
//! | The moment | The answer |
//! |---|---|
//! | The place of the account, of `PATCH /api/me/progress/:id` | `currentTime 12000` |
//! | The Home view | `A Book Of Many Hours`, `Progress: 65%, 4h40m left` |
//! | The log of the key `l` | `[play] the item … starts at 12000 seconds with 1 tracks` |
//! | The log of the engine | `[worker] the playback starts at 12000 seconds` |
//! | The row of the player, after five seconds | `▶ 4:55 / 0:0 \| Elapsed: 4:55 \| Left: 0:0 (0%)` |
//! | The row of the player, after fifteen seconds | `▶ 26:30 / 0:0` |
//! | The words of the program | none |
//!
//! **The book started at the first second of eight hours**, and it said
//! nothing: `locate` found no file whose end stands after 12000 seconds,
//! therefore it gave the last file of the book at the offset 0, and the engine
//! made no seek. The loop of the playback then wrote no position at all —
//! `position_is_at_the_start(0, 12000)` is false for ever — therefore the user
//! also read no percent and no chapter of their place.
//!
//! Two rules hold this road now. The session of the playback holds the length
//! of the media, and a book of one file takes it. A book of many files with no
//! length keeps the position in the first file of no length, and the row of the
//! player says `N/A` for a length that the program does not have.
//!
//! **This test needs no server.** The body below is the answer of the proxy of
//! the measurement, with the fields that this road reads.

use serde_json::json;
use toutui::logic::playback::{the_tracks_of_the_playback, PlaybackTarget};

/// The media of the measurement.
const THE_MEDIA: &str = "6ba57b9a-acb5-44f9-b2b6-39ad9107b420";

/// The place of the account, in seconds.
const THE_PLACE: f64 = 12000.0;

/// The length of the media, of the answer of `POST /api/items/:id/play`. The
/// program gives this value to `the_tracks_of_the_playback` as a text.
const THE_LENGTH: &str = "28800";

/// The answer of `GET /api/items/:id` of the measurement: one audio file of a
/// book of eight hours, and no field `duration` at any depth.
fn the_item_of_a_server_that_gives_no_length() -> serde_json::Value {
    json!({
        "id": THE_MEDIA,
        "media": {
            "chapters": [
                {"id": 0, "start": 0.0, "end": 10000.0, "title": "The hours of the start"},
                {"id": 1, "start": 10000.0, "end": 20000.0, "title": "The hours of the middle"},
                {"id": 2, "start": 20000.0, "end": 28800.0, "title": "The hours of the end"}
            ],
            "audioFiles": [
                {
                    "index": 1,
                    "ino": "9103848",
                    "mimeType": "audio/mpeg",
                    "metadata": {
                        "filename": "01 - The Whole Book.mp3",
                        "size": 115200330u64
                    }
                }
            ]
        }
    })
}

/// The same book of three audio files. No file holds its length.
fn an_item_of_three_files_with_no_length() -> serde_json::Value {
    json!({
        "id": THE_MEDIA,
        "media": {
            "chapters": [],
            "audioFiles": [
                {"index": 1, "ino": "1", "mimeType": "audio/mpeg",
                 "metadata": {"filename": "01.mp3", "size": 100u64}},
                {"index": 2, "ino": "2", "mimeType": "audio/mpeg",
                 "metadata": {"filename": "02.mp3", "size": 100u64}},
                {"index": 3, "ino": "3", "mimeType": "audio/mpeg",
                 "metadata": {"filename": "03.mp3", "size": 100u64}}
            ]
        }
    })
}

/// The target of the Home view. The Home view gives no length of the book, and
/// the measurement came of that view.
fn the_target_of_the_home_view() -> PlaybackTarget {
    PlaybackTarget::Book {
        item_id: THE_MEDIA.to_string(),
        whole_book_duration: None,
    }
}

/// **The parts of this test stay in one function.** Two test functions of one
/// module fight for the slot of that module. See T-144 and T-157.
#[test]
fn a_book_of_one_file_with_no_length_starts_at_the_place_of_the_user() {
    let item = the_item_of_a_server_that_gives_no_length();

    // The length of the session gives the length of the file, therefore the
    // seek of the engine goes to the place of the user.
    let tracks = the_tracks_of_the_playback(&item, &the_target_of_the_home_view(), THE_LENGTH)
        .expect("the item holds one audio file");

    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks.total_duration(), 28800.0);
    assert_eq!(tracks.locate(THE_PLACE).unwrap(), (0, THE_PLACE));

    // The chapter of the place comes with it. The old road gave the first
    // second, therefore the row of the player said "The hours of the start".
    assert_eq!(
        tracks
            .chapter_at(THE_PLACE)
            .map(|chapter| chapter.title.as_str()),
        Some("The hours of the middle")
    );

    // A target of the Library view holds the length of the book, and a session
    // that gives no length then takes it.
    let of_the_library = PlaybackTarget::Book {
        item_id: THE_MEDIA.to_string(),
        whole_book_duration: Some(28800.0),
    };
    let tracks = the_tracks_of_the_playback(&item, &of_the_library, "0")
        .expect("the item holds one audio file");
    assert_eq!(tracks.total_duration(), 28800.0);
    assert_eq!(tracks.locate(THE_PLACE).unwrap(), (0, THE_PLACE));

    // **A book of many files keeps its 0**: the length of the media says
    // nothing about the length of one file of that media. The position then
    // belongs to the first file of no length, and not to the last file of the
    // book: the old road played the end of the book for a place of 12000
    // seconds.
    let many = the_tracks_of_the_playback(
        &an_item_of_three_files_with_no_length(),
        &the_target_of_the_home_view(),
        THE_LENGTH,
    )
    .expect("the item holds three audio files");

    assert_eq!(many.len(), 3);
    assert_eq!(many.total_duration(), 0.0);
    assert_eq!(many.locate(THE_PLACE).unwrap(), (0, THE_PLACE));

    // A book that holds its lengths changes nothing.
    let of_the_sandbox = json!({
        "id": THE_MEDIA,
        "media": {
            "chapters": [],
            "audioFiles": [
                {"index": 1, "ino": "9103848", "duration": 28800.0, "mimeType": "audio/mpeg",
                 "metadata": {"filename": "01 - The Whole Book.mp3", "size": 115200330u64}}
            ]
        }
    });

    let tracks = the_tracks_of_the_playback(&of_the_sandbox, &the_target_of_the_home_view(), "0")
        .expect("the item holds one audio file");
    assert_eq!(tracks.total_duration(), 28800.0);
    assert_eq!(tracks.locate(THE_PLACE).unwrap(), (0, THE_PLACE));
}

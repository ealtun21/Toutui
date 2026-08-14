//! The fields of `media.audioFiles` that name a file and that put it in its
//! place. See T-181.
//!
//! **The shape of T-179 and of T-180**: a field that the program reads with a
//! default gives no fault of a decode, therefore the program **uses** that
//! default. T-179 took `metadata.size`, T-180 took `duration`, and this test
//! holds the two fields that stay: `ino` and `index`.
//!
//! A measurement of 2026-08-14 against the sandbox with
//! `docs/harness/a_field_of_one_row_goes_away.py`, which takes a field out of
//! **one row** of `media.audioFiles` of
//! `GET /api/items/ac365248-ba42-47ec-a92b-0e5818abc00d` — `Multi File Test
//! Book`, three files of 20 seconds each:
//!
//! | The field that went away | The key `D` | The disk after that key |
//! |---|---|---|
//! | `ino` of the second file | `"Multi File Test Book" is now available offline.` | `001 - 01 - Part 1.mp3` and `003 - 03 - Part 3.mp3`, and the row of the download says 40 seconds of a book of 60 |
//! | `index` of the third file | the same sentence, and the log says `3 file(s)` | `001 - 01 - Part 1.mp3`, **`001 - 03 - Part 3.mp3`**, and `002 - 02 - Part 2.mp3` |
//!
//! **The user lost a part of the book, and the program said nothing.** The old
//! `plan_from_item` dropped a file with no `ino` with the `?` of a
//! `filter_map`, and it gave a file with no `index` the number 1 of
//! `unwrap_or(1)`: two files of one book then held that number, therefore the
//! sort put the last file in the middle of the book, two files took one name on
//! the disk, and the row of `download_files` of the number 1 named `Part 3` and
//! not `Part 1`.
//!
//! **This test needs no server.** The bodies below are the answers of the proxy
//! of the measurement, with the fields that this road reads.

use toutui::logic::download::plan::{
    plan_from_item, the_words_of_a_plan_that_did_not_come, WhyNoPlan,
};
use toutui::logic::playback::tracks_from_item;

/// The media of the measurement.
const THE_MEDIA: &str = "ac365248-ba42-47ec-a92b-0e5818abc00d";

/// One audio file of the answer of the sandbox.
fn a_file(ino: Option<&str>, index: Option<u32>, name: &str) -> serde_json::Value {
    let mut file = serde_json::json!({
        "metadata": { "filename": name, "size": 160613 },
        "duration": 20.0,
        "mimeType": "audio/mpeg",
    });

    if let Some(ino) = ino {
        file["ino"] = serde_json::json!(ino);
    }

    if let Some(index) = index {
        file["index"] = serde_json::json!(index);
    }

    file
}

/// The answer of `GET /api/items/:id` of the measurement. `no_ino` and
/// `no_index` name the file of the book, from 1, that holds no such field.
fn the_book(no_ino: Option<usize>, no_index: Option<usize>) -> serde_json::Value {
    let names = ["01 - Part 1.mp3", "02 - Part 2.mp3", "03 - Part 3.mp3"];
    let inos = ["30853118", "30861586", "30861962"];

    let files: Vec<serde_json::Value> = (1..=3)
        .map(|number| {
            a_file(
                (no_ino != Some(number)).then_some(inos[number - 1]),
                (no_index != Some(number)).then_some(number as u32),
                names[number - 1],
            )
        })
        .collect();

    serde_json::json!({
        "id": THE_MEDIA,
        "media": {
            "metadata": { "title": "Multi File Test Book", "authorName": "Test Author" },
            "audioFiles": files,
        }
    })
}

/// The answer of the sandbox itself gives the plan of the three files.
#[test]
fn the_book_of_the_sandbox_gives_three_files() {
    let plan = plan_from_item(&the_book(None, None)).unwrap();

    assert_eq!(plan.files.len(), 3);
    assert_eq!(plan.total_duration(), 60.0);
    assert_eq!(
        plan.files
            .iter()
            .map(|file| file.disk_name())
            .collect::<Vec<_>>(),
        vec![
            "001 - 01 - Part 1.mp3".to_string(),
            "002 - 02 - Part 2.mp3".to_string(),
            "003 - 03 - Part 3.mp3".to_string(),
        ]
    );
}

/// **The measurement of the field `ino`.** The program has no address of a file
/// that the server did not name, therefore it makes no plan of that book and it
/// says which file it cannot ask for.
#[test]
fn a_file_that_the_server_did_not_name_stops_the_download() {
    let why = plan_from_item(&the_book(Some(2), None)).unwrap_err();

    assert_eq!(
        why,
        WhyNoPlan::AFileWithNoIdentity("02 - Part 2.mp3".to_string())
    );

    let words = the_words_of_a_plan_that_did_not_come(&why);

    assert!(
        words.contains("02 - Part 2.mp3"),
        "the words must name the file: {words}"
    );

    // The words of T-91: the server gave three audio files, therefore the
    // program must not say that it gave none.
    assert!(
        !words.contains("no audio file"),
        "the words say a reason that the program does not have: {words}"
    );
}

/// **The measurement of the field `index`.** The sequence of the answer is the
/// sequence of the book, and every file keeps a number of its own.
#[test]
fn a_file_that_the_server_did_not_number_keeps_the_sequence_of_the_book() {
    let plan = plan_from_item(&the_book(None, Some(3))).unwrap();

    assert_eq!(plan.files.len(), 3);

    assert_eq!(
        plan.files
            .iter()
            .map(|file| file.filename.as_str())
            .collect::<Vec<_>>(),
        vec!["01 - Part 1.mp3", "02 - Part 2.mp3", "03 - Part 3.mp3"]
    );

    assert_eq!(
        plan.files.iter().map(|file| file.index).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );

    // Two files of one name on the disk take one place, and the row of
    // `download_files` of one number takes one file: the second write of that
    // number replaces the first one.
    let names: Vec<String> = plan.files.iter().map(|file| file.disk_name()).collect();

    assert_eq!(
        names.len(),
        names.iter().collect::<std::collections::HashSet<_>>().len(),
        "two files hold one name of the disk: {names:?}"
    );
}

/// The playback reads the same field, and `select_sources` finds the file of
/// the disk of a track by that number.
#[test]
fn the_tracks_of_a_book_of_no_number_keep_the_sequence_of_the_book() {
    let tracks = tracks_from_item(&the_book(None, Some(3))).unwrap();

    assert_eq!(tracks.len(), 3);

    let numbers: Vec<u32> = (0..tracks.len())
        .map(|number| tracks.get(number).unwrap().index)
        .collect();

    assert_eq!(numbers, vec![1, 2, 3]);

    let names: Vec<String> = (0..tracks.len())
        .map(|number| tracks.get(number).unwrap().filename.clone())
        .collect();

    assert_eq!(
        names,
        vec!["01 - Part 1.mp3", "02 - Part 2.mp3", "03 - Part 3.mp3"]
    );
}

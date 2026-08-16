//! The gate of the facts of the design of the panel 5 of the Home view. See
//! T-326.
//!
//! **T-325 gave the facts of the design to the Library view alone, and the
//! Home view draws that same panel 5 of that same book.** The answer of the
//! personalized view holds the same six facts as the answer of the items of a
//! library, and the Home view read none of them. The measurement of the
//! sandbox of 2026-08-16, of `GET /api/libraries/<Books>/personalized` for
//! `A Long Test Book` of the shelf `continue-listening`:
//!
//! ```json
//! "narratorName": "A Test Narrator",
//! "genres": [ "Fiction", "Adventure" ],
//! "numAudioFiles": 1,
//! "size": 7337326,
//! "ebookFormat": "epub"
//! ```
//!
//! **The struct of the media of that answer held no field of the ebook at
//! all**, therefore the sixth fact could not reach the program even after the
//! plumbing of the other five.
//!
//! The measurement of the real program v0.8.155 inside tmux, of the Home view
//! of the library `Books` of the sandbox at 160 columns and 45 rows, with the
//! cursor on `A Long Test Book`. The panel 5 held the picture over 21 rows,
//! two facts over two rows, and **15 rows of no character at all**:
//!
//! ```text
//! │      ▄                                         │
//! │Author: Long Author - Year: N/A - Duration: 30m │
//! │Progress: 50%, 15m left, Not finished           │
//! │                                                │
//! │No description available                        │
//! ```
//!
//! The corrected program of the same harness, of that same row:
//!
//! ```text
//! │Author    Long Author                           │
//! │Narrator  A Test Narrator                       │
//! │Time      30m, 15m left                         │
//! │Genre     Fiction, Adventure                    │
//! │Files     1 file, 7.0 MB                        │
//! │Ebook     epub                                  │
//! │Progress  50%, Not finished                     │
//! │████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░│
//! ```
//!
//! **The controls of that same run**: the row `A Big Book Of A Scan` said
//! `Ebook     pdf` and it took no line of a series, of a narrator, and of a
//! genre, because the server gave none of the three; the row `The Test
//! Chronicles Volume 2` said `Series    The Test Chronicles #2` and it took no
//! line of an ebook; a row of the shelf `Recent Series` kept the line of today,
//! `Depthless Hunger, Book - 1 book - Duration: 10m`; and the Home view of the
//! library `Podcasts` kept the two lines of today,
//! `[Arthur Gordon Pym] - Author: LibriVox - Episode: 1 - Duration: 22m` and
//! `Progress: 22%, 17m left, Not finished`.

use toutui::api::libraries::get_library_perso_view::Root;
use toutui::api::utils::collect_personalized_view::{
    collect_ids_cnt_list, collect_the_facts_cnt_list, collect_titles_cnt_list,
};
use toutui::logic::the_facts_of_a_media::{the_lines_of_the_facts, TheMediaOfThePanel};

/// The answer of the sandbox for the shelves of the library `Books`, of the
/// measurement of 2026-08-16, cut to the fields that this gate reads.
///
/// The shelf `recent-series` stands between the two shelves of media, and its
/// entity holds no media at all: it is the reason of the rule of
/// `media_entities`, and it keeps this gate honest about the sequence.
fn the_shelves_of_the_sandbox() -> Vec<Root> {
    serde_json::from_value(serde_json::json!([
        {
            "id": "continue-listening",
            "label": "Continue Listening",
            "entities": [
                {
                    "id": "9a671047-6146-4003-8510-d215db074a9c",
                    "media": {
                        "duration": 1800.0,
                        "numAudioFiles": 1,
                        "size": 7337326i64,
                        "ebookFormat": "epub",
                        "metadata": {
                            "title": "A Long Test Book",
                            "authorName": "Long Author",
                            "narratorName": "A Test Narrator",
                            "seriesName": "",
                            "genres": [ "Fiction", "Adventure" ]
                        }
                    }
                },
                {
                    "id": "89be0784-ce09-431a-bf2e-72f81f99e39a",
                    "media": {
                        "duration": 3.0,
                        "numAudioFiles": 1,
                        "size": 24648i64,
                        "metadata": {
                            "title": "The Test Chronicles Volume 2",
                            "authorName": "Series Author",
                            "narratorName": "",
                            "seriesName": "The Test Chronicles #2",
                            "genres": []
                        }
                    }
                }
            ]
        },
        {
            "id": "recent-series",
            "label": "Recent Series",
            "entities": [
                { "id": "8a5dce78-c823-441e-a998-eba9f9e8d06b",
                  "name": "The Test Chronicles",
                  "books": [ { "id": "5a66f3c0-7c4e-4dda-881e-622a6f505f9a" } ] }
            ]
        },
        {
            "id": "recently-added",
            "label": "Recently Added",
            "entities": [
                {
                    "id": "27c55369-b048-4d68-9e70-17653b4d618f",
                    "media": {
                        "duration": 3.0,
                        "numAudioFiles": 1,
                        "size": 47433454i64,
                        "ebookFormat": "pdf",
                        "metadata": {
                            "title": "A Big Book Of A Scan",
                            "authorName": "Big Author",
                            "narratorName": "",
                            "seriesName": "",
                            "genres": []
                        }
                    }
                }
            ]
        }
    ]))
    .expect("the answer of the server must read")
}

/// **The six facts of the answer reach the program.** Each of them was away
/// from every list of the Home view, and the field of the ebook was away from
/// the struct of the media itself.
#[tokio::test]
async fn the_shelves_give_the_six_facts_of_each_media() {
    let facts = collect_the_facts_cnt_list(&the_shelves_of_the_sandbox()).await;

    assert_eq!(facts.len(), 3, "one row for each media of the shelves");

    assert_eq!(facts[0].narrator, "A Test Narrator");
    assert_eq!(facts[0].genre, "Fiction, Adventure");
    assert_eq!(facts[0].files, 1);
    assert_eq!(facts[0].size, 7337326);
    // **The struct of the media of this answer held no field of the ebook**,
    // therefore this value was away from the program and not merely away from
    // the panel.
    assert_eq!(facts[0].the_ebook, "epub");
    // A book of no series gives no text, and the panel then takes no line.
    assert_eq!(facts[0].series, "");

    assert_eq!(facts[1].series, "The Test Chronicles #2");
    assert_eq!(facts[1].the_ebook, "");

    assert_eq!(facts[2].the_ebook, "pdf");
}

/// **The list of the facts walks the media in the sequence of the other lists
/// of the Home view.** A shelf of series holds no media, therefore it gives no
/// row: a list that pushed one row for each entity would put the narrator of
/// one book beside the title of another.
#[tokio::test]
async fn the_list_of_the_facts_keeps_the_sequence_of_the_other_lists() {
    let shelves = the_shelves_of_the_sandbox();

    let facts = collect_the_facts_cnt_list(&shelves).await;
    let titles = collect_titles_cnt_list(&shelves).await;
    let ids = collect_ids_cnt_list(&shelves).await;

    assert_eq!(facts.len(), titles.len());
    assert_eq!(facts.len(), ids.len());

    assert_eq!(
        titles,
        vec![
            "A Long Test Book",
            "The Test Chronicles Volume 2",
            "A Big Book Of A Scan"
        ]
    );
    // The third media stands after the shelf of the series, and its facts stand
    // in the third row of the facts.
    assert_eq!(facts[2].size, 47433454);
}

/// **The panel of the Home view says the same lines as the panel of the Library
/// view**, for the same book and the same place of the user. The screen of the
/// measurement of T-326 stands in the head of this file.
#[tokio::test]
async fn the_panel_of_a_media_of_the_home_view_says_every_fact() {
    let facts = collect_the_facts_cnt_list(&the_shelves_of_the_sandbox()).await;

    let lines = the_lines_of_the_facts(
        &TheMediaOfThePanel {
            facts: &facts[0],
            author: "Long Author",
            year: "N/A",
            length: "30m",
            of_the_disk: "",
            percent: "50",
            the_time_that_is_left: "15m left,",
            the_end: "Not finished",
        },
        48,
    );

    assert_eq!(
        lines,
        vec![
            "Author    Long Author",
            "Narrator  A Test Narrator",
            "Time      30m, 15m left",
            "Genre     Fiction, Adventure",
            "Files     1 file, 7.0 MB",
            "Ebook     epub",
            "Progress  50%, Not finished",
            "████████████████████████░░░░░░░░░░░░░░░░░░░░░░░░",
        ],
        "the panel of the Home view must say the eight lines of the design"
    );
}

/// **A fact that the server did not give takes no line at all.** The row
/// `A Big Book Of A Scan` of the measurement held no line of a series, of a
/// narrator, and of a genre, because the server gave none of the three.
#[tokio::test]
async fn a_fact_that_the_server_did_not_give_takes_no_line() {
    let facts = collect_the_facts_cnt_list(&the_shelves_of_the_sandbox()).await;

    let lines = the_lines_of_the_facts(
        &TheMediaOfThePanel {
            facts: &facts[2],
            author: "Big Author",
            year: "N/A",
            length: "0m",
            of_the_disk: "",
            percent: "90",
            the_time_that_is_left: "0m left,",
            the_end: "Not finished",
        },
        48,
    );

    for word in ["Series", "Narrator", "Genre", "N/A"] {
        assert!(
            !lines.iter().any(|line| line.starts_with(word)),
            "no line of the panel may start with {:?}: {:#?}",
            word,
            lines
        );
    }

    assert!(lines.iter().any(|line| line == "Ebook     pdf"));
}

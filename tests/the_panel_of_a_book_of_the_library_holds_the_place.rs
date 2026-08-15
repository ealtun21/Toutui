//! The panel of a line of a book of the Library view and of the view of the
//! search holds the place of the user. See T-241.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a
//! fault that nextest hides (T-144 and T-157).
//!
//! The measurement of 2026-08-15 against the sandbox: the server held
//! `A Book Of Many Hours` at 10800 seconds of 28800 with the percent 38, and
//! one program of one account said two things of that one book. The Home view:
//!
//! ```text
//! ➤ 38% A Book Of Many Hours
//! Author: Many Hours Author - Year: N/A - Duration: 8h
//! Progress: 38%, 5h left, Not finished
//! ```
//!
//! The Library view, and the view of the search of that same run:
//!
//! ```text
//! ➤     A Book Of Many Hours
//! Author: Many Hours Author - Year: N/A
//! ```
//!
//! **The two panels named the author and the year, and no place at all**, while
//! the answer of the account that the start reads for the permissions holds the
//! place of that book already (T-127).

use toutui::api::me::get_media_progress::Root;
use toutui::logic::the_positions::the_places_of_the_account;

/// A row of the answer of `GET /api/me`, in the shape that the program reads.
fn a_row(item: &str, episode: serde_json::Value, percent: f64, place: f64, finished: bool) -> Root {
    serde_json::from_value(serde_json::json!({
        "id": "a-row",
        "userId": "a-user",
        "libraryItemId": item,
        "episodeId": episode,
        "mediaItemId": "a-media",
        "mediaItemType": "book",
        "duration": 28800.0,
        "progress": percent,
        "currentTime": place,
        "isFinished": finished,
        "hideFromContinueListening": false,
        "ebookLocation": null,
        "ebookProgress": 0,
        "lastUpdate": 1,
        "startedAt": 1,
        "finishedAt": null
    }))
    .expect("the row of the answer of the account")
}

/// Gives the block of a function of a file of the source. See the trap 209.
///
/// A window of a number of characters is a window of the comments of the
/// function after it: the words of a correction take a line out of that window,
/// and the gate then says that the program lost a rule that it holds. The block
/// ends at the comment or at the head of the function that comes after this one.
fn the_block_of(source: &str, head: &str) -> String {
    let start = source
        .find(head)
        .unwrap_or_else(|| panic!("the source holds no function `{}`", head));
    let body = &source[start + head.len()..];

    let end = body
        .find("\n    /// ")
        .into_iter()
        .chain(body.find("\n    pub fn "))
        .chain(body.find("\n    fn "))
        .min()
        .unwrap_or(body.len());

    body[..end].to_string()
}

#[tokio::test]
async fn the_panel_of_a_book_of_the_library_holds_the_place() {
    // ---------------------------------------------------------------------
    // The box of the places of the account.
    // ---------------------------------------------------------------------

    // **One answer of `GET /api/me` gives the place of every media of the
    // account** (T-127), therefore a library of 2056 items costs no request at
    // all. The key names the episode after the item (T-223).
    let places = the_places_of_the_account(&[
        a_row("a-book", serde_json::Value::Null, 0.375, 10800.0, false),
        a_row(
            "a-podcast",
            serde_json::json!("an-episode"),
            1.0,
            1320.0,
            true,
        ),
    ])
    .await;

    let row = places
        .get("a-book")
        .expect("the box holds the place of the book");
    assert_eq!(row[0], "38", "the panel says the percent of the user");
    assert_eq!(row[1], "Not finished", "the panel says the mark of the end");
    assert_eq!(
        row[2], "10800",
        "the panel makes the time that is left of the place in seconds (T-234)"
    );

    // **A key of the item alone gives the place of one episode to every episode
    // of a podcast** (T-188 and T-228), therefore the key of an episode names
    // that episode after the item.
    assert!(
        places.get("a-podcast").is_none(),
        "the key of an episode is not the key of the item"
    );
    let row = places
        .get("a-podcast/an-episode")
        .expect("the box holds the place of the episode");
    assert_eq!(row[0], "100");
    assert_eq!(row[1], "Finished");

    // **A media that stands in no row of the answer played never** (T-127): the
    // box holds no place of it, and the panel of that line then says the words
    // of a media that never played.
    assert!(places.get("a-book-that-played-never").is_none());

    // ---------------------------------------------------------------------
    // The two panels of the screen.
    // ---------------------------------------------------------------------

    let source = std::fs::read_to_string("src/ui/tui.rs").expect("the source of the screen");

    for (head, of_the_view) in [
        (
            "fn render_info_library(",
            "the_place_of_the_panel_of_the_library",
        ),
        (
            "fn render_info_search_book(",
            "the_place_of_the_panel_of_the_search_book",
        ),
    ] {
        let block = the_block_of(&source, head);

        assert!(
            block.contains(of_the_view),
            "the panel of `{}` reads no place of the user",
            head
        );
        assert!(
            block.contains("Progress: {}%, {} {}"),
            "the panel of `{}` says no percent, no time that is left, and no \
             mark of the end",
            head
        );
        assert!(
            block.contains("place.percent")
                && block.contains("place.the_time_that_is_left")
                && block.contains("place.the_end"),
            "the panel of `{}` holds no value of the place",
            head
        );
    }

    // ---------------------------------------------------------------------
    // The three roads to the place of a book of those two views.
    // ---------------------------------------------------------------------

    // **The engine of this program comes first, the row of a live message after
    // it, and the row of the box last** (T-239 and T-240): the panel of the two
    // views takes the function of the panel of the Home view, therefore the
    // sequence of the three roads is the sequence of that function.
    let source = std::fs::read_to_string("src/app.rs").expect("the source of the application");
    let block = the_block_of(&source, "fn the_place_of_the_panel_of_this_book(");

    assert!(
        block.contains("crate::logic::live::progress_of"),
        "the panel of a book reads no live message of the server (T-240)"
    );
    assert!(
        block.contains("self.the_place_of_the_playback()"),
        "the panel of a book reads no place of the engine of this program (T-239)"
    );
    assert!(
        block.contains("crate::logic::the_positions::the_place_of"),
        "the panel of a book reads no place of the answer of the account"
    );
    assert!(
        block.contains("the_place_of_the_panel("),
        "the panel of a book takes no function of the panel of a line"
    );
}

//! The line and the panel of the view of the books of a series, and of the
//! view of the media of a collection or of a playlist, hold the place of the
//! user. See T-243.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a
//! fault that nextest hides (T-144 and T-157).
//!
//! The measurement of 2026-08-15 against the sandbox. The server held
//! `The Test Chronicles Volume 2` at the percent 41 and `A Long Test Book` at
//! the percent 63, and the Home view of that run said both of them:
//!
//! ```text
//! ➤ 41% The Test Chronicles Volume 2
//!   63% A Long Test Book
//! ```
//!
//! The view of the books of that same series, and the view of the media of
//! `A Test Collection`, of that same run said nothing of either place:
//!
//! ```text
//! ➤ #2 - The Test Chronicles Volume 2
//! Author: Series Author - Duration: 0m
//!
//! ➤ A Long Test Book
//! Book - Author: Long Author - Duration: 30m
//! ```
//!
//! **A media of a collection or of a playlist is one book or one episode of a
//! podcast** (T-223): the two episodes `Chapter 00` and `Chapter 01` of one
//! podcast of `A Podcast Playlist` stood at 22 percent and at 81 percent, and a
//! key of the item alone gives the two lines one place.

use toutui::api::utils::collect_lists::{ListEntry, ListKind, ListView};
use toutui::api::utils::collect_series::{SeriesBookView, SeriesView};

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

#[test]
fn the_views_of_a_series_and_of_a_list_hold_the_place() {
    // ---------------------------------------------------------------------
    // The line of the two views.
    // ---------------------------------------------------------------------

    let source = std::fs::read_to_string("src/app.rs").expect("the source of the application");

    for head in ["fn series_book_lines(", "fn list_entry_lines("] {
        let block = the_block_of(&source, head);

        assert!(
            block.contains("self.the_mark_of_this_media("),
            "the line of `{}` reads no place of the user",
            head
        );
        assert!(
            block.contains("crate::ui::marks::line("),
            "the line of `{}` holds no mark at all",
            head
        );
    }

    // **The key of a media of a list names the episode after the item**
    // (T-223): a line of an episode of a podcast holds the place of its own
    // episode, and a key of the item alone gives every episode of one podcast
    // the same place.
    assert!(
        the_block_of(&source, "fn list_entry_lines(").contains("entry.episode_id.as_deref()"),
        "the line of a media of a list takes the place of the podcast and not \
         of the episode"
    );

    // A book of a series holds no episode.
    assert!(
        the_block_of(&source, "fn series_book_lines(")
            .contains("the_key_of_a_line(&book.id, None)"),
        "the line of a book of a series reads no key of that book"
    );

    // ---------------------------------------------------------------------
    // The panel of the two views.
    // ---------------------------------------------------------------------

    for head in [
        "fn the_place_of_the_panel_of_a_series_book(",
        "fn the_place_of_the_panel_of_a_list_entry(",
    ] {
        let block = the_block_of(&source, head);

        assert!(
            block.contains("self.the_place_of_the_panel_of_this_media("),
            "the panel of `{}` reads no place of the user",
            head
        );
    }

    assert!(
        the_block_of(&source, "fn the_place_of_the_panel_of_a_list_entry(")
            .contains("entry.episode_id.as_deref()"),
        "the panel of a media of a list takes the place of the podcast and not \
         of the episode"
    );

    // ---------------------------------------------------------------------
    // The three roads to the place of a media of those two views.
    // ---------------------------------------------------------------------

    // **The engine of this program comes first, the row of a live message after
    // it, and the row of the box of the places last** (T-239, T-240, and
    // T-241). The line and the panel each take that sequence, therefore one
    // media says one place in the two rows of the screen.
    let of_the_mark = the_block_of(&source, "fn the_mark_of_this_media(");
    let of_the_panel = the_block_of(&source, "fn the_place_of_the_panel_of_this_media(");

    for (name, block) in [("the mark", &of_the_mark), ("the panel", &of_the_panel)] {
        assert!(
            block.contains("crate::logic::live::progress_of"),
            "{} of a media reads no live message of the server (T-240)",
            name
        );
        assert!(
            block.contains("crate::logic::the_positions::the_place_of"),
            "{} of a media reads no place of the answer of the account (T-241)",
            name
        );
        assert!(
            block.contains("plays_now"),
            "{} of a media says nothing of the media that plays (T-239)",
            name
        );
    }

    assert!(
        of_the_panel.contains("self.the_place_of_the_playback()"),
        "the panel of a media reads no place of the engine of this program"
    );
    assert!(
        of_the_mark.contains("crate::ui::marks::of_progress"),
        "the mark of a media gives no percent at all"
    );

    // ---------------------------------------------------------------------
    // The key of a line that names one media.
    // ---------------------------------------------------------------------

    // **A media with no identity has no place** (T-192): the answer of the
    // server gives such a media no id, and the line of it keeps the mark and
    // the panel of a media that played never.
    let block = the_block_of(&source, "fn the_key_of_a_line(");

    assert!(
        block.contains("id.trim().is_empty()") && block.contains("return None"),
        "a media with no identity takes a key of the place"
    );

    // ---------------------------------------------------------------------
    // The two views of the screen.
    // ---------------------------------------------------------------------

    let source = std::fs::read_to_string("src/ui/tui.rs").expect("the source of the screen");

    for (head, of_the_lines, of_the_panel) in [
        (
            "fn render_series_book(",
            "self.series_book_lines()",
            "self.the_place_of_the_panel_of_a_series_book(book)",
        ),
        (
            "fn render_list_entries(",
            "self.list_entry_lines()",
            "self.the_place_of_the_panel_of_a_list_entry(entry)",
        ),
    ] {
        let block = the_block_of(&source, head);

        assert!(
            block.contains(of_the_lines),
            "the line of `{}` holds the title of the server and no mark",
            head
        );
        assert!(
            block.contains(of_the_panel),
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
    // The line of a media of a list names one media, and the line of a series
    // names more than one.
    // ---------------------------------------------------------------------

    let book = SeriesBookView {
        id: "a-book".into(),
        title: "A Book".into(),
        author: "An Author".into(),
        sequence: "2".into(),
        duration: 1800.0,
        description: String::new(),
    };

    assert_eq!(book.line(), "#2 - A Book");

    let series = SeriesView {
        id: "a-series".into(),
        name: "A Series".into(),
        description: String::new(),
        books: vec![book],
    };

    // **A line of a series holds more than one book**, therefore the view of
    // the series keeps its line with no mark of a place (T-44 and T-22).
    assert_eq!(series.line(), "A Series [1 book]");

    let episode = ListEntry {
        id: "a-podcast".into(),
        episode_id: Some("an-episode".into()),
        title: "Chapter 01".into(),
        author: "An Author".into(),
        duration: 1320.0,
        description: String::new(),
    };

    assert!(episode.is_episode());
    assert_eq!(episode.line(), "Chapter 01");

    let list = ListView {
        id: "a-list".into(),
        kind: ListKind::Playlist,
        name: "A Podcast Playlist".into(),
        description: String::new(),
        entries: vec![episode],
    };

    // **A line of a collection or of a playlist holds more than one media**,
    // therefore the view of the lists keeps its line with no mark of a place.
    assert_eq!(list.line(), "[Playlist] A Podcast Playlist [1 item]");
}

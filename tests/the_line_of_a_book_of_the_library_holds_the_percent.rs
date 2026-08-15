//! The line of a book of the Library view and of the view of the search holds
//! the percent of the user. See T-242.
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a
//! fault that nextest hides (T-144 and T-157).
//!
//! The measurement of 2026-08-15 against the sandbox: the server held
//! `A Book Of Many Hours` at 10800 seconds of 28800 with the percent 84, and
//! one program of one account said the place of that book in the line of one
//! view and in no line of two others. The Home view:
//!
//! ```text
//!   84% A Book Of Many Hours
//! ```
//!
//! The Library view, and the view of the search, of that same run:
//!
//! ```text
//! ➤     A Book Of Many Hours
//! Author: Many Hours Author - Year: N/A - Duration: 8h
//! Progress: 84%, 5h left, Not finished
//! ```
//!
//! **The panel of those two lines said the percent already** (T-241), and the
//! line above it said nothing: a list of 18 books of the Library view held no
//! number at all, and the mark of a book that the user finished stood on no
//! line of it.

use toutui::ui::marks;

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
fn the_line_of_a_book_of_the_library_holds_the_percent() {
    // ---------------------------------------------------------------------
    // The mark of the percent.
    // ---------------------------------------------------------------------

    // `marks::of_library` gives the mark of the media that plays alone, and a
    // book of the library took it: the line of that book said no percent, no
    // mark of the end, and nothing of the user at all.
    assert_eq!(
        marks::of_library(false).trim(),
        "",
        "the mark of a line that holds more than one media says no place"
    );

    // The mark of a book of a list takes the place of the user, in the words of
    // `collect_progress_percentage_book` and of `collect_is_finished_book`.
    assert!(
        marks::of_progress("84", "Not finished", false).contains("84%"),
        "the mark of a book that the user began says the percent"
    );
    assert!(
        marks::of_progress("100", "Finished", false).contains('✓'),
        "the mark of a book that the user finished says the end"
    );

    // **The engine of this program comes first** (T-239): the media that plays
    // says the mark of the playback and no number, because the row of the
    // player of that same frame says the place of the engine.
    assert!(
        marks::of_progress("84", "Not finished", true).contains('▶'),
        "the mark of the media that plays stands above the percent of the row"
    );

    // **A book that the box of the places does not name played never** (T-127
    // and T-241), and the mark of that line is then no mark at all.
    assert_eq!(marks::of_progress("", "", false).trim(), "");

    // ---------------------------------------------------------------------
    // The two lines of the application.
    // ---------------------------------------------------------------------

    let source = std::fs::read_to_string("src/app.rs").expect("the source of the application");

    for head in ["fn library_lines(", "fn search_book_lines("] {
        let block = the_block_of(&source, head);

        assert!(
            block.contains("self.the_mark_of_a_book("),
            "the line of `{}` reads no place of the user",
            head
        );
    }

    // A line of a series holds more than one book, therefore it keeps the mark
    // of `of_library`. See T-44 and T-22.
    assert!(
        the_block_of(&source, "fn library_lines(").contains("crate::ui::marks::of_library(false)"),
        "the line of a series of the Library view took a mark of a place"
    );

    // ---------------------------------------------------------------------
    // The three roads to the place of a book of those two views.
    // ---------------------------------------------------------------------

    // **The engine of this program comes first, the row of a live message after
    // it, and the row of the box of the places last** (T-239, T-240, and
    // T-241): the mark of a line takes the sequence of the panel of that same
    // line, therefore one book says one place in the two rows of the screen.
    // **`the_mark_of_a_book` gives the key of a book to `the_mark_of_this_media`**
    // (T-243): a line of a media of a collection or of a playlist holds an
    // episode of a podcast too, therefore the three roads stand in the function
    // that takes the key.
    let block = the_block_of(&source, "fn the_mark_of_this_media(");

    assert!(
        block.contains("crate::logic::live::progress_of"),
        "the mark of a book reads no live message of the server (T-240)"
    );
    assert!(
        block.contains("crate::logic::the_positions::the_place_of"),
        "the mark of a book reads no place of the answer of the account (T-241)"
    );
    assert!(
        block.contains("plays_now"),
        "the mark of a book says nothing of the media that plays (T-239)"
    );
    assert!(
        block.contains("crate::ui::marks::of_progress"),
        "the mark of a book gives no percent at all"
    );

    // **The key of a place names the episode after the item** (T-223), and a
    // line of a library of podcasts holds more than one media (T-221): the box
    // holds no row of a podcast, therefore that line keeps the mark of the
    // media that plays.
    assert!(
        the_block_of(&source, "fn the_mark_of_a_book(").contains("the_key_of_the_media(id, None)"),
        "the mark of a book takes the place of one episode of a podcast"
    );

    // ---------------------------------------------------------------------
    // The line of the view of the search.
    // ---------------------------------------------------------------------

    let source = std::fs::read_to_string("src/ui/tui.rs").expect("the source of the screen");
    let block = the_block_of(&source, "fn render_search_book(");

    assert!(
        block.contains("self.search_book_lines(titles_search_book_or_pod)"),
        "the view of the search renders the titles of the server and no mark"
    );
    assert!(
        block.contains("if self.is_podcast {"),
        "the view of the search gives a mark of a place to a line of a podcast"
    );
}

//! The key `h` gives the user a way out of every view. See T-143.
//!
//! **The footer of a view says "h: back"**, and the view of the cache of the
//! ebooks promised it while the key did nothing: a measurement of 2026-08-13
//! pressed `h` three times in that view and the screen did not move. The next
//! key of the measurement was `Esc`, and that key stops the program (the trap
//! 69). **A key that does nothing in one view is a fault of its own** (T-79).
//!
//! The handler of the key holds one arm for each view, therefore a new view of a
//! later session can forget that arm again. This test reads the source of the
//! program and it names every view of `AppView`, as the test of T-135 does.

/// The key `h` goes back in every view that holds a footer of a list.
///
/// The footer of `footer_with` says "h: back", and the view of the cache of the
/// ebooks promised it while the key did nothing: `Esc` then stopped the program.
/// **A key that does nothing in one view is a fault of its own** (T-79). See
/// T-143.
#[test]
fn the_key_h_names_every_view_that_it_must_leave() {
    let source = include_str!("../src/app.rs");

    let start = source
        .find("KeyCode::Char('h') => {")
        .expect("the handler of the key h");
    let block = &source[start..start + 4000];

    // The key `h` in these three views is not a key that goes back: the Home
    // view and the Library view hold a list of media, and the reader of an ebook
    // turns a page with it.
    let of_a_list = ["Home", "Library", "Reader"];

    for view in every_view_of_the_program(source) {
        if of_a_list.contains(&view.as_str()) {
            continue;
        }

        assert!(
            block.contains(&format!("AppView::{}", view)),
            "the key h must give the user a way out of the view {}: the footer \
             of that view says \"h: back\"",
            view
        );
    }
}

/// Gives the name of every view of `AppView`, out of the source of the program.
fn every_view_of_the_program(source: &str) -> Vec<String> {
    let start = source
        .find("pub enum AppView {")
        .expect("the enum of the views");
    let end = start
        + source[start..]
            .find("\n}\n")
            .expect("the end of the enum of the views");

    source[start..end]
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with("///") && line.ends_with(','))
        .map(|line| line.trim_end_matches(',').to_string())
        .filter(|name| {
            !name.is_empty()
                && name
                    .chars()
                    .next()
                    .is_some_and(|first| first.is_uppercase())
        })
        .collect()
}

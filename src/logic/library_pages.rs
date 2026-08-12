//! The pages of the library, between the task and the screen. See T-70.
//!
//! `get_all_books` read **every** page of the library at the start. A library
//! of 2056 items therefore made five requests before the first frame, and a
//! library of 250000 items made 500 of them: the cost of the start grew with
//! the library of the user.
//!
//! The program reads the first page now, and it asks the server for the next
//! page when the user comes near the end of the lines that it holds. This is
//! the shape of `logic::sessions_view`, and the render takes the page at the
//! next frame.
//!
//! **The search of the server stays the authority for a title that the program
//! did not load.** The key `/` shows the titles of the program at once, and it
//! adds the answer of the server when that answer comes.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

/// The number of the lines before the end where the program asks for the next
/// page.
///
/// The page holds 500 items, and a screen holds 40 lines at the most. A user
/// who moves fast therefore never waits at the end of the list.
pub const LINES_BEFORE_THE_END: usize = 50;

/// One page of the library, in the shape that the lists of the screen need.
///
/// The task makes these lists, because `collect_titles_library` and the six
/// functions beside it are asynchronous and the render is not.
#[derive(Debug, Clone, Default)]
pub struct Page {
    /// The number of the page. The first page is 0.
    pub number: usize,
    /// The number of items of the library, over every page.
    pub total: usize,
    pub titles: Vec<String>,
    pub ids: Vec<String>,
    pub authors: Vec<String>,
    pub authors_of_a_podcast: Vec<String>,
    pub durations: Vec<f64>,
    pub descriptions: Vec<String>,
    pub years: Vec<String>,
}

/// Tells if the program must ask the server for the next page now.
///
/// The program asks when the server holds more items, when no task asks
/// already, and when the user is near the end of the items that the program
/// holds.
///
/// The function is pure, therefore a test needs no server.
pub fn wants_the_next_page(loaded: usize, total: usize, selected: usize, asking: bool) -> bool {
    if asking || loaded >= total || loaded == 0 {
        return false;
    }

    selected + LINES_BEFORE_THE_END >= loaded
}

/// Gives the place of the library that comes after the library of now.
///
/// The key Shift+Tab takes it. The list goes round: the last library gives the
/// first one. A server of one library gives `None`, and the program then says
/// that it holds one library. See T-66.
///
/// The function is pure, therefore a test needs no server.
pub fn the_next_library(ids: &[String], now: &str) -> Option<usize> {
    if ids.len() < 2 {
        return None;
    }

    let place = ids.iter().position(|id| id == now)?;

    Some((place + 1) % ids.len())
}

/// Gives the place of the library that the program must take, when the library
/// of its database is not a library of the account. See T-136.
///
/// **An account may lose a library while the program of that account holds it.**
/// The server then answers `403` for every request of that library, and the
/// program showed a view of no line with the words "This library holds no
/// media": the name of the library and the kind of it went away from the header,
/// the key Shift+Tab said "This server holds one library" and it moved to
/// nothing, and **no key of the program gave the user the library that they may
/// read**. A start after it gave the same screen.
///
/// The answer gives `None` when the library of the program is a library of the
/// account, and it gives `None` for a list of no library: the offline mode holds
/// such a list, and a program that writes a library there would forget the
/// library of the user.
///
/// The function is pure, therefore a test needs no server.
pub fn the_library_that_the_program_must_take(ids: &[String], now: &str) -> Option<usize> {
    if ids.is_empty() || ids.iter().any(|id| id == now) {
        return None;
    }

    Some(0)
}

fn the_page_that_waits() -> &'static Mutex<Option<Page>> {
    static PAGE: OnceLock<Mutex<Option<Page>>> = OnceLock::new();
    PAGE.get_or_init(|| Mutex::new(None))
}

fn the_flag() -> &'static AtomicBool {
    static ASKING: OnceLock<AtomicBool> = OnceLock::new();
    ASKING.get_or_init(|| AtomicBool::new(false))
}

/// Tells if a task asks the server for a page now.
pub fn asks() -> bool {
    the_flag().load(Ordering::SeqCst)
}

/// Writes that a task asks the server, or that it does not.
pub fn keep_the_flag(asking: bool) {
    the_flag().store(asking, Ordering::SeqCst);
}

/// Puts a page in the box that the render reads. The task calls this.
pub fn keep(page: Page) {
    if let Ok(mut place) = the_page_that_waits().lock() {
        *place = Some(page);
    }
}

/// Takes the page that waits, and it leaves the box empty.
pub fn take() -> Option<Page> {
    match the_page_that_waits().lock() {
        Ok(mut place) => place.take(),
        Err(_) => None,
    }
}

/// Empties the box and the flag.
///
/// A new library, a new filter, and the key `R` all make the pages of the
/// library before them wrong. A page of the library before must not go into
/// the lists of the library now.
pub fn forget() {
    keep_the_flag(false);

    if let Ok(mut place) = the_page_that_waits().lock() {
        *place = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The program asks for the next page when the user comes near the end,
    /// and at no other moment. See T-70.
    #[test]
    fn the_program_asks_for_a_page_near_the_end_of_the_lines() {
        // 500 items of 2056, and the user stands at the line 1: far from the
        // end.
        assert!(!wants_the_next_page(500, 2056, 1, false));

        // The user comes to the line 450, and 50 lines are left.
        assert!(wants_the_next_page(500, 2056, 450, false));

        // A task asks already. A second request of the same page gives the
        // same answer, and it costs the server.
        assert!(!wants_the_next_page(500, 2056, 450, true));

        // The program holds every item of the library.
        assert!(!wants_the_next_page(2056, 2056, 2055, false));

        // A library that gave nothing asks for nothing: the program does not
        // know that a page exists.
        assert!(!wants_the_next_page(0, 0, 0, false));
        assert!(!wants_the_next_page(0, 500, 0, false));
    }

    /// The key Shift+Tab goes round the libraries of the server. See T-66.
    #[test]
    fn the_next_library_goes_round() {
        let ids: Vec<String> = ["a", "b", "c"].iter().map(|id| id.to_string()).collect();

        assert_eq!(the_next_library(&ids, "a"), Some(1));
        assert_eq!(the_next_library(&ids, "b"), Some(2));

        // The last library gives the first one.
        assert_eq!(the_next_library(&ids, "c"), Some(0));

        // A server of one library has no next library.
        assert_eq!(the_next_library(&ids[..1], "a"), None);
        assert_eq!(the_next_library(&[], "a"), None);

        // A library that the list does not hold gives nothing: the program
        // must not take a library that the user cannot see.
        assert_eq!(the_next_library(&ids, "d"), None);
    }

    /// **A library that the account may not read gives the first library that
    /// it may read.** A measurement of 2026-08-13 took the library of an account
    /// away while the program of that account held it: every view held no line,
    /// the header said "📖  ()", and no key gave the user the library that they
    /// may read. See T-136.
    #[test]
    fn a_library_that_the_account_may_not_read_gives_the_first_one() {
        let ids: Vec<String> = ["a", "b", "c"].iter().map(|id| id.to_string()).collect();

        // The library of the program is a library of the account. Nothing
        // changes.
        assert_eq!(the_library_that_the_program_must_take(&ids, "a"), None);
        assert_eq!(the_library_that_the_program_must_take(&ids, "c"), None);

        // The account lost that library. The program takes the first library of
        // the account.
        assert_eq!(the_library_that_the_program_must_take(&ids, "d"), Some(0));

        // **The offline mode holds no library**, and the program must keep the
        // library of the user for the start that answers.
        assert_eq!(the_library_that_the_program_must_take(&[], "a"), None);

        // A database of no library takes the first library of the account.
        assert_eq!(the_library_that_the_program_must_take(&ids, ""), Some(0));
    }

    /// The box holds one page, and the render takes it one time.
    #[test]
    fn the_box_gives_the_page_one_time() {
        forget();
        assert!(take().is_none());

        keep(Page {
            number: 1,
            total: 12,
            titles: vec!["A book".to_string()],
            ..Default::default()
        });

        let page = take().expect("the box must hold the page");
        assert_eq!(page.number, 1);
        assert_eq!(page.titles, vec!["A book".to_string()]);
        assert!(take().is_none());

        keep_the_flag(true);
        assert!(asks());
        forget();
        assert!(!asks());
    }
}

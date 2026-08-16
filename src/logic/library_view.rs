//! The lines of the Library view. See T-22.
//!
//! A library of a series holds one item for each book. The view showed each
//! book on its own line, therefore a series of twelve books filled the screen.
//! This module puts every book of a series on one line. The user then opens
//! the series with the key `l`, and the books come in the sequence of the
//! series.
//!
//! The function here is pure, therefore a test needs no server and no screen.

use crate::api::utils::collect_series::SeriesView;
use std::collections::HashMap;

/// One line of the Library view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryRow {
    /// One book, or one podcast. The number is the position of the item in
    /// `ids_library` and in every other list of the library.
    Book { item: usize },
    /// Every book of one series, in one line.
    Series {
        /// The position of the series in `App::series`.
        series: usize,
        /// The position of the first book of the series in `ids_library`. The
        /// view uses it when it needs a value of the library, for example the
        /// name of the author.
        first_item: usize,
    },
}

impl LibraryRow {
    /// Gives the position in the lists of the library.
    pub fn item(&self) -> usize {
        match self {
            LibraryRow::Book { item } => *item,
            LibraryRow::Series { first_item, .. } => *first_item,
        }
    }

    /// Gives the position of the series, if the line is a series.
    pub fn series(&self) -> Option<usize> {
        match self {
            LibraryRow::Book { .. } => None,
            LibraryRow::Series { series, .. } => Some(*series),
        }
    }
}

/// Makes the lines of the Library view.
///
/// The sequence of the library does not change. A series takes the place of
/// its first book, and the other books of that series give no line.
///
/// A podcast library and the offline mode give no series. Every line is then a
/// book, and the position of a line is the position of the item.
///
/// **`the_books_stand_apart` says that every book takes a line of its own**
/// (T-318). The filter of one series is the one road of the user to the books
/// of a series inside this list: the server then answers with those books and
/// with no other media, and a line of the group would repeat the name of the
/// filter and hide the answer. See
/// `crate::logic::sort_filter::is_a_filter_of_one_series`.
pub fn group_library(
    ids: &[String],
    series: &[SeriesView],
    the_books_stand_apart: bool,
) -> Vec<LibraryRow> {
    if the_books_stand_apart || series.is_empty() {
        return (0..ids.len())
            .map(|item| LibraryRow::Book { item })
            .collect();
    }

    // The identity of a book gives the series of that book. A book that stands
    // in more than one series belongs to the first series of the list.
    let mut series_of_book: HashMap<&str, usize> = HashMap::new();
    for (index, one) in series.iter().enumerate() {
        for book in &one.books {
            series_of_book.entry(book.id.as_str()).or_insert(index);
        }
    }

    let mut rows = Vec::with_capacity(ids.len());
    let mut done: Vec<bool> = vec![false; series.len()];

    for (item, id) in ids.iter().enumerate() {
        match series_of_book.get(id.as_str()) {
            Some(&index) => {
                if done[index] {
                    continue;
                }
                done[index] = true;
                rows.push(LibraryRow::Series {
                    series: index,
                    first_item: item,
                });
            }
            None => rows.push(LibraryRow::Book { item }),
        }
    }

    rows
}

/// Says if every book of a series takes a line of its own. See T-318 and
/// T-324.
///
/// **The two roads to a book of a series are one rule**: the filter of one
/// series asks the server for the books of that series alone (T-318), and the
/// mode of the whole library asks for every item of the library with no group
/// at all (T-324). A call site that reads one of them and not the other gives a
/// list that hides a book.
pub fn the_books_stand_apart(filter: &str) -> bool {
    the_whole_library::stands() || crate::logic::sort_filter::is_a_filter_of_one_series(filter)
}

/// The part of the request of the items that groups the books of a series.
/// See T-22 and T-324.
///
/// The text starts with `&`, because the caller writes `limit` and `page`
/// before it.
///
/// **A library of podcasts holds no series**, therefore that request takes no
/// parameter at all, and **the mode of the whole library takes the parameter
/// away**: the answer of the server then holds one item for each book.
pub fn the_group_of_the_request(is_podcast: bool) -> &'static str {
    if is_podcast || the_whole_library::stands() {
        ""
    } else {
        "&collapseseries=1"
    }
}

/// The mode of the whole library. See T-324.
///
/// **A book of a series stands in no row of the Library view**, because the
/// request of the items holds `collapseseries=1` and the answer of the server
/// then gives one item for the whole series. The measurement of 2026-08-16, of
/// the library `Books` of the sandbox: the server holds 22 books,
/// `collapseseries=0` gives 22 rows, `collapseseries=1` gives 18, and the four
/// books that go away are `The Test Chronicles Volume 2`,
/// `The Test Chronicles Volume 3`, `Second Series Volume 2`, and `Second Series
/// Volume 3`. The list of the program said `4 Library [18 items]`, and the user
/// had **no key at all** that gives every book of every series in one list.
///
/// **The parameter of the server alone changes no screen** (the trap of T-318):
/// [`group_library`] collapses the answer again on the side of the program,
/// therefore the mode writes the request **and** the rows.
///
/// **The mode lives in the process and not in the row of the account**, in the
/// same way as `crate::logic::library_pages`. The key of the mode makes the
/// application again (`must_refresh`), and
/// `App::keep_the_state_of_the_application_before` runs **after**
/// `App::new_with_the_engine` writes the request of the items: a field of `App`
/// therefore reaches no query at all. **The mode is not the mode of the start**,
/// and a log out and a change of the account each start a new process
/// (`App::start_the_program_with_this_account`), therefore the mode of the start
/// comes back with that process and this module needs no `forget`.
pub mod the_whole_library {
    use std::sync::atomic::{AtomicBool, Ordering};

    static STANDS: AtomicBool = AtomicBool::new(false);

    /// Says if every book of every series takes a line of its own.
    pub fn stands() -> bool {
        STANDS.load(Ordering::Relaxed)
    }

    /// Writes the mode.
    pub fn keep(yes: bool) {
        STANDS.store(yes, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::utils::collect_series::SeriesBookView;

    fn book(id: &str, sequence: &str) -> SeriesBookView {
        SeriesBookView {
            id: id.to_string(),
            title: format!("Book {}", id),
            author: "An Author".to_string(),
            sequence: sequence.to_string(),
            duration: 0.0,
            description: String::new(),
        }
    }

    fn series(name: &str, ids: &[&str]) -> SeriesView {
        SeriesView {
            id: format!("series-{}", name),
            name: name.to_string(),
            description: String::new(),
            books: ids
                .iter()
                .enumerate()
                .map(|(index, id)| book(id, &(index + 1).to_string()))
                .collect(),
        }
    }

    fn ids(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| v.to_string()).collect()
    }

    #[test]
    fn a_library_with_no_series_gives_one_line_for_each_book() {
        let rows = group_library(&ids(&["a", "b", "c"]), &[], false);
        assert_eq!(
            rows,
            vec![
                LibraryRow::Book { item: 0 },
                LibraryRow::Book { item: 1 },
                LibraryRow::Book { item: 2 },
            ]
        );
    }

    #[test]
    fn a_series_of_three_books_gives_one_line() {
        // The library of the sandbox: two series of three books, and one book
        // that stands alone.
        let all = ids(&["t3", "t2", "t1", "s3", "s2", "s1", "multi"]);
        let list = vec![
            series("The Test Chronicles", &["t1", "t2", "t3"]),
            series("Second Series", &["s1", "s2", "s3"]),
        ];

        let rows = group_library(&all, &list, false);

        assert_eq!(
            rows,
            vec![
                LibraryRow::Series {
                    series: 0,
                    first_item: 0
                },
                LibraryRow::Series {
                    series: 1,
                    first_item: 3
                },
                LibraryRow::Book { item: 6 },
            ]
        );
    }

    #[test]
    fn a_series_stands_at_the_place_of_its_first_book() {
        let all = ids(&["alone", "t1", "t2"]);
        let list = vec![series("A Series", &["t1", "t2"])];

        let rows = group_library(&all, &list, false);

        assert_eq!(
            rows,
            vec![
                LibraryRow::Book { item: 0 },
                LibraryRow::Series {
                    series: 0,
                    first_item: 1
                },
            ]
        );
    }

    #[test]
    fn a_book_of_a_series_that_the_library_does_not_hold_gives_no_line() {
        // The series holds three books, and the library holds one of them.
        let all = ids(&["t2"]);
        let list = vec![series("A Series", &["t1", "t2", "t3"])];

        let rows = group_library(&all, &list, false);
        assert_eq!(
            rows,
            vec![LibraryRow::Series {
                series: 0,
                first_item: 0
            }]
        );
    }

    #[test]
    fn a_book_of_two_series_gives_one_line_only() {
        let all = ids(&["a"]);
        let list = vec![series("First", &["a"]), series("Second", &["a"])];

        let rows = group_library(&all, &list, false);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].series(), Some(0));
    }

    /// **A filter of one series gives the books of that series** (T-318). The
    /// measurement of the real program v0.8.149 against the sandbox: the row
    /// `The Test Chronicles` of the view of the key `f` gave the header
    /// `4 Library [1 item] — a filter is on (f)` over one line,
    /// `The Test Chronicles [3 books]`, and the server answered with three
    /// books.
    #[test]
    fn a_filter_of_one_series_gives_a_line_for_each_book_of_it() {
        let all = ids(&["t1", "t2", "t3"]);
        let list = vec![series("The Test Chronicles", &["t1", "t2", "t3"])];

        assert_eq!(
            group_library(&all, &list, true),
            vec![
                LibraryRow::Book { item: 0 },
                LibraryRow::Book { item: 1 },
                LibraryRow::Book { item: 2 },
            ]
        );

        // No line of such a list opens a view of the books of a series: the
        // list holds those books already.
        assert!(group_library(&all, &list, true)
            .iter()
            .all(|row| row.series().is_none()));
    }

    #[test]
    fn an_empty_library_gives_no_line() {
        assert!(group_library(&[], &[], false).is_empty());
        assert!(group_library(&[], &[series("A Series", &["a"])], false).is_empty());
    }

    #[test]
    fn a_line_gives_the_position_in_the_lists_of_the_library() {
        let rows = group_library(&ids(&["a", "b"]), &[series("A Series", &["b"])], false);
        assert_eq!(rows[0].item(), 0);
        assert_eq!(rows[1].item(), 1);
        assert_eq!(rows[1].series(), Some(0));
        assert_eq!(rows[0].series(), None);
    }
}

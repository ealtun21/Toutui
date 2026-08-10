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
pub fn group_library(ids: &[String], series: &[SeriesView]) -> Vec<LibraryRow> {
    if series.is_empty() {
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
        let rows = group_library(&ids(&["a", "b", "c"]), &[]);
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

        let rows = group_library(&all, &list);

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

        let rows = group_library(&all, &list);

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

        let rows = group_library(&all, &list);
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

        let rows = group_library(&all, &list);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].series(), Some(0));
    }

    #[test]
    fn an_empty_library_gives_no_line() {
        assert!(group_library(&[], &[]).is_empty());
        assert!(group_library(&[], &[series("A Series", &["a"])]).is_empty());
    }

    #[test]
    fn a_line_gives_the_position_in_the_lists_of_the_library() {
        let rows = group_library(&ids(&["a", "b"]), &[series("A Series", &["b"])]);
        assert_eq!(rows[0].item(), 0);
        assert_eq!(rows[1].item(), 1);
        assert_eq!(rows[1].series(), Some(0));
        assert_eq!(rows[0].series(), None);
    }
}

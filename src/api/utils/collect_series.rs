//! The display data of the series.
//!
//! This module changes the answer of the server into the lists that the user
//! interface shows. It has no network code, thus the tests need no server.

use crate::api::libraries::get_all_series::SeriesRoot;
use crate::utils::html_text::to_plain_text;

/// One book of a series.
#[derive(Debug, Clone, PartialEq)]
pub struct SeriesBookView {
    pub id: String,
    pub title: String,
    pub author: String,
    /// The number of the book in the series, for example `1` or `1.5`. The
    /// value is empty when the server gives no number.
    pub sequence: String,
    pub duration: f64,
    pub description: String,
}

/// One series of a library.
#[derive(Debug, Clone, PartialEq)]
pub struct SeriesView {
    pub id: String,
    pub name: String,
    pub description: String,
    /// The books, in the sequence of the series.
    pub books: Vec<SeriesBookView>,
}

impl SeriesView {
    /// Gives the line of this series in the list.
    pub fn line(&self) -> String {
        match self.books.len() {
            1 => format!("{} [1 book]", self.name),
            count => format!("{} [{} books]", self.name, count),
        }
    }
}

impl SeriesBookView {
    /// Gives the line of this book in the list of the series.
    ///
    /// The number comes first. The user then reads the sequence of the series.
    pub fn line(&self) -> String {
        if self.sequence.is_empty() {
            self.title.clone()
        } else {
            format!("#{} - {}", self.sequence, self.title)
        }
    }
}

/// Reads the number of the book from the field `seriesName`.
///
/// The server gives a value like `The Test Chronicles #1`. A book with no
/// number gives an empty text.
pub fn sequence_from(series_name: &str) -> String {
    match series_name.rsplit_once('#') {
        Some((_, number)) => number.trim().to_string(),
        None => String::new(),
    }
}

/// Puts the books in the sequence of the series.
///
/// The server gives the correct sequence today. This function keeps that
/// sequence when a number is absent or is not a number. It sorts only when
/// every book has a number, because a text sort gives `#10` before `#2`.
fn sort_by_sequence(books: &mut [SeriesBookView]) {
    let numbers: Option<Vec<f64>> = books
        .iter()
        .map(|book| book.sequence.parse::<f64>().ok())
        .collect();

    let Some(numbers) = numbers else {
        return;
    };

    let mut order: Vec<usize> = (0..books.len()).collect();
    order.sort_by(|a, b| numbers[*a].total_cmp(&numbers[*b]));

    let sorted: Vec<SeriesBookView> = order.iter().map(|index| books[*index].clone()).collect();
    books.clone_from_slice(&sorted);
}

/// Makes the display data from the answer of the server.
pub fn collect_series(root: &SeriesRoot) -> Vec<SeriesView> {
    let Some(results) = &root.results else {
        return Vec::new();
    };

    results
        .iter()
        .map(|series| {
            let mut books: Vec<SeriesBookView> = series
                .books
                .iter()
                .flatten()
                .map(|book| {
                    let metadata = book.media.as_ref().and_then(|media| media.metadata.as_ref());

                    SeriesBookView {
                        id: book.id.clone().unwrap_or_default(),
                        title: metadata
                            .and_then(|data| data.title.clone())
                            .unwrap_or_else(|| "N/A".to_string()),
                        author: metadata
                            .and_then(|data| data.author_name.clone())
                            .unwrap_or_else(|| "N/A".to_string()),
                        sequence: sequence_from(
                            metadata
                                .and_then(|data| data.series_name.as_deref())
                                .unwrap_or_default(),
                        ),
                        duration: book
                            .media
                            .as_ref()
                            .and_then(|media| media.duration)
                            .unwrap_or(0.0),
                        description: metadata
                            .and_then(|data| data.description.as_deref())
                            .map(to_plain_text)
                            .unwrap_or_else(|| "No description available".to_string()),
                    }
                })
                .collect();

            sort_by_sequence(&mut books);

            SeriesView {
                id: series.id.clone().unwrap_or_default(),
                name: series
                    .name
                    .clone()
                    .unwrap_or_else(|| "N/A".to_string()),
                description: series
                    .description
                    .as_deref()
                    .map(to_plain_text)
                    .unwrap_or_else(|| "No description available".to_string()),
                books,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root_of(raw: &str) -> SeriesRoot {
        serde_json::from_str(raw).unwrap()
    }

    fn sample() -> SeriesRoot {
        root_of(
            r#"{
              "results": [
                { "id": "s1", "name": "A Series", "description": "<p>Two &amp; more</p>",
                  "books": [
                    { "id": "b2", "media": { "duration": 20.0, "metadata":
                        { "title": "Second", "authorName": "An Author",
                          "seriesName": "A Series #2" } } },
                    { "id": "b1", "media": { "duration": 10.0, "metadata":
                        { "title": "First", "authorName": "An Author",
                          "seriesName": "A Series #1" } } }
                  ] }
              ],
              "total": 1
            }"#,
        )
    }

    #[test]
    fn the_number_comes_from_the_name_of_the_series() {
        assert_eq!(sequence_from("A Series #3"), "3");
        assert_eq!(sequence_from("A Series #1.5"), "1.5");
        assert_eq!(sequence_from("A Series"), "");
        assert_eq!(sequence_from(""), "");
    }

    /// A name that holds the character `#` gives the last number.
    #[test]
    fn a_name_with_two_signs_gives_the_last_number() {
        assert_eq!(sequence_from("Book #1 of #4"), "4");
    }

    #[test]
    fn the_books_come_in_the_sequence_of_the_series() {
        let series = collect_series(&sample());

        assert_eq!(series.len(), 1);
        let order: Vec<&str> = series[0].books.iter().map(|b| b.title.as_str()).collect();
        assert_eq!(order, vec!["First", "Second"]);
    }

    /// A text sort gives `#10` before `#2`. A number sort does not.
    #[test]
    fn the_number_ten_comes_after_the_number_two() {
        let mut books = vec![
            SeriesBookView {
                id: "a".into(), title: "Ten".into(), author: String::new(),
                sequence: "10".into(), duration: 0.0, description: String::new(),
            },
            SeriesBookView {
                id: "b".into(), title: "Two".into(), author: String::new(),
                sequence: "2".into(), duration: 0.0, description: String::new(),
            },
        ];

        sort_by_sequence(&mut books);

        assert_eq!(books[0].title, "Two");
        assert_eq!(books[1].title, "Ten");
    }

    /// One book with no number keeps the sequence of the server.
    #[test]
    fn a_book_with_no_number_keeps_the_sequence_of_the_server() {
        let mut books = vec![
            SeriesBookView {
                id: "a".into(), title: "Second".into(), author: String::new(),
                sequence: "2".into(), duration: 0.0, description: String::new(),
            },
            SeriesBookView {
                id: "b".into(), title: "No number".into(), author: String::new(),
                sequence: String::new(), duration: 0.0, description: String::new(),
            },
        ];

        sort_by_sequence(&mut books);

        assert_eq!(books[0].title, "Second");
    }

    #[test]
    fn the_description_holds_no_html() {
        let series = collect_series(&sample());
        assert_eq!(series[0].description, "Two & more");
    }

    #[test]
    fn a_book_with_no_description_gives_a_message() {
        let series = collect_series(&sample());
        assert_eq!(series[0].books[0].description, "No description available");
    }

    #[test]
    fn the_line_of_a_series_gives_the_number_of_books() {
        let series = collect_series(&sample());
        assert_eq!(series[0].line(), "A Series [2 books]");
    }

    #[test]
    fn the_line_of_one_book_uses_the_singular() {
        let view = SeriesView {
            id: "s".into(), name: "One".into(), description: String::new(),
            books: vec![SeriesBookView {
                id: "a".into(), title: "Only".into(), author: String::new(),
                sequence: "1".into(), duration: 0.0, description: String::new(),
            }],
        };

        assert_eq!(view.line(), "One [1 book]");
    }

    #[test]
    fn the_line_of_a_book_gives_the_number_first() {
        let series = collect_series(&sample());
        assert_eq!(series[0].books[0].line(), "#1 - First");
        assert_eq!(series[0].books[1].line(), "#2 - Second");
    }

    #[test]
    fn a_book_with_no_number_gives_the_title_only() {
        let view = SeriesBookView {
            id: "a".into(), title: "Alone".into(), author: String::new(),
            sequence: String::new(), duration: 0.0, description: String::new(),
        };

        assert_eq!(view.line(), "Alone");
    }

    #[test]
    fn an_answer_with_no_series_gives_an_empty_list() {
        assert!(collect_series(&SeriesRoot::default()).is_empty());
        assert!(collect_series(&root_of(r#"{"results": [], "total": 0}"#)).is_empty());
    }

    /// A series with no book must not stop the application.
    #[test]
    fn a_series_with_no_book_gives_an_empty_list_of_books() {
        let root = root_of(r#"{"results": [{"id": "s1", "name": "Empty"}], "total": 1}"#);
        let series = collect_series(&root);

        assert_eq!(series[0].books.len(), 0);
        assert_eq!(series[0].line(), "Empty [0 books]");
    }

    /// This answer comes from a real Audiobookshelf 2.36.0 server.
    #[test]
    fn the_module_reads_a_real_answer() {
        let raw = include_str!("../../../tests/fixtures/library_series.json");
        let series = collect_series(&root_of(raw));

        assert_eq!(series.len(), 2);
        assert_eq!(series[0].name, "Second Series");
        assert_eq!(series[0].books.len(), 3);

        let order: Vec<&str> = series[0].books.iter().map(|b| b.sequence.as_str()).collect();
        assert_eq!(order, vec!["1", "2", "3"]);
        assert_eq!(series[0].books[0].title, "Second Series Volume 1");
        assert_eq!(series[0].books[0].author, "Series Author");
    }
}

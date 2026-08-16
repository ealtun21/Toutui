//! The sequence and the filter of a library. See T-24.
//!
//! `GET /api/libraries/:id/items` takes `sort`, `desc`, and `filter`. The
//! program sent none of them, therefore the user read a library of 2056 items
//! in one sequence only, and they could not show the books of one author.
//!
//! Measurements against an Audiobookshelf 2.36.0 on 2026-08-11:
//!
//! - `?sort=media.metadata.title` gives `A Long Test Book, Alice in
//!   Wonderland, Multi File Test Book`, and `&desc=1` gives the other
//!   direction.
//! - `?filter=authors.Y2M1ODkxZDMtZjBhNS00MmIwLWFjMzktNmMzM2RmMTk5ZWZk` gives
//!   one book of four authors. The text after the full stop is the identity of
//!   the author in base64.
//! - **The server takes a name of a field that does not exist.**
//!   `?sort=bogus.field` gives `200` and `sortBy: "bogus.field"`, and the
//!   sequence is then not specified. Therefore the program offers the names
//!   that this file names, and no other name.
//!
//! Every function here is pure, therefore a test needs no server.

/// One choice of the sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SortChoice {
    /// The name for the user.
    pub label: &'static str,
    /// The name of the field for the server.
    pub field: &'static str,
}

/// The choices of the sequence for a library of books.
///
/// Every field of this list is measured against an Audiobookshelf 2.36.0.
pub const SORTS_OF_BOOKS: [SortChoice; 7] = [
    SortChoice {
        label: "The title",
        field: "media.metadata.title",
    },
    SortChoice {
        label: "The title, with no \"A\" and no \"The\"",
        field: "media.metadata.titleIgnorePrefix",
    },
    SortChoice {
        label: "The author",
        field: "media.metadata.authorNameLF",
    },
    SortChoice {
        label: "The time when the book came",
        field: "addedAt",
    },
    SortChoice {
        label: "The year",
        field: "media.metadata.publishedYear",
    },
    SortChoice {
        label: "The length",
        field: "media.duration",
    },
    SortChoice {
        label: "The size on the disk",
        field: "size",
    },
];

/// The choices of the sequence for a library of podcasts.
pub const SORTS_OF_PODCASTS: [SortChoice; 3] = [
    SortChoice {
        label: "The title",
        field: "media.metadata.title",
    },
    SortChoice {
        label: "The time when the podcast came",
        field: "addedAt",
    },
    SortChoice {
        label: "The number of the episodes",
        field: "numEpisodes",
    },
];

/// Gives the choices of the sequence for a library.
pub fn sorts_of(is_podcast: bool) -> &'static [SortChoice] {
    if is_podcast {
        &SORTS_OF_PODCASTS
    } else {
        &SORTS_OF_BOOKS
    }
}

/// Gives the name for the user of a field of the server.
pub fn label_of(field: &str, is_podcast: bool) -> &'static str {
    sorts_of(is_podcast)
        .iter()
        .find(|one| one.field == field)
        .map(|one| one.label)
        .unwrap_or("The sequence of the server")
}

/// Tells if the program knows this field.
///
/// The server takes a field that does not exist and it gives an unspecified
/// sequence. A value of an older version of the program, or of a hand that
/// changed the database, must not reach the server.
pub fn is_a_field_of_the_program(field: &str, is_podcast: bool) -> bool {
    sorts_of(is_podcast).iter().any(|one| one.field == field)
}

/// Writes the part of the address that holds the sequence and the filter.
///
/// The text starts with `&`, because the caller writes `limit` and `page`
/// before it. An empty choice gives an empty text, therefore the request is
/// the request that the program sent before this work.
pub fn query(field: &str, desc: bool, filter: &str) -> String {
    let mut out = String::new();

    if !field.is_empty() {
        out.push_str("&sort=");
        out.push_str(field);

        if desc {
            out.push_str("&desc=1");
        }
    }

    if !filter.is_empty() {
        out.push_str("&filter=");
        out.push_str(filter);
    }

    out
}

/// The letters of base64.
const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Writes a text in base64.
///
/// The filter of the server is `<type>.<value in base64>`. The program has no
/// crate for this work, and the rule of base64 is short. See T-20 for the rule
/// of the dependencies.
pub fn encode_base64(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for group in bytes.chunks(3) {
        let a = group[0] as u32;
        let b = *group.get(1).unwrap_or(&0) as u32;
        let c = *group.get(2).unwrap_or(&0) as u32;
        let three = (a << 16) | (b << 8) | c;

        out.push(ALPHABET[(three >> 18) as usize & 63] as char);
        out.push(ALPHABET[(three >> 12) as usize & 63] as char);

        // A group of one byte gives two letters and two full stops of base64.
        // A group of two bytes gives three letters and one.
        if group.len() > 1 {
            out.push(ALPHABET[(three >> 6) as usize & 63] as char);
        } else {
            out.push('=');
        }

        if group.len() > 2 {
            out.push(ALPHABET[three as usize & 63] as char);
        } else {
            out.push('=');
        }
    }

    out
}

/// One choice of the filter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterChoice {
    /// The name for the user, for example "Lewis Carroll".
    pub label: String,
    /// The group of the name, for example "The authors".
    pub group: &'static str,
    /// The value for the server, for example `authors.Y2M1...`.
    pub value: String,
}

/// Writes the value of a filter for the server.
pub fn filter_value(kind: &str, value: &str) -> String {
    format!("{}.{}", kind, encode_base64(value))
}

/// Tells if the filter of the library asks the server for one series.
///
/// **A filter of one series is the one road of this program to the books of a
/// series inside the list of the library** (T-318). The measurement of the
/// real program v0.8.149 of 2026-08-16 against the sandbox: the user took the
/// row `The Test Chronicles` of the group `The series` of the view of the key
/// `f`, the server answered with the **three** books of that series, and the
/// Library view held one line, `The Test Chronicles [3 books]`, and the header
/// `4 Library [1 item] — a filter is on (f)`.
///
/// `crate::logic::library_view::group_library` puts every book of a series on
/// one line (T-22), and that rule is right for a library of many series. **It
/// is wrong for a list that holds one series and no other media**: the user
/// asked for that series, and the line of the group then repeats the name of
/// the filter and it hides the answer of the server.
///
/// The value of such a filter is `series.<the identity in base64>`, and
/// `filter_value` writes it.
pub fn is_a_filter_of_one_series(filter: &str) -> bool {
    filter.starts_with("series.")
}

/// The three choices of the position that the server knows.
///
/// A measurement on 2026-08-11: `progress.ZmluaXNoZWQ=` gives 2 books,
/// `progress.aW4tcHJvZ3Jlc3M=` gives 4, and `progress.bm90LXN0YXJ0ZWQ=`
/// gives 4.
///
/// **The words hold no empty part** (T-330.2). The three labels of the start
/// each began with fourteen columns that say nothing, because every row of a
/// library is a media. A line of the panel 3 of the stack holds 30 columns
/// (T-324), therefore the third of them reached that panel cut at the word
/// `not…`, and the same words met the address of the server at the second row
/// of the header (T-329): a row of 84 columns held none of them at all.
///
/// **`Started, not finished` is not `Started`**: the filter of the server
/// gives the media that the user started **and did not finish**, therefore
/// `Started` alone says less than the filter does.
pub const PROGRESS: [(&str, &str); 3] = [
    ("Finished", "finished"),
    ("Started, not finished", "in-progress"),
    ("Not started", "not-started"),
];

/// Gives the choices of the position.
pub fn progress_choices() -> Vec<FilterChoice> {
    PROGRESS
        .iter()
        .map(|(label, value)| FilterChoice {
            label: label.to_string(),
            group: "Your position",
            value: filter_value("progress", value),
        })
        .collect()
}

/// One line of the view of the sequence and of the filter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Row {
    /// The name of a group. The user cannot select this line.
    Title(String),
    /// A choice of the sequence.
    Sort { field: String, label: String },
    /// The line that changes the direction.
    Direction,
    /// The line that gives every book of every series a line of its own.
    /// See T-324.
    ///
    /// A library of podcasts holds no series, therefore that library holds no
    /// such line.
    TheWholeLibrary,
    /// The line that removes the filter.
    NoFilter,
    /// A choice of the filter.
    Filter { label: String, value: String },
    /// A sentence for the user. The user cannot select this line.
    Note(String),
}

impl Row {
    /// Tells if the user can select this line.
    pub fn is_a_line_of_the_user(&self) -> bool {
        !matches!(self, Row::Title(_) | Row::Note(_))
    }
}

/// Makes the lines of the view of the sequence and of the filter.
///
/// `filters` comes from `GET /api/libraries/:id/filterdata`. That request
/// goes when the user opens the view, therefore the start of the program does
/// not wait for it. A note tells the user while the answer is not here.
pub fn rows(is_podcast: bool, filters: &[FilterChoice], note: Option<String>) -> Vec<Row> {
    let mut out: Vec<Row> = Vec::new();

    out.push(Row::Title("The sequence".to_string()));

    for one in sorts_of(is_podcast) {
        out.push(Row::Sort {
            field: one.field.to_string(),
            label: one.label.to_string(),
        });
    }

    out.push(Row::Direction);

    // **A library of podcasts holds no series** (T-324), therefore the request
    // of that library takes no `collapseseries` and this line says nothing to
    // its user.
    if !is_podcast {
        out.push(Row::TheWholeLibrary);
    }

    out.push(Row::Title("The filter".to_string()));
    out.push(Row::NoFilter);

    let mut group = "";

    for one in progress_choices().iter().chain(filters.iter()) {
        // A group gives its name one time, before its first value.
        if one.group != group {
            group = one.group;
            out.push(Row::Title(one.group.to_string()));
        }

        out.push(Row::Filter {
            label: one.label.clone(),
            value: one.value.clone(),
        });
    }

    if let Some(note) = note {
        out.push(Row::Note(note));
    }

    out
}

/// Makes the text of one line.
///
/// A mark stands before the choice that the program uses now. The user then
/// reads the state of the library in the same list that changes it.
pub fn line_of(row: &Row, field: &str, desc: bool, filter: &str) -> String {
    let mark = |yes: bool| if yes { "✓ " } else { "  " };

    match row {
        Row::Title(name) => format!("▌ {}", name),
        Row::Note(text) => format!("  {}", text),
        Row::Sort { field: one, label } => format!("{}{}", mark(one == field), label),
        Row::Direction => {
            if desc {
                "  The direction: the largest first".to_string()
            } else {
                "  The direction: the smallest first".to_string()
            }
        }
        Row::TheWholeLibrary => {
            // **The mark says the state, and the words do not** (T-324): the
            // panel 2 of the stack holds 32 columns, and the two sentences of
            // the first form of this row (`The books of a series: one line for
            // the series` and `… one line for each book`) each reached the
            // screen as `The books of a series: one…`. A row that says one text
            // for the two states of the program says nothing at all.
            format!(
                "{}{}",
                mark(crate::logic::library_view::the_whole_library::stands()),
                "Every book of a series"
            )
        }
        Row::NoFilter => format!("{}{}", mark(filter.is_empty()), "No filter"),
        Row::Filter { label, value } => format!("{}{}", mark(value == filter), label),
    }
}

/// The answer of `GET /api/libraries/:id/filterdata`. See T-24.
///
/// The render is not asynchronous. Therefore a task asks the server and it
/// puts the answer here, and the render takes it at the next frame. This is
/// the shape of the statistics and of the search of the server.
pub mod from_the_server {
    use super::FilterChoice;
    use std::sync::{Mutex, OnceLock};

    /// What the view must draw.
    #[derive(Debug, Clone, Default)]
    pub enum State {
        /// The program did not ask the server.
        #[default]
        Nothing,
        /// The program asked the server, and no answer came.
        Waiting,
        /// The server answered.
        Ready(Vec<FilterChoice>),
        /// The server gave no answer, and this text says why.
        Fault(String),
    }

    fn box_of_the_state() -> &'static Mutex<State> {
        static STATE: OnceLock<Mutex<State>> = OnceLock::new();
        STATE.get_or_init(|| Mutex::new(State::Nothing))
    }

    /// Writes the state. The task of the request calls this.
    pub fn keep(state: State) {
        if let Ok(mut place) = box_of_the_state().lock() {
            *place = state;
        }
    }

    /// Gives the state. The render calls this at each frame.
    pub fn state() -> State {
        match box_of_the_state().lock() {
            Ok(place) => place.clone(),
            Err(_) => State::Nothing,
        }
    }

    /// Forgets the answer. A refresh of the program asks the server again.
    pub fn forget() {
        keep(State::Nothing);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_program_knows_the_fields_that_it_offers() {
        assert!(is_a_field_of_the_program("addedAt", false));
        assert!(is_a_field_of_the_program("media.metadata.title", true));
        assert!(!is_a_field_of_the_program("bogus.field", false));
        // A library of podcasts has no author and no year.
        assert!(!is_a_field_of_the_program(
            "media.metadata.authorNameLF",
            true
        ));
    }

    #[test]
    fn a_field_gives_its_name_for_the_user() {
        assert_eq!(label_of("addedAt", false), "The time when the book came");
        assert_eq!(label_of("addedAt", true), "The time when the podcast came");
        assert_eq!(label_of("bogus.field", false), "The sequence of the server");
    }

    /// The value of the filter of a series comes of `filter_value`, and no
    /// other group of the filters starts with those seven characters.
    #[test]
    fn the_program_knows_the_filter_of_one_series() {
        assert!(is_a_filter_of_one_series(&filter_value(
            "series",
            "8a5dce78-c823-441e-a998-eba9f9e8d06b"
        )));
        assert!(!is_a_filter_of_one_series(""));
        assert!(!is_a_filter_of_one_series(&filter_value(
            "progress", "finished"
        )));
        assert!(!is_a_filter_of_one_series(&filter_value(
            "authors",
            "cc5891d3-f0a5-42b0-ac39-6c33df199efd"
        )));
    }

    #[test]
    fn no_choice_gives_the_request_of_before() {
        assert_eq!(query("", false, ""), "");
    }

    #[test]
    fn the_sequence_goes_in_the_address() {
        assert_eq!(
            query("media.metadata.title", false, ""),
            "&sort=media.metadata.title"
        );
        assert_eq!(
            query("media.metadata.title", true, ""),
            "&sort=media.metadata.title&desc=1"
        );
    }

    #[test]
    fn the_filter_goes_in_the_address() {
        assert_eq!(query("", false, "authors.Y2M1"), "&filter=authors.Y2M1");
        assert_eq!(
            query("addedAt", true, "authors.Y2M1"),
            "&sort=addedAt&desc=1&filter=authors.Y2M1"
        );
    }

    /// The measurement of 2026-08-11 gives this value for the author
    /// "Long Author".
    #[test]
    fn the_program_writes_the_base64_of_the_server() {
        assert_eq!(
            encode_base64("cc5891d3-f0a5-42b0-ac39-6c33df199efd"),
            "Y2M1ODkxZDMtZjBhNS00MmIwLWFjMzktNmMzM2RmMTk5ZWZk"
        );
        assert_eq!(encode_base64("finished"), "ZmluaXNoZWQ=");
        assert_eq!(encode_base64("in-progress"), "aW4tcHJvZ3Jlc3M=");
        assert_eq!(encode_base64("not-started"), "bm90LXN0YXJ0ZWQ=");
    }

    /// The three lengths of the last group of base64.
    #[test]
    fn every_length_of_a_text_gives_a_correct_value() {
        assert_eq!(encode_base64(""), "");
        assert_eq!(encode_base64("f"), "Zg==");
        assert_eq!(encode_base64("fo"), "Zm8=");
        assert_eq!(encode_base64("foo"), "Zm9v");
        assert_eq!(encode_base64("foob"), "Zm9vYg==");
        assert_eq!(encode_base64("fooba"), "Zm9vYmE=");
        assert_eq!(encode_base64("foobar"), "Zm9vYmFy");
    }

    #[test]
    fn a_text_of_a_different_writing_gives_a_correct_value() {
        assert_eq!(encode_base64("é"), "w6k=");
        assert_eq!(encode_base64("日本"), "5pel5pys");
    }

    /// The length of base64 is four letters for each three bytes.
    #[test]
    fn the_length_of_the_value_follows_the_rule() {
        for length in 0..40usize {
            let text = "a".repeat(length);
            let value = encode_base64(&text);

            assert_eq!(value.len(), length.div_ceil(3) * 4);
            assert!(value.len().is_multiple_of(4));
        }
    }

    #[test]
    fn the_choices_of_the_position_hold_the_value_of_the_server() {
        let choices = progress_choices();

        assert_eq!(choices.len(), 3);
        assert_eq!(choices[0].value, "progress.ZmluaXNoZWQ=");
        assert_eq!(choices[1].value, "progress.aW4tcHJvZ3Jlc3M=");
        assert_eq!(choices[2].value, "progress.bm90LXN0YXJ0ZWQ=");
    }

    fn the_filters() -> Vec<FilterChoice> {
        vec![
            FilterChoice {
                label: "Lewis Carroll".to_string(),
                group: "The authors",
                value: "authors.MzEy".to_string(),
            },
            FilterChoice {
                label: "Long Author".to_string(),
                group: "The authors",
                value: "authors.Y2M1".to_string(),
            },
            FilterChoice {
                label: "Second Series".to_string(),
                group: "The series",
                value: "series.ZTIz".to_string(),
            },
        ]
    }

    #[test]
    fn the_view_holds_the_sequence_and_the_filter() {
        let rows = rows(false, &the_filters(), None);

        assert_eq!(rows[0], Row::Title("The sequence".to_string()));
        assert_eq!(
            rows.iter()
                .filter(|row| matches!(row, Row::Sort { .. }))
                .count(),
            SORTS_OF_BOOKS.len()
        );
        assert_eq!(rows.iter().filter(|row| **row == Row::Direction).count(), 1);
        assert_eq!(rows.iter().filter(|row| **row == Row::NoFilter).count(), 1);
        // Three choices of the position, and three values of the server.
        assert_eq!(
            rows.iter()
                .filter(|row| matches!(row, Row::Filter { .. }))
                .count(),
            6
        );
    }

    /// A group gives its name one time, and it gives it before its first
    /// value.
    #[test]
    fn a_group_of_the_filter_gives_its_name_one_time() {
        let rows = rows(false, &the_filters(), None);
        let titles: Vec<&String> = rows
            .iter()
            .filter_map(|row| match row {
                Row::Title(name) => Some(name),
                _ => None,
            })
            .collect();

        assert_eq!(
            titles,
            vec![
                "The sequence",
                "The filter",
                "Your position",
                "The authors",
                "The series"
            ]
        );
    }

    #[test]
    fn a_library_of_podcasts_gives_its_own_choices_of_the_sequence() {
        let rows = rows(true, &[], None);

        assert_eq!(
            rows.iter()
                .filter(|row| matches!(row, Row::Sort { .. }))
                .count(),
            SORTS_OF_PODCASTS.len()
        );
    }

    #[test]
    fn a_title_and_a_note_are_not_lines_of_the_user() {
        assert!(!Row::Title("A Title".to_string()).is_a_line_of_the_user());
        assert!(!Row::Note("A note".to_string()).is_a_line_of_the_user());
        assert!(Row::Direction.is_a_line_of_the_user());
        assert!(Row::NoFilter.is_a_line_of_the_user());
    }

    #[test]
    fn a_note_stands_at_the_end_of_the_view() {
        let rows = rows(false, &[], Some("The program asks the server…".to_string()));

        assert_eq!(
            rows.last(),
            Some(&Row::Note("The program asks the server…".to_string()))
        );
    }

    #[test]
    fn the_mark_stands_beside_the_choice_of_the_program() {
        let rows = rows(false, &the_filters(), None);
        let text: Vec<String> = rows
            .iter()
            .map(|row| line_of(row, "addedAt", true, "authors.Y2M1"))
            .collect();

        assert!(text.contains(&"✓ The time when the book came".to_string()));
        assert!(text.contains(&"  The title".to_string()));
        assert!(text.contains(&"✓ Long Author".to_string()));
        assert!(text.contains(&"  Lewis Carroll".to_string()));
        assert!(text.contains(&"  No filter".to_string()));
        assert!(text.contains(&"  The direction: the largest first".to_string()));
    }

    #[test]
    fn no_choice_marks_the_sequence_of_the_server_and_no_filter() {
        let rows = rows(false, &the_filters(), None);
        let text: Vec<String> = rows.iter().map(|row| line_of(row, "", false, "")).collect();

        assert!(text.contains(&"✓ No filter".to_string()));
        assert!(text.contains(&"  The direction: the smallest first".to_string()));
        assert!(!text.iter().any(|line| line.starts_with("✓ The t")));
    }

    #[test]
    fn a_filter_of_an_author_holds_the_value_of_the_server() {
        assert_eq!(
            filter_value("authors", "cc5891d3-f0a5-42b0-ac39-6c33df199efd"),
            "authors.Y2M1ODkxZDMtZjBhNS00MmIwLWFjMzktNmMzM2RmMTk5ZWZk"
        );
    }
}

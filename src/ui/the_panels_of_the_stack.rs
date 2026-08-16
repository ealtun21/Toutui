//! The panels 2 and 3 of the stack: the sequence and the filter. See T-318.
//!
//! **This is the stage 5 of the road of the panels**, and it comes after the
//! frame of T-320, the mouse of T-316, and the table of T-321. The panel 1 of
//! the views stood alone in the stack of 34 columns, and the design of
//! `docs/mockups/mockup-1.txt` gives that stack three panels: the views, the
//! sequence, and the filter.
//!
//! **The measurement of the real program v0.8.148 of 2026-08-16 says the
//! fault**: the Library view of 160 columns held the header
//! `4 Library [500 items of 2056]` and **no word of the sequence that stands
//! and no word of the filter that stands**. The user could read those two
//! values in the view of the key `f` alone, and that view takes the whole
//! screen and it hides the list that it describes.
//!
//! ## What this module holds, and what it does not
//!
//! **Every function here is pure**, therefore a test of it needs no terminal,
//! no server, and no `App` at all. The module holds:
//!
//! - the rows of the panel 2 and of the panel 3, which are the rows of
//!   `crate::logic::sort_filter` of the view of the key `f`, in two groups;
//! - the arithmetic that divides the stack between the three panels;
//! - the words of the sequence and of the filter for the header of the screen.
//!
//! **The two panels hold the rows of the view of the key `f` and no row of
//! their own** (T-118 in reverse): a panel that names a sequence which the
//! request of the items does not send is a text that promises a function that
//! the program does not have. The panel 2 therefore holds
//! `crate::logic::sort_filter::sorts_of` and the line of the direction, and the
//! panel 3 holds the line of no filter and the three places of the user.
//!
//! **The authors, the series, the narrators, and the tags stay in the view of
//! the key `f`**: those rows come of `GET /api/libraries/:id/filterdata`, which
//! goes when the user opens that view, and a panel of the start that made that
//! request would give every start of the program one request more. The panel 3
//! names that view for them.
//!
//! ## The words of the header, and the shape of two columns
//!
//! **The stack stands at 120 columns and up** (`crate::ui::frame::the_shape_of`),
//! therefore a terminal of 84 to 119 columns draws no panel 2 and no panel 3 at
//! all. The decision 3 of the road of the panels says that the status bar of
//! such a screen keeps the words of the sequence and of the filter, and
//! [`the_words_of_the_sequence_and_the_filter`] writes them.

use ratatui::layout::{Constraint, Layout, Rect};

use crate::logic::sort_filter::{self, FilterChoice, Row};

/// The smallest number of rows of the panel 1 that this program draws.
///
/// Two rows go to the border, and three rows hold the Home view, the Library
/// view, and one line more. **A panel of a title and of no line at all is a
/// text that promises a function that the program does not have** (T-118),
/// therefore the panels 2 and 3 go away before the panel 1 loses its lines.
pub const THE_SMALLEST_PANEL_OF_THE_VIEWS: u16 = 5;

/// The rows of the panel 2 of the sequence.
///
/// The rows are the rows of the group "The sequence" of the view of the key
/// `f`: one row for every field that the program knows, and the row of the
/// direction after them.
pub fn the_rows_of_the_sequence(is_podcast: bool) -> Vec<Row> {
    let mut out: Vec<Row> = sort_filter::sorts_of(is_podcast)
        .iter()
        .map(|one| Row::Sort {
            field: one.field.to_string(),
            label: one.label.to_string(),
        })
        .collect();

    out.push(Row::Direction);

    // **A library of podcasts holds no series** (T-324), and the row of the
    // whole library therefore stands for a library of books alone.
    if !is_podcast {
        out.push(Row::TheWholeLibrary);
    }

    out
}

/// The rows of the panel 3 of the filter.
///
/// The rows are the row of no filter and the three places of the user. See the
/// head of this module for the reason that the authors, the series, the
/// narrators, and the tags stay in the view of the key `f`.
pub fn the_rows_of_the_filter() -> Vec<Row> {
    let mut out = vec![Row::NoFilter];

    out.extend(
        sort_filter::progress_choices()
            .into_iter()
            .map(|one| Row::Filter {
                label: one.label,
                value: one.value,
            }),
    );

    out
}

/// The number of rows of a panel of the stack that holds this many lines.
///
/// Two rows of the panel go to its border.
pub fn the_height_of_a_panel(lines: usize) -> u16 {
    u16::try_from(lines).unwrap_or(u16::MAX).saturating_add(2)
}

/// The lines of a panel of the stack, of the width of the inside of it.
///
/// **A line that is longer than the panel loses its end and not its start**
/// (the rule of T-304): the start of a name says which sequence it is.
/// `crate::logic::message::in_one_row` does that work, and two columns of the
/// width go to the sign of the cursor of ratatui.
pub fn the_lines_of_a_panel(
    rows: &[Row],
    width: u16,
    field: &str,
    desc: bool,
    filter: &str,
) -> Vec<String> {
    let of_the_line = width.saturating_sub(2);

    rows.iter()
        .map(|row| {
            crate::logic::message::in_one_row(
                &sort_filter::line_of(row, field, desc, filter),
                of_the_line,
            )
        })
        .collect()
}

/// Divides the stack of the panels between the panel 1, the panel 2, and the
/// panel 3.
///
/// The panel 2 and the panel 3 take the rows that they need, and the panel 1
/// takes the rows that stay. **A stack that is too short loses the panel 3
/// first and the panel 2 after it**, and the panel 1 never goes under
/// [`THE_SMALLEST_PANEL_OF_THE_VIEWS`] rows: a panel of a title and of no line
/// at all is the fault of T-118.
///
/// A panel that this function gives no area for holds no cell of the screen at
/// all, therefore it takes no click of the mouse and no digit of the focus
/// (T-79).
pub fn the_three_panels(
    stack: Rect,
    of_the_sequence: u16,
    of_the_filter: u16,
) -> (Rect, Option<Rect>, Option<Rect>) {
    let room = stack.height;

    if room >= THE_SMALLEST_PANEL_OF_THE_VIEWS + of_the_sequence + of_the_filter {
        let [views, sequence, filter] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(of_the_sequence),
            Constraint::Length(of_the_filter),
        ])
        .areas(stack);

        return (views, Some(sequence), Some(filter));
    }

    if room >= THE_SMALLEST_PANEL_OF_THE_VIEWS + of_the_sequence {
        let [views, sequence] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(of_the_sequence)])
                .areas(stack);

        return (views, Some(sequence), None);
    }

    (stack, None, None)
}

/// The name for the user of the filter that stands.
///
/// **The value of a filter of the server is a name in base64 of an identity**
/// (`authors.Y2M1ODkxZDMt…`), therefore no arithmetic gives the name of it
/// back: the name comes of the list of the choices. The three places of the
/// user stand in this program (`sort_filter::PROGRESS`), and every other choice
/// comes of `GET /api/libraries/:id/filterdata`, which the view of the key `f`
/// asks for.
///
/// **A filter whose name did not come names its group and no word more**: the
/// user then reads that a filter stands, and the view of the key `f` gives the
/// name of it.
pub fn the_name_of_a_filter(value: &str, of_the_server: &[FilterChoice]) -> String {
    if value.is_empty() {
        return "No filter".to_string();
    }

    let of_the_program = sort_filter::progress_choices();

    for one in of_the_program.iter().chain(of_the_server.iter()) {
        if one.value == value {
            return one.label.clone();
        }
    }

    match value.split('.').next().unwrap_or_default() {
        "authors" => "An author".to_string(),
        "series" => "A series".to_string(),
        "narrators" => "A narrator".to_string(),
        "genres" => "A genre".to_string(),
        "tags" => "A tag".to_string(),
        "languages" => "A language".to_string(),
        "progress" => "Your position".to_string(),
        _ => "A filter of the server".to_string(),
    }
}

/// The words of the sequence and of the filter for the header of the screen.
///
/// **The stack of the panels stands at 120 columns and up**, therefore a
/// terminal of 84 to 119 columns holds no panel 2 and no panel 3, and the
/// header of it keeps these words (the decision 3 of the road of the panels).
///
/// The words name the sequence, the direction of it, and the filter. A library
/// whose sequence is the sequence of the server names that
/// (`sort_filter::label_of`), and it names no direction: the direction of a
/// request with no `sort` reaches the server in no field at all.
pub fn the_words_of_the_sequence_and_the_filter(
    is_podcast: bool,
    field: &str,
    desc: bool,
    filter: &str,
    of_the_server: &[FilterChoice],
) -> String {
    let of_the_sequence = if field.is_empty() {
        "The sequence of the server".to_string()
    } else {
        format!(
            "{}, {}",
            sort_filter::label_of(field, is_podcast),
            if desc {
                "the largest first"
            } else {
                "the smallest first"
            }
        )
    };

    format!(
        "⇅ {} ▣ {}",
        of_the_sequence,
        the_name_of_a_filter(filter, of_the_server)
    )
}

/// The columns between the words of the header and the part beside them.
///
/// **The header holds three parts on one row, and each of them writes its own
/// letters only** (T-115). A gap of two columns is the gap that the row of the
/// address held at 80 columns before T-329, therefore a screen that stood
/// stands in the same shape.
pub const THE_GAP_OF_THE_WORDS: u16 = 2;

/// The column where the words of the sequence and of the filter start, and
/// `None` for a row that has no room for them. See T-329.
///
/// **The row of the address is not the row of the words alone.** The second row
/// of the header holds the address of the server at the left, the words of the
/// sequence and of the filter in the middle, and the notice of the key `R` at
/// the right, and every one of them is a paragraph of its own over the whole
/// area: a part that is too long therefore writes on the letters of its
/// neighbour, which is the fault of T-115 one row below. The measurement of the
/// real program v0.8.158 at 84 columns read
/// `🔗 localhost:13399title, the largest first ▣ The media that you finished`,
/// with no gap, no mark `⇅`, and no first word.
///
/// **The words keep the middle of the row while the middle is free**, therefore
/// every screen that stood before this correction stands in the same shape.
/// They stand beside the address when the middle is not free, and **a row with
/// no room for the whole of them holds none of them**: a text that the row cuts
/// says nothing to the user (T-91), and the view of the key `f` holds those two
/// values for every width of the screen already.
///
/// `at_the_left` and `at_the_right` are the columns of the two parts beside the
/// words, and `the_words` is the columns of the words themselves
/// (`crate::logic::message::the_columns_of`, and not the bytes of the text —
/// the marks `⇅` and `▣` take three bytes and one column each).
pub fn the_column_of_the_words(
    width: u16,
    at_the_left: u16,
    at_the_right: u16,
    the_words: u16,
) -> Option<u16> {
    let first = at_the_left.saturating_add(THE_GAP_OF_THE_WORDS);
    let after_the_last = width.saturating_sub(at_the_right.saturating_add(THE_GAP_OF_THE_WORDS));

    // A row whose two neighbours meet holds no word at all, and
    // `saturating_sub` gives 0 for the width that is too small.
    let the_room = after_the_last.saturating_sub(first);

    if the_words == 0 || the_words > the_room {
        return None;
    }

    // The middle of the whole row, which is the place that `Paragraph::centered`
    // gives. A row of an odd number of free columns keeps the rule of ratatui,
    // which puts the extra column at the right.
    let of_the_middle = width.saturating_sub(the_words) / 2;

    if of_the_middle >= first && of_the_middle.saturating_add(the_words) <= after_the_last {
        return Some(of_the_middle);
    }

    Some(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The rows of the panel 2 and of the panel 3 are the rows of the view of
    /// the key `f`, and every one of them is a line that the user can take.
    /// See T-318.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_two_panels_hold_the_rows_of_the_view_of_the_key_f() {
        let of_the_books = the_rows_of_the_sequence(false);

        // Seven fields of a library of books, the direction after them, and the
        // row of the whole library at the end (T-324).
        assert_eq!(of_the_books.len(), sort_filter::SORTS_OF_BOOKS.len() + 2);
        assert_eq!(of_the_books.last(), Some(&Row::TheWholeLibrary));
        assert_eq!(
            of_the_books.get(of_the_books.len() - 2),
            Some(&Row::Direction)
        );

        // A library of podcasts holds the three fields of a podcast, and no
        // field of a book: a field that the server does not know gives a
        // sequence that no one specified. **It holds no series either** (T-324),
        // therefore the row of the whole library stands in no panel of it.
        let of_the_podcasts = the_rows_of_the_sequence(true);
        assert_eq!(
            of_the_podcasts.len(),
            sort_filter::SORTS_OF_PODCASTS.len() + 1
        );
        assert!(!of_the_podcasts.contains(&Row::TheWholeLibrary));

        // The panel 3 holds the line of no filter and the three places of the
        // user.
        let of_the_filter = the_rows_of_the_filter();
        assert_eq!(of_the_filter.len(), 1 + sort_filter::PROGRESS.len());
        assert_eq!(of_the_filter.first(), Some(&Row::NoFilter));

        // **Every row of the two panels is a line that the user can take**: a
        // row of a title or of a note takes the key `l` of the user and it does
        // nothing, and that is the fault of T-79.
        for row in of_the_books.iter().chain(of_the_filter.iter()) {
            assert!(
                row.is_a_line_of_the_user(),
                "the row {row:?} takes no key of the user"
            );
        }

        // The line of a row holds the mark of the choice that stands, and it
        // keeps the width of the panel (the rule of T-304).
        let lines = the_lines_of_a_panel(&of_the_books, 32, "media.metadata.title", false, "");
        assert!(lines[0].starts_with("✓ "), "{:?}", lines[0]);
        assert!(lines[1].starts_with("  "), "{:?}", lines[1]);

        for line in &lines {
            assert!(
                crate::logic::message::the_columns_of(line) <= 30,
                "the line {line:?} stands over the 30 columns of the panel"
            );
        }

        // A panel of no width writes no line of a negative width.
        assert_eq!(
            the_lines_of_a_panel(&of_the_books, 0, "", false, "").len(),
            of_the_books.len()
        );
    }

    /// The stack gives the panel 2 and the panel 3 the rows that they need, and
    /// a stack that is too short loses them. See T-318.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_stack_loses_the_panel_3_before_the_panel_2() {
        let of_the_sequence = the_height_of_a_panel(the_rows_of_the_sequence(false).len());
        let of_the_filter = the_height_of_a_panel(the_rows_of_the_filter().len());

        // Seven fields, the direction, the row of the whole library (T-324),
        // and two rows of the border.
        assert_eq!(of_the_sequence, 11);
        // No filter and the three places of the user, and two rows of the
        // border.
        assert_eq!(of_the_filter, 6);

        // A stack of a terminal of 45 rows holds the three panels.
        let stack = Rect::new(0, 2, 34, 40);
        let (views, sequence, filter) = the_three_panels(stack, of_the_sequence, of_the_filter);
        let sequence = sequence.expect("the stack of 40 rows holds the panel 2");
        let filter = filter.expect("the stack of 40 rows holds the panel 3");

        assert_eq!(views.height, 40 - of_the_sequence - of_the_filter);
        assert_eq!(views.y, stack.y);
        assert_eq!(sequence.y, views.y + views.height);
        assert_eq!(filter.y, sequence.y + sequence.height);
        assert_eq!(filter.y + filter.height, stack.y + stack.height);

        // **The panel 1 keeps its smallest number of rows**, and the panel 3
        // goes away before it.
        let (views, sequence, filter) = the_three_panels(
            Rect::new(0, 2, 34, THE_SMALLEST_PANEL_OF_THE_VIEWS + of_the_sequence),
            of_the_sequence,
            of_the_filter,
        );
        assert_eq!(views.height, THE_SMALLEST_PANEL_OF_THE_VIEWS);
        assert!(sequence.is_some());
        assert_eq!(filter, None, "the stack has no room for the panel 3");

        // **A stack that holds no panel 2 gives every row to the panel 1**, and
        // the two panels then hold no cell of the screen at all: a click of the
        // mouse and a digit of the focus name nothing (T-79).
        let short = Rect::new(0, 2, 34, 8);
        assert_eq!(
            the_three_panels(short, of_the_sequence, of_the_filter),
            (short, None, None)
        );
        assert_eq!(
            the_three_panels(Rect::default(), of_the_sequence, of_the_filter),
            (Rect::default(), None, None)
        );
    }

    /// The words of the header name the sequence, the direction, and the
    /// filter. See T-318.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_words_of_the_header_name_the_sequence_and_the_filter() {
        let words = the_words_of_the_sequence_and_the_filter(
            false,
            "media.metadata.authorNameLF",
            false,
            "",
            &[],
        );
        assert!(words.contains("The author"), "{words}");
        assert!(words.contains("the smallest first"), "{words}");
        assert!(words.contains("No filter"), "{words}");

        let words = the_words_of_the_sequence_and_the_filter(false, "addedAt", true, "", &[]);
        assert!(words.contains("the largest first"), "{words}");

        // **A library with no sequence names the sequence of the server, and it
        // names no direction**: a request with no `sort` sends no `desc`.
        let words = the_words_of_the_sequence_and_the_filter(false, "", true, "", &[]);
        assert!(words.contains("The sequence of the server"), "{words}");
        assert!(!words.contains("first"), "{words}");

        // The three places of the user stand in this program.
        let of_the_place = sort_filter::filter_value("progress", "in-progress");
        assert_eq!(
            the_name_of_a_filter(&of_the_place, &[]),
            "Started, not finished"
        );

        // **A filter whose name did not come names its group**: the names of
        // the authors and of the series come of the server, and the value holds
        // an identity in base64 and no name at all.
        let of_an_author = sort_filter::filter_value("authors", "an-identity");
        assert_eq!(the_name_of_a_filter(&of_an_author, &[]), "An author");
        assert_eq!(
            the_name_of_a_filter(
                &of_an_author,
                &[FilterChoice {
                    label: "Ada Lovelace".to_string(),
                    group: "The authors",
                    value: of_an_author.clone(),
                }]
            ),
            "Ada Lovelace"
        );
        assert_eq!(the_name_of_a_filter("", &[]), "No filter");
        assert_eq!(
            the_name_of_a_filter("a-word-of-no-group", &[]),
            "A filter of the server"
        );
    }
}

//! The wrap of a line that holds spans. See T-363.
//!
//! [`crate::logic::message::the_parts_of_a_wrap`] is the one loop of the wrap of
//! this program (T-362), and it takes a text and it gives text. A line of the
//! view of the statistics and of the view of the sessions holds **spans**, and
//! each span holds a style of its own: the time of a session is bold, the bar of
//! a day has the colour of the accent, and the name of a group has that colour
//! too. A wrap that reads the text alone therefore loses the style of every
//! column.
//!
//! [`the_rows_of_a_line`] keeps the one loop and it keeps the style. It takes
//! the text of every span together, it gives that text to the loop of the wrap,
//! and it then cuts the spans again at the bytes of each row: a row of a wrap
//! holds the styles that its columns held before the wrap.
//!
//! **A row of a wrap that is not the first row stands at an indent**
//! ([`THE_INDENT_OF_A_ROW`]). A user of the view of the sessions reads a column
//! of the times at the left, and a name that goes over two rows with no indent
//! reads like a session of its own.

use ratatui::text::{Line, Span};

use crate::logic::message::{the_columns_of, the_parts_of_a_wrap};

/// The columns of the indent of a row of a wrap that is not the first row.
pub const THE_INDENT_OF_A_ROW: usize = 4;

/// Gives the rows that a line takes at a width, with the style of every span.
///
/// The line comes back whole, in one row, when it stands in the width already.
/// A line of a width of no column comes back whole too, because a wrap of no
/// column has no end.
pub fn the_rows_of_a_line(line: &Line<'static>, width: usize, indent: usize) -> Vec<Line<'static>> {
    if width == 0 || line.spans.is_empty() {
        return vec![line.clone()];
    }

    let text: String = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect();

    let ranges = the_ranges_of_a_wrap(&text, width, indent);

    if ranges.len() <= 1 {
        return vec![line.clone()];
    }

    let mut out: Vec<Line<'static>> = Vec::new();

    for (number, (from, to)) in ranges.iter().enumerate() {
        let mut spans = the_spans_of(line, *from, *to);

        // A row that no span gives back keeps its text, and it loses the style
        // alone. A view that loses a word says nothing (T-362).
        if spans.is_empty() && from < to {
            spans.push(Span::raw(text[*from..*to].to_string()));
        }

        if number > 0 && indent > 0 {
            spans.insert(0, Span::raw(" ".repeat(indent)));
        }

        let mut row = Line::from(spans);
        row.style = line.style;
        row.alignment = line.alignment;
        out.push(row);
    }

    out
}

/// Gives the bytes of each row of the wrap of a text.
///
/// **The first row holds the whole width, and a row after it holds the width
/// less the indent** (T-363). A wrap of one width for every row costs the first
/// row the columns of the indent, and a line of the statistics that stands in
/// the screen already then takes two rows: the bar of a day of `15 min 11 s`
/// lost the `s` of its time to a second row in the first form of this item.
///
/// **The whitespace at the start of the line stays on the first row.** The loop
/// of the wrap drops the whitespace at the start of a row, and the time of a
/// session stands to the right of a field of twelve columns: a wrap that drops
/// that field takes the column of the times away, and the times of the view
/// then stand at no column of their own.
///
/// The loop of the wrap of this program stands in one place (T-362), therefore
/// this function takes the **first** row of that loop and it gives the rest of
/// the text to it again.
fn the_ranges_of_a_wrap(text: &str, width: usize, indent: usize) -> Vec<(usize, usize)> {
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    let mut at = 0usize;

    while at < text.len() {
        let first = ranges.is_empty();
        let room = if first {
            width
        } else {
            width.saturating_sub(indent)
        }
        .max(1);

        let rest = &text[at..];
        let head = rest.len() - rest.trim_start().len();

        // The whitespace of the start of a row that is not the first row went
        // away with the row before it.
        if !first {
            at += head;
            if at >= text.len() {
                break;
            }
        }

        let start = at;
        let head = if first { head } else { 0 };
        let body = &text[start + head..];

        if body.is_empty() {
            break;
        }

        let of_the_head = the_columns_of(&text[start..start + head]);
        let parts = the_parts_of_a_wrap(body, room.saturating_sub(of_the_head).max(1));

        let Some(part) = parts.first() else {
            break;
        };

        let end = (start + head + part.len()).min(text.len());

        // A row of no byte would hold this loop for ever.
        if end <= start {
            break;
        }

        ranges.push((start, end));
        at = end;
    }

    if ranges.is_empty() {
        ranges.push((0, text.len()));
    }

    ranges
}

/// Gives the spans of the bytes `from` to `to` of the text of a line.
fn the_spans_of(line: &Line<'static>, from: usize, to: usize) -> Vec<Span<'static>> {
    let mut out: Vec<Span<'static>> = Vec::new();
    let mut at = 0usize;

    for span in &line.spans {
        let content: &str = span.content.as_ref();
        let start = at;
        let end = at + content.len();
        at = end;

        if end <= from || start >= to {
            continue;
        }

        let first = from.max(start) - start;
        let last = to.min(end) - start;

        if !content.is_char_boundary(first) || !content.is_char_boundary(last) || first >= last {
            continue;
        }

        out.push(Span::styled(content[first..last].to_string(), span.style));
    }

    out
}

/// Gives the rows of every line of a view at a width.
///
/// **This is the rule of the two views of a text of a scroll** (T-363): every
/// line of them stands in the width of its panel, and a line that is longer
/// takes the rows that it needs. The render of each of those two views counts
/// the lines that it draws for the end of the scroll, therefore the keys `j`
/// and `k` reach every row of a wrap with no other change.
pub fn the_rows_of_the_lines(lines: &[Line<'static>], width: usize) -> Vec<Line<'static>> {
    lines
        .iter()
        .flat_map(|line| the_rows_of_a_line(line, width, THE_INDENT_OF_A_ROW))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::{Modifier, Style};

    fn the_text_of(line: &Line<'static>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    #[test]
    fn a_line_that_stands_in_the_width_comes_back_whole() {
        let line = Line::from("Today: 1 h 46 min".to_string());
        let rows = the_rows_of_a_line(&line, 40, 4);

        assert_eq!(rows.len(), 1);
        assert_eq!(the_text_of(&rows[0]), "Today: 1 h 46 min");
    }

    #[test]
    fn a_line_of_no_column_comes_back_whole() {
        let line = Line::from("A name of many words that no width holds".to_string());
        let rows = the_rows_of_a_line(&line, 0, 4);

        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn a_wrap_keeps_every_word_of_the_line() {
        let text = "3. A Long Test Book — Long Author  (1 h 26 min)";
        let line = Line::from(text.to_string());
        let rows = the_rows_of_a_line(&line, 38, 4);

        assert!(rows.len() > 1, "the line of 46 columns takes one row");

        let together: String = rows
            .iter()
            .map(|row| the_text_of(row).trim().to_string())
            .collect::<Vec<String>>()
            .join(" ");

        for word in text.split_whitespace() {
            assert!(
                together.contains(word),
                "the wrap lost the word {word:?}: {together:?}"
            );
        }
    }

    #[test]
    fn no_row_of_a_wrap_is_wider_than_the_width() {
        let line = Line::from(
            "2. A Second Book Of Many Hours — Many Hours Author  (8 h 00 min)".to_string(),
        );

        for width in 10..=80usize {
            for row in the_rows_of_a_line(&line, width, THE_INDENT_OF_A_ROW) {
                let columns = crate::logic::message::the_columns_of(&the_text_of(&row));
                assert!(
                    columns <= width,
                    "a row of {columns} columns at a width of {width}"
                );
            }
        }
    }

    #[test]
    fn a_row_that_is_not_the_first_row_stands_at_the_indent() {
        let line = Line::from("A name of many words that one row does not hold".to_string());
        let rows = the_rows_of_a_line(&line, 20, THE_INDENT_OF_A_ROW);

        assert!(rows.len() > 1);

        for row in rows.iter().skip(1) {
            assert!(
                the_text_of(row).starts_with("    "),
                "a row of a wrap with no indent: {:?}",
                the_text_of(row)
            );
        }
    }

    #[test]
    fn the_wrap_keeps_the_style_of_every_span() {
        let bold = Style::default().add_modifier(Modifier::BOLD);
        let line = Line::from(vec![
            Span::raw("  ".to_string()),
            Span::styled("   2 min 34 s".to_string(), bold),
            Span::raw("  A Second Book Of Many Hours — Many Hours Author".to_string()),
        ]);

        let rows = the_rows_of_a_line(&line, 30, THE_INDENT_OF_A_ROW);
        assert!(rows.len() > 1);

        // The time stands on the first row, and it keeps the bold.
        let of_the_time: String = rows[0]
            .spans
            .iter()
            .filter(|span| span.style.add_modifier.contains(Modifier::BOLD))
            .map(|span| span.content.as_ref())
            .collect();

        assert!(
            of_the_time.contains("2 min 34 s"),
            "the time lost the bold: {of_the_time:?}"
        );
    }

    /// The first form of T-363 gave the first row the width less the indent,
    /// and the bar of a day of the statistics then lost the `s` of its time to
    /// a second row while it stood in the screen.
    #[test]
    fn a_line_that_fills_the_width_takes_one_row() {
        // The shape of a bar of the statistics at 38 columns: a name of 11
        // columns, a bar of 11, two spaces, and a time of 11.
        let line = Line::from(vec![
            Span::raw("2026-08-12 ".to_string()),
            Span::raw("▌          ".to_string()),
            Span::raw("  ".to_string()),
            Span::raw("15 min 11 s".to_string()),
        ]);

        let columns = the_columns_of(&the_text_of(&line));
        assert!(columns <= 38, "the line of the measurement takes {columns}");

        let rows = the_rows_of_a_line(&line, 38, THE_INDENT_OF_A_ROW);
        assert_eq!(
            rows.len(),
            1,
            "a line of {columns} columns took {} rows at a width of 38",
            rows.len()
        );
    }

    /// The first form of T-363 dropped the whitespace of the start of the line,
    /// and the times of the view of the sessions then stood at no column of
    /// their own.
    #[test]
    fn the_wrap_keeps_the_field_of_the_start_of_the_line() {
        let of_the_line = |time: &str| {
            Line::from(vec![
                Span::raw("  ".to_string()),
                Span::raw(format!("{time:>12}")),
                Span::raw("  A Second Book Of Many Hours — Many Hours Author".to_string()),
            ])
        };

        let one = the_rows_of_a_line(&of_the_line("2 min 34 s"), 38, THE_INDENT_OF_A_ROW);
        let two = the_rows_of_a_line(&of_the_line("12 min 59 s"), 38, THE_INDENT_OF_A_ROW);

        assert!(one.len() > 1, "the line of the measurement takes one row");

        // The time of each of the two rows ends at the same column.
        let of_the_end = |row: &Line<'static>, time: &str| {
            let text = the_text_of(row);
            let at = text.find(time).expect("the row holds the time");
            the_columns_of(&text[..at]) + the_columns_of(time)
        };

        assert_eq!(
            of_the_end(&one[0], "2 min 34 s"),
            of_the_end(&two[0], "12 min 59 s"),
            "the times of the two rows stand at two columns: {:?} and {:?}",
            the_text_of(&one[0]),
            the_text_of(&two[0])
        );
    }

    #[test]
    fn the_wrap_of_a_word_that_no_row_holds_keeps_that_word() {
        let line = Line::from("AVeryLongNameOfABookWithNoSpaceAtAll".to_string());
        let rows = the_rows_of_a_line(&line, 12, 0);

        let together: String = rows.iter().map(the_text_of).collect();
        assert_eq!(together, "AVeryLongNameOfABookWithNoSpaceAtAll");
    }
}

//! The screen of the reader of an ebook. See T-10.
//!
//! The screen has three parts: a line at the top with the name of the book and
//! the chapter, the text in the middle, and the keys at the bottom. The key
//! `t` puts the table of contents in the place of the text.
//!
//! The functions that make a text are pure, therefore a test can examine them
//! with no terminal.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{
    Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget, Widget,
    Wrap,
};

use crate::logic::reader::Reader;

/// The largest width of a line of text, in columns.
///
/// A line of 200 columns is hard to read, because the eye loses the start of
/// the next line. A book on paper holds about 70 characters in a line.
pub const MAX_TEXT_WIDTH: u16 = 100;

/// Gives the area of the text inside the area of the reader.
///
/// The text stands in the middle of a wide terminal, and it never becomes
/// wider than `MAX_TEXT_WIDTH`.
pub fn text_area(area: Rect) -> Rect {
    let width = area.width.min(MAX_TEXT_WIDTH);

    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y,
        width,
        height: area.height,
    }
}

/// Makes the line at the top of the screen.
pub fn header(title: &str, chapter: usize, count: usize, part: f64) -> String {
    line_of_the_top(title, chapter, count, part, false)
}

/// Makes the line at the top of the screen, and it names the part of the book.
///
/// One chapter of a PDF is one page. Therefore the line says "page" for such a
/// book: a user of a PDF looks for a page, and the word "chapter" says nothing
/// about that file. See T-54.
pub fn line_of_the_top(
    title: &str,
    chapter: usize,
    count: usize,
    part: f64,
    holds_pages: bool,
) -> String {
    let part = (part * 100.0).round().clamp(0.0, 100.0) as i64;
    let word = if holds_pages { "page" } else { "chapter" };

    format!(
        "{} — {} {} of {} — {}%",
        title,
        word,
        chapter + 1,
        count.max(1),
        part
    )
}

/// Draws the reader.
pub fn render(reader: &mut Reader, area: Rect, buf: &mut Buffer) {
    let [top, middle, bottom] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(2),
    ])
    .areas(area);

    Paragraph::new(line_of_the_top(
        &reader.title,
        reader.chapter,
        reader.chapter_count(),
        reader.fraction(),
        reader.holds_pages(),
    ))
    .style(Style::default().add_modifier(Modifier::BOLD))
    .render(top, buf);

    // One chapter of a PDF is one page, therefore the keys of that book name a
    // page and not a chapter. See T-54.
    let keys = match (reader.contents_open, reader.holds_pages()) {
        (true, false) => {
            "j/k: move  l/Enter: go to the chapter  t: back to the text  h: leave the book"
        }
        (true, true) => "j/k: move  l/Enter: go to the page  t: the pages  h: leave the book",
        (false, false) => {
            "j/k: line  Space/b: screen  n/p: chapter  t: contents  g/G: start/end\n \
             s: send the position  ?: every key  h: leave the book  Q: quit"
        }
        (false, true) => {
            "j/k: line  Space/b: screen  n/p: page  t: the pages  g/G: start/end\n \
             s: send the position  ?: every key  h: leave the book  Q: quit"
        }
    };

    Paragraph::new(keys)
        .centered()
        .style(Style::default().fg(Color::Rgb(120, 120, 120)))
        .render(bottom, buf);

    if reader.contents_open {
        render_contents(reader, middle, buf);
        return;
    }

    let inside = text_area(middle);

    // The task renders for this width. The screen shows the lines that are
    // ready, and a message while it waits.
    reader.render_for(inside.width);

    if reader.lines.is_empty() {
        let message = reader
            .message
            .clone()
            .unwrap_or_else(|| "This chapter has no text.".to_string());

        // **A sentence that says why is longer than one line** (T-277): a
        // paragraph with no wrap cuts that sentence at the width of the screen,
        // and the user then reads no reason and no key.
        Paragraph::new(message)
            .centered()
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::Rgb(150, 150, 150)))
            .render(inside, buf);

        return;
    }

    let top_line = reader.top_line.min(reader.lines.len().saturating_sub(1));
    let visible: Vec<Line<'static>> = reader
        .lines
        .iter()
        .skip(top_line)
        .take(usize::from(inside.height))
        .cloned()
        .collect();

    Paragraph::new(visible)
        .wrap(Wrap { trim: false })
        .render(inside, buf);
}

/// Draws the table of contents.
fn render_contents(reader: &mut Reader, area: Rect, buf: &mut Buffer) {
    let items: Vec<ListItem> = reader
        .contents
        .iter()
        .map(|entry| {
            // The depth of an entry gives the space before its name. The user
            // then sees a part and its chapters.
            let space = " ".repeat(entry.depth * 2);
            ListItem::new(format!("{}{}", space, entry.label))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(
        reader.contents_line.min(items.len().saturating_sub(1)),
    ));

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("The contents of the book"),
        )
        .highlight_symbol("➤ ")
        .highlight_spacing(HighlightSpacing::Always)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    StatefulWidget::render(list, area, buf, &mut state);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_text_stands_in_the_middle_of_a_wide_screen() {
        let area = text_area(Rect::new(0, 0, 200, 30));
        assert_eq!(area.width, MAX_TEXT_WIDTH);
        assert_eq!(area.x, 50);
    }

    #[test]
    fn a_narrow_screen_gives_every_column_to_the_text() {
        let area = text_area(Rect::new(0, 0, 60, 30));
        assert_eq!(area.width, 60);
        assert_eq!(area.x, 0);
    }

    #[test]
    fn the_area_of_the_text_stays_inside_the_area() {
        for width in 1..250u16 {
            let whole = Rect::new(3, 5, width, 20);
            let inside = text_area(whole);

            assert!(inside.x >= whole.x);
            assert!(inside.x + inside.width <= whole.x + whole.width);
            assert_eq!(inside.height, whole.height);
        }
    }

    #[test]
    fn the_header_names_the_chapter_and_the_part() {
        let text = header("A Book", 2, 14, 0.213);
        assert!(text.contains("A Book"));
        assert!(text.contains("chapter 3 of 14"));
        assert!(text.contains("21%"));
    }

    #[test]
    fn a_book_with_no_chapter_gives_no_division_by_zero() {
        let text = header("A Book", 0, 0, 0.0);
        assert!(text.contains("chapter 1 of 1"));
    }

    #[test]
    fn a_part_outside_the_limits_stays_inside_them() {
        assert!(header("A", 0, 1, 5.0).contains("100%"));
        assert!(header("A", 0, 1, -3.0).contains("0%"));
    }
}

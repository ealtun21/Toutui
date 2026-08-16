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

/// Makes the line at the top of the screen, for a screen of every width.
///
/// The width of 0 says that the caller has no screen: the line then holds
/// every character, and the tests of the words of this line need no width at
/// all.
pub fn header(title: &str, chapter: usize, count: usize, part: f64) -> String {
    line_of_the_top(title, chapter, count, part, false, 0)
}

/// Makes the line at the top of the screen, and it names the part of the book.
///
/// One chapter of a PDF is one page. Therefore the line says "page" for such a
/// book: a user of a PDF looks for a page, and the word "chapter" says nothing
/// about that file. See T-54.
///
/// **The place of the user keeps its room, and the title loses its end**
/// (T-300). This line stands in a `Paragraph` of one row with no `wrap`,
/// therefore a title that fills the width of the terminal takes the number of
/// the chapter, the count of the chapters, and the percent away. The
/// measurement of 2026-08-16 gave a book of the title of Robinson Crusoe of
/// Project Gutenberg (153 characters) to the reader, and the line at 80 columns
/// said `The Life and Adventures of Robinson Crusoe, of York, Mariner: Who
/// Lived Eight an` at the chapter 1, at the chapter 2, and at the chapter 3:
/// the three lines held the same characters, and the user read no place of
/// their own at all. **A terminal of 160 columns lost the same numbers.**
///
/// The title says what the user chose already, and the view of the media holds
/// it too; the place of the user is what this line measures, and it is the one
/// part of it that changes. Therefore the title takes the room that stays, and
/// it loses its end to three points.
///
/// A width of 0 says that the caller has no screen, and the line then holds
/// every character.
pub fn line_of_the_top(
    title: &str,
    chapter: usize,
    count: usize,
    part: f64,
    holds_pages: bool,
    width: u16,
) -> String {
    let part = (part * 100.0).round().clamp(0.0, 100.0) as i64;
    let word = if holds_pages { "page" } else { "chapter" };

    // **The header says the number that the program measured** (T-283): a
    // `count.max(1)` of an older version kept the division away, and it told
    // the user that a book of no chapter holds one chapter and that the reader
    // stands in it. The program measured 0, therefore the line says that the
    // book holds no chapter, and it names no number of a chapter and no part.
    let the_place = if count == 0 {
        format!(" — this book holds no {word}")
    } else {
        format!(" — {} {} of {} — {}%", word, chapter + 1, count, part)
    };

    the_line_that_stands(title, &the_place, width)
}

/// Puts a title and the place of the user in a width of columns.
///
/// **The place of the user comes first** (T-300): it stands whole while one
/// column stays for it, and the title takes the room after it. A title that
/// does not stand loses its end to three points, and a place of the user that
/// is wider than the whole screen loses its end in the same way.
///
/// The function is pure, therefore a test needs no screen.
pub fn the_line_that_stands(title: &str, the_place: &str, width: u16) -> String {
    if width == 0 {
        return format!("{title}{the_place}");
    }

    let width = usize::from(width);
    let letters = title.chars().count();
    let place = the_place.chars().count();

    if letters + place <= width {
        return format!("{title}{the_place}");
    }

    // The place of the user alone is wider than the screen: no room stays for
    // the title, and the place then loses its own end.
    if place + 1 > width {
        return in_one_row(the_place, width);
    }

    // One column stays for the three points of the title.
    let kept: String = title.chars().take(width - place - 1).collect();

    format!("{}…{}", kept.trim_end(), the_place)
}

/// Gives a text that stands in a width of columns, with three points for the
/// end that goes away. See T-300.
fn in_one_row(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    if text.chars().count() <= width {
        return text.to_string();
    }

    if width == 1 {
        return "…".to_string();
    }

    let kept: String = text.chars().take(width - 1).collect();

    format!("{}…", kept.trim_end())
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
        top.width,
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
    /// **The header of a book of no chapter says what the program measured**
    /// (T-283): the older line said "chapter 1 of 1", and the book holds no
    /// chapter at all.
    fn a_book_with_no_chapter_names_no_chapter_of_its_own() {
        let text = header("A Book", 0, 0, 0.0);
        assert!(text.contains("A Book"));
        assert!(text.contains("holds no chapter"), "{text}");
        assert!(!text.contains("of 1"), "{text}");
        assert!(!text.contains("chapter 1"), "{text}");

        // A PDF of no page says the same of a page. See T-54.
        let text = line_of_the_top("A PDF", 0, 0, 0.0, true, 0);
        assert!(text.contains("holds no page"), "{text}");
    }

    /// **The place of the user keeps its room** (T-300).
    ///
    /// The parts of this test stay in one function: the widths and the titles
    /// are one measurement of one line.
    #[test]
    fn a_long_title_never_takes_the_place_of_the_user_away() {
        // The title of Robinson Crusoe of Project Gutenberg, of the
        // measurement of 2026-08-16.
        let title = "The Life and Adventures of Robinson Crusoe, of York, \
                     Mariner: Who Lived Eight and Twenty Years All Alone in an \
                     Uninhabited Island on the Coast of America";
        assert_eq!(title.chars().count(), 153);

        for width in [40u16, 80, 100, 160] {
            let text = line_of_the_top(title, 1, 3, 0.5, false, width);

            assert!(
                text.chars().count() <= usize::from(width),
                "the line of {width} columns stands in them: {text}"
            );
            assert!(
                text.contains("chapter 2 of 3"),
                "the line of {width} columns holds the chapter: {text}"
            );
            assert!(
                text.contains("50%"),
                "the line of {width} columns holds the percent: {text}"
            );
            assert!(
                text.starts_with("The Life"),
                "the line of {width} columns starts with the title: {text}"
            );
            assert!(
                text.contains('…'),
                "the line of {width} columns says that the title lost its end: {text}"
            );
        }

        // A book of no chapter says so at every width. See T-283.
        let text = line_of_the_top(title, 0, 0, 0.0, false, 80);
        assert!(text.contains("holds no chapter"), "{text}");
        assert!(text.chars().count() <= 80, "{text}");

        // A title that stands loses nothing at all.
        let text = line_of_the_top("A Book", 1, 3, 0.5, false, 80);
        assert_eq!(text, "A Book — chapter 2 of 3 — 50%");
        assert!(!text.contains('…'), "{text}");

        // A screen that holds no room for the place of the user takes the end
        // of that place, and the title goes away with it.
        let text = line_of_the_top(title, 1, 3, 0.5, false, 10);
        assert_eq!(text.chars().count(), 10, "{text}");
        assert!(text.ends_with('…'), "{text}");

        // A width of 1 and a width of 0 give no panic at all.
        assert_eq!(line_of_the_top(title, 1, 3, 0.5, false, 1), "…");
        assert!(line_of_the_top(title, 1, 3, 0.5, false, 0).contains("chapter 2 of 3"));
    }

    #[test]
    fn a_part_outside_the_limits_stays_inside_them() {
        assert!(header("A", 0, 1, 5.0).contains("100%"));
        assert!(header("A", 0, 1, -3.0).contains("0%"));
    }
}

//! The render of the list of a view, and the bar of the scroll of it.
//! See T-255 and T-256.
//!
//! **No test of this repository drew a frame of the program into a buffer**
//! (the paragraph of T-253, of T-254, and of T-255 that stayed open through
//! three rounds). `render_list` was a private method of `App`, therefore the
//! bar of the scroll of T-255 stood on the measurement of tmux alone, and a
//! build that lost that bar broke no test at all.
//!
//! The render of a list needs the colors of the user, the title, the lines, and
//! the state of the list, and it needs no other part of `App`. It stands here
//! as a function of its own, and the tests of this module draw it into a
//! `Buffer` and they read the characters of that buffer. A `Buffer` needs no
//! terminal and no screen, therefore those tests run in the gate.

use crate::config::Colors;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Scrollbar,
        ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget,
    },
};

/// Gives the background of the line `i` of a list. The even lines take one
/// colour of the user and the odd lines take the other.
///
/// **The render of the program reads no disk** (T-204 and T-257). This function
/// takes the colours that `App` read at its start, and it opens no file: the
/// function before it called `load_config()` one time for each line of each
/// frame, and a measurement of the Library view of 500 lines counted **14061
/// opens of `config.toml` in ten seconds**.
pub fn the_colour_of_a_line(colors: &Colors, i: usize) -> Color {
    if i.is_multiple_of(2) {
        colors.list_background()
    } else {
        colors.list_background_alt_row()
    }
}

/// Draws the list of a view, and the bar of the scroll of it.
///
/// `title` stands in the header of the block, `lines` are the lines of the
/// list, and `list_state` holds the cursor of the user and the offset of the
/// panel. ratatui writes that offset while it draws the list.
///
/// **The background of each line comes of this function** (T-257): the caller
/// gave the lines with a colour already, therefore no test of this module
/// reached the colours of the list at all.
pub fn render_the_list(
    area: Rect,
    buf: &mut Buffer,
    colors: &Colors,
    title: &str,
    lines: &[String],
    list_state: &mut ListState,
) {
    render_the_list_of_a_panel(area, buf, colors, None, title, lines, list_state);
}

/// Draws the list of a view inside the panel 4 of the frame of the panels, or
/// inside the block of a header alone. See T-320.
///
/// `the_panel` holds the number of the panel and the mark of the focus of it.
/// **The value `None` gives the block of one border at the top**, which is the
/// block of every view that the road of the panels did not reach yet: the stage
/// 2 of that road draws the Home view and the Library view, and the views after
/// them come with the stages after it.
pub fn render_the_list_of_a_panel(
    area: Rect,
    buf: &mut Buffer,
    colors: &Colors,
    the_panel: Option<(u8, bool)>,
    title: &str,
    lines: &[String],
    list_state: &mut ListState,
) {
    // **A line of a list stands on one row of the panel** (T-311). A `ListItem`
    // of a text that holds a `\n` takes the rows of the ends of the lines of it,
    // and every rule of a list of this program then fails together: the mark of
    // the line and the sign of the cursor stand on the first row alone, the row
    // after it names a media that the library does not hold, and the bar of the
    // scroll counts the lines of the list and not the rows of the panel (T-255).
    // `in_one_line` is the one place of that rule.
    let items: Vec<ListItem> = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            ListItem::new(crate::logic::message::in_one_line(line).into_owned())
                .bg(the_colour_of_a_line(colors, i))
        })
        .collect();

    // **A colour of the configuration that holds no three numbers is a colour
    // that the program does not have** (T-257): `rgb_parts` gives the value of
    // the program for a number that the file does not hold, and an index of the
    // list stops the program of the user at the first frame.
    let selected_style: Style = Style::new()
        .bg(colors.list_selected_background())
        .fg(colors.list_selected_foreground())
        .add_modifier(Modifier::BOLD);

    let header_style: Style = Style::new()
        .fg(colors.line_header())
        .bg(colors.header_background());

    // **The title of a list keeps its start** (T-304). ratatui centers a title
    // that stands, and a title that is wider than the block takes
    // `render_centered_titles_with_truncation`: that road gives the title an
    // area of `width - (title - width) / 2` columns and it draws it
    // **right-aligned** in that area, therefore the title loses its start and
    // its end together. The measurement of 2026-08-16, of a terminal of 40
    // columns: the title "Search result [2 items, with the books of Many Hours
    // Author]" of 60 characters gave `he books of Many Hours Author]` and ten
    // characters of the border, and the user read no name of the view and no
    // number of its items. The three points say that the screen cut it.
    // **The title of the panel 4 keeps the two corners of its border and the
    // number of the panel**: those six columns come away before the title of
    // the view takes what stays.
    let of_the_title = if the_panel.is_some() {
        area.width.saturating_sub(6)
    } else {
        area.width
    };
    let title = crate::logic::message::in_one_row(title, of_the_title);

    // **The panel 4 of the frame of the panels holds a border of four sides**
    // (T-320), and the shape of that border says the focus of the user. The
    // block of a view that the road of the panels did not reach keeps the one
    // border at the top that it had.
    let block = match the_panel {
        Some((number, it_holds_the_focus)) => {
            crate::ui::frame::a_panel(number, &title, it_holds_the_focus)
                .bg(colors.list_background())
        }
        None => Block::new()
            .title(Line::raw(title).centered())
            .borders(Borders::TOP)
            .border_style(header_style)
            .bg(colors.list_background()),
    };

    // **A list that holds more lines than its rows says so** (T-255). The block
    // draws the header of the view over the whole width, and the list and the
    // bar of the scroll then divide the area below it.
    let inner = block.inner(area);
    let the_list = crate::logic::the_scroll_of_a_list::the_list_of_the_render(
        items.len(),
        inner.width,
        inner.height,
    );

    block.render(area, buf);

    let [list_area, bar_area] = Layout::horizontal([
        Constraint::Length(the_list.width_of_the_lines),
        Constraint::Fill(1),
    ])
    .areas(inner);

    let list = List::new(items)
        .highlight_style(selected_style)
        .highlight_symbol("➤ ")
        .highlight_spacing(HighlightSpacing::Always);

    StatefulWidget::render(list, list_area, buf, list_state);

    if the_list.the_bar_comes() {
        // **ratatui writes the offset of the list while it draws it**,
        // therefore the bar comes after the list and it needs no second
        // measurement of the place of the user.
        //
        // **The bar holds the line of the cursor and not the offset of the
        // panel** (T-256): the track counts the lines of the list, therefore
        // the thumb stands at the top of it at the first line of the list and
        // at the foot of it at the last one.
        //
        // **The bar of a list names no key** (T-255): the footer of every view
        // of a list says `j/k: move` already.
        let mut state = ScrollbarState::new(the_list.the_track).position(
            crate::logic::the_scroll_of_a_list::the_place_of_the_bar(
                list_state.selected(),
                list_state.offset(),
                the_list.the_track,
            ),
        );

        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .render(bar_area, buf, &mut state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The size of the panel of the measurement of T-256: the area of a view of
    /// a screen of 160 columns, and 18 rows of lines under the header of it.
    const WIDTH: u16 = 160;
    const ROWS: u16 = 18;

    /// Draws a list of `lines` lines with the cursor at `selected`, and gives
    /// the rows of the buffer that hold the thumb of the bar.
    ///
    /// The row 0 of the buffer is the header of the block, therefore the first
    /// line of the list stands at the row 1.
    fn the_rows_of_the_thumb(lines: usize, selected: Option<usize>) -> Vec<u16> {
        let area = Rect::new(0, 0, WIDTH, ROWS + 1);
        let mut buf = Buffer::empty(area);

        let the_lines: Vec<String> = (0..lines).map(|i| format!("Letter {}", i + 1)).collect();

        let mut state = ListState::default();
        state.select(selected);

        render_the_list(
            area,
            &mut buf,
            &Colors::default(),
            "Episodes",
            &the_lines,
            &mut state,
        );

        (0..area.height)
            .filter(|row| buf[(WIDTH - 1, *row)].symbol() == "█")
            .collect()
    }

    /// The bar of the scroll of a list, drawn into a buffer. See T-255.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn a_list_that_is_longer_than_its_panel_draws_a_bar() {
        // The list of the measurement of T-256: 57 episodes in 18 rows.
        assert!(!the_rows_of_the_thumb(57, Some(0)).is_empty());

        // The control: a list that holds every line of it in the panel draws no
        // character of a bar at all.
        assert!(the_rows_of_the_thumb(18, Some(0)).is_empty());
        assert!(the_rows_of_the_thumb(2, Some(0)).is_empty());
    }

    /// The thumb of the bar of a list holds the line of the cursor. See T-256.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_thumb_of_the_bar_moves_with_the_cursor_inside_the_panel() {
        // **The fault of T-256, of the real program v0.8.84 inside tmux**: the
        // 17 presses of the key `j` of the view of the episodes of "Letters of
        // Two Brides" took the cursor from the line 1 to the line 18 — the
        // whole of the panel — and the thumb held the rows 4 to 9 of the screen
        // both times. The offset of the list stays at 0 while the cursor stands
        // inside the rows of the panel.
        let the_first_line = the_rows_of_the_thumb(57, Some(0));
        let the_last_line_of_the_panel = the_rows_of_the_thumb(57, Some(17));

        assert_ne!(
            the_first_line, the_last_line_of_the_panel,
            "the cursor crossed the whole panel and the bar said nothing"
        );

        // The thumb of the first line stands at the top of the track, and the
        // thumb of the line 18 stands below it.
        assert_eq!(the_first_line.first(), Some(&1));
        assert!(the_last_line_of_the_panel.first() > the_first_line.first());

        // The last line of the list takes the thumb to the foot of the track.
        let the_last_line = the_rows_of_the_thumb(57, Some(56));
        assert_eq!(the_last_line.last(), Some(&ROWS));

        // Every step of the cursor of a list of few lines moves the thumb.
        let one_more = the_rows_of_the_thumb(20, Some(0));
        let two_more = the_rows_of_the_thumb(20, Some(10));
        assert_ne!(one_more, two_more);

        // A list with no cursor keeps the offset of the panel: the thumb of it
        // stands at the top of the track.
        assert_eq!(the_rows_of_the_thumb(57, None).first(), Some(&1));
    }

    /// Gives the row of the header of a list of a width, with the border of it
    /// taken away.
    fn the_header_of_the_list(title: &str, width: u16) -> String {
        let area = Rect::new(0, 0, width, 4);
        let mut buf = Buffer::empty(area);

        let the_lines: Vec<String> = (0..3).map(|i| format!("Letter {}", i + 1)).collect();

        let mut state = ListState::default();
        state.select(None);

        render_the_list(
            area,
            &mut buf,
            &Colors::default(),
            title,
            &the_lines,
            &mut state,
        );

        (0..width)
            .map(|column| buf[(column, 0)].symbol().to_string())
            .collect::<String>()
            .replace('─', "")
    }

    /// The title of a list keeps its start at every width. See T-304.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn a_title_that_is_longer_than_the_screen_keeps_its_start() {
        // **The fault of T-304, of the real program v0.8.132 inside tmux**: a
        // search of "hours" in a terminal of 40 columns gave the title
        // "Search result [2 items, with the books of Many Hours Author]" of 60
        // characters, and the header of the view said
        // `he books of Many Hours Author]` and ten characters of the border.
        // ratatui draws a centered title that does not stand **right-aligned**
        // in an area that it cut, therefore the title lost its start and its
        // end together.
        let long = "Search result [2 items, with the books of Many Hours Author]";
        assert_eq!(long.chars().count(), 60);

        for width in [39u16, 40, 41, 59] {
            let header = the_header_of_the_list(long, width);

            assert!(
                header.starts_with("Search result ["),
                "the title lost its start at {width} columns: {header}"
            );
            assert!(
                header.ends_with('…'),
                "the title said nothing of the end that went away at {width} columns: {header}"
            );
            assert!(
                header.chars().count() <= usize::from(width),
                "the title stood outside the screen at {width} columns: {header}"
            );
        }

        // The control: a title that stands loses nothing at all, and the border
        // of the block then holds every other column of the row.
        assert_eq!(the_header_of_the_list(long, 60), long);
        assert_eq!(the_header_of_the_list("Episodes", 40), "Episodes");

        // A width that holds no title at all gives no panic.
        assert_eq!(the_header_of_the_list(long, 1), "…");
    }

    /// Draws a list of three lines with the colours of a user, and gives the
    /// buffer of it. The row 0 is the header of the block, and the rows 1, 2,
    /// and 3 are the lines of the list.
    fn the_buffer_of_the_colours(colors: &Colors) -> Buffer {
        let area = Rect::new(0, 0, WIDTH, 4);
        let mut buf = Buffer::empty(area);

        let the_lines: Vec<String> = (0..3).map(|i| format!("Letter {}", i + 1)).collect();

        let mut state = ListState::default();
        state.select(None);

        render_the_list(area, &mut buf, colors, "Episodes", &the_lines, &mut state);

        buf
    }

    /// The lines of a list take the colours that the program holds, and the
    /// render opens no file at all. See T-257 and T-204.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn a_line_of_a_list_takes_the_colours_that_the_program_holds() {
        // **The fault of T-257, of the real program v0.8.85 inside tmux**:
        // `alternate_colors` called `load_config()` one time for each line of
        // each frame, and `strace -f -e trace=openat` of the Library view of
        // the library `ManyPods` counted **14061 opens of `config.toml` in ten
        // seconds**. That function took no colour of its caller: it read the
        // disk, therefore the colours of this test reached no line of the list.
        let colors = Colors {
            list_background_color: vec![11, 12, 13],
            list_background_color_alt_row: vec![21, 22, 23],
            ..Colors::default()
        };

        let buf = the_buffer_of_the_colours(&colors);

        assert_eq!(buf[(0, 1)].style().bg, Some(Color::Rgb(11, 12, 13)));
        assert_eq!(buf[(0, 2)].style().bg, Some(Color::Rgb(21, 22, 23)));
        assert_eq!(buf[(0, 3)].style().bg, Some(Color::Rgb(11, 12, 13)));

        // The colours of the whole list come of the same values: the block of
        // the header takes the background of the list too.
        assert_eq!(
            the_colour_of_a_line(&colors, 0),
            Color::Rgb(11, 12, 13),
            "the even lines take the first colour of the user"
        );
        assert_eq!(the_colour_of_a_line(&colors, 1), Color::Rgb(21, 22, 23));
    }

    /// A colour of the configuration that holds no three numbers draws a frame,
    /// and it stops the program of the user no more. See T-257.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn a_colour_of_fewer_than_three_numbers_draws_a_frame() {
        // **The fault of T-257, of the real program v0.8.85 inside tmux**: the
        // line `list_background_color = [50, 50]` of `config.toml` of the
        // sandbox gave the log of the program
        // `[panic] panicked at src/ui/tui.rs:3000:73: index out of bounds: the
        // len is 2 but the index is 2`, and the terminal of the user went away
        // before the first frame with no word on the screen.
        let of_two_numbers = Colors {
            list_background_color: vec![50, 50],
            list_background_color_alt_row: vec![60],
            list_selected_background_color: vec![],
            line_header_color: vec![70, 71],
            header_background_color: vec![],
            ..Colors::default()
        };

        let buf = the_buffer_of_the_colours(&of_two_numbers);

        // A number that the file does not hold takes the last number of the
        // list.
        assert_eq!(buf[(0, 1)].style().bg, Some(Color::Rgb(50, 50, 50)));
        assert_eq!(buf[(0, 2)].style().bg, Some(Color::Rgb(60, 60, 60)));

        // **A list of no number at all is the colour of the terminal of the
        // user** (T-317). The background of the title of a block is the
        // background of the terminal, and the row of the cursor holds the
        // accent of the program.
        assert_eq!(of_two_numbers.header_background(), Color::Reset);
        assert_eq!(
            of_two_numbers.list_selected_background(),
            crate::ui::theme::THE_ACCENT
        );
    }
}

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

/// Draws the list of a view, and the bar of the scroll of it.
///
/// `title` stands in the header of the block, `items` are the lines of the
/// list, and `list_state` holds the cursor of the user and the offset of the
/// panel. ratatui writes that offset while it draws the list.
pub fn render_the_list(
    area: Rect,
    buf: &mut Buffer,
    colors: &Colors,
    title: &str,
    items: Vec<ListItem<'_>>,
    list_state: &mut ListState,
) {
    let bg_color_header = &colors.header_background_color;
    let fg_color_header = &colors.line_header_color;
    let bg_color_block = &colors.list_background_color;
    let bg_selected = &colors.list_selected_background_color;
    let fg_selected = &colors.list_selected_foreground_color;

    let selected_style: Style = Style::new()
        .bg(Color::Rgb(bg_selected[0], bg_selected[1], bg_selected[2]))
        .fg(Color::Rgb(fg_selected[0], fg_selected[1], fg_selected[2]))
        .add_modifier(Modifier::BOLD);

    let header_style: Style = Style::new()
        .fg(Color::Rgb(
            fg_color_header[0],
            fg_color_header[1],
            fg_color_header[2],
        ))
        .bg(Color::Rgb(
            bg_color_header[0],
            bg_color_header[1],
            bg_color_header[2],
        ));

    let block = Block::new()
        .title(Line::raw(title.to_string()).centered())
        .borders(Borders::TOP)
        .border_style(header_style)
        .bg(Color::Rgb(
            bg_color_block[0],
            bg_color_block[1],
            bg_color_block[2],
        ));

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

        let items: Vec<ListItem> = (0..lines)
            .map(|i| ListItem::new(format!("Letter {}", i + 1)))
            .collect();

        let mut state = ListState::default();
        state.select(selected);

        render_the_list(
            area,
            &mut buf,
            &Colors::default(),
            "Episodes",
            items,
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
}

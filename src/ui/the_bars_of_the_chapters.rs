//! The two bars of the view of the chapters. See T-330.5.
//!
//! **The maintainer asked for a Chapters view of two bars on 2026-08-16**, and
//! `docs/mockups/mockup-7.md` holds every rule of it: the bar of the whole book,
//! with a mark at each boundary of a chapter, and the bar of the chapter of the
//! cursor under it.
//!
//! The cells of the two bars are a pure function of `crate::logic::chapters`.
//! **The render stands in a module of its own** (the shape of T-256): a private
//! method of `App` reaches no test, therefore a render that draws nothing at all
//! would stand on the measurement of tmux alone.

use crate::logic::chapters::ABarOfTheView;
use crate::ui::theme;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

/// Draws the two bars, one on each row of the area, and it gives the cells of
/// the bar of the book back.
///
/// **The cells that played take the accent of the program**, as the row 3 of the
/// band of the player does (T-322). The marks of the boundaries stand inside the
/// cells of the bar, therefore the two parts of it keep the columns that
/// [`crate::logic::chapters::the_bar_of_the_book`] gave them.
///
/// **The row keeps one space at its left**, as the rows of the band do.
///
/// **The answer is the area of the cells of the first bar** (T-333), with no
/// column of the name and no column of the percent: a click of one of those
/// cells moves the media to the place that the cell holds, therefore the
/// arithmetic of the render and the arithmetic of the click stay in one place.
/// A frame that draws no bar at all gives `Rect::default()`.
pub fn render(area: Rect, buf: &mut Buffer, the_bars: &[ABarOfTheView; 2], the_name: u16) -> Rect {
    if area.width < 3 || area.height == 0 {
        return Rect::default();
    }

    for (number, bar) in the_bars.iter().enumerate() {
        let row = area.y + number as u16;

        if row >= area.bottom() {
            break;
        }

        Paragraph::new(the_line_of_a_bar(bar, the_name))
            .render(Rect::new(area.x + 1, row, area.width - 1, 1), buf);
    }

    the_area_of_the_bar_of_the_book(area, &the_bars[0], the_name)
}

/// The cells of the bar of the book on the screen. See T-333.
///
/// The line of a bar holds the name, the cells, and the percent, in that
/// sequence, and it starts one column inside the area:
/// [`the_line_of_a_bar`] writes it, and this function reads the same numbers.
pub fn the_area_of_the_bar_of_the_book(area: Rect, bar: &ABarOfTheView, the_name: u16) -> Rect {
    let x = area.x.saturating_add(1).saturating_add(the_name);
    let cells = u16::try_from(bar.the_cells.chars().count()).unwrap_or(u16::MAX);
    // A bar whose cells stand outside the area holds no cell of the screen.
    let width = cells.min(area.right().saturating_sub(x.min(area.right())));

    if width == 0 {
        return Rect::default();
    }

    Rect::new(x, area.y, width, 1)
}

/// The spans of one bar: the name, the cells that played, the cells that stay,
/// and the percent.
///
/// **A bar of a chapter that the media does not have says no percent at all**
/// (T-91): the columns of the percent then hold spaces, therefore the two bars
/// keep the same shape.
pub fn the_line_of_a_bar(bar: &ABarOfTheView, the_name: u16) -> Line<'static> {
    let cells: Vec<char> = bar.the_cells.chars().collect();
    let the_percent = match bar.the_percent {
        Some(percent) => format!("{percent:>4}%"),
        None => " ".repeat(usize::from(
            crate::logic::chapters::the_columns_of_the_percent(),
        )),
    };

    Line::from(vec![
        Span::styled(
            format!("{:<width$}", bar.the_name, width = usize::from(the_name)),
            theme::a_quiet_text(),
        ),
        Span::styled(
            cells
                .iter()
                .take(bar.the_cells_that_played)
                .collect::<String>(),
            Style::default().fg(theme::THE_ACCENT),
        ),
        Span::styled(
            cells
                .iter()
                .skip(bar.the_cells_that_played)
                .collect::<String>(),
            theme::a_quiet_text(),
        ),
        Span::styled(the_percent, theme::a_quiet_text()),
    ])
}

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

/// Draws the two bars, one on each row of the area.
///
/// **The cells that played take the accent of the program**, as the row 3 of the
/// band of the player does (T-322). The marks of the boundaries stand inside the
/// cells of the bar, therefore the two parts of it keep the columns that
/// [`crate::logic::chapters::the_bar_of_the_book`] gave them.
///
/// **The row keeps one space at its left**, as the rows of the band do.
pub fn render(area: Rect, buf: &mut Buffer, the_bars: &[ABarOfTheView; 2], the_name: u16) {
    if area.width < 3 || area.height == 0 {
        return;
    }

    for (number, bar) in the_bars.iter().enumerate() {
        let row = area.y + number as u16;

        if row >= area.bottom() {
            break;
        }

        Paragraph::new(the_line_of_a_bar(bar, the_name))
            .render(Rect::new(area.x + 1, row, area.width - 1, 1), buf);
    }
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

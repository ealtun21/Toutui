//! The reports of the mouse, and the place of the screen that each one names.
//!
//! **The mouse is the stage 3 of the road of the panels** (T-316), and it comes
//! after the frame of T-320 because a click names a panel: a click that no
//! frame divides can name nothing at all.
//!
//! **A terminal sends no report of the mouse before the program asks for it.**
//! `crossterm::event::EnableMouseCapture` is that request, and
//! `crate::utils::the_terminal_of_the_program` sends it with the raw mode and
//! the alternate screen. **A capture of the mouse takes the selection of the
//! text away from the user**, because the terminal then gives the report to the
//! program and not to itself: the key `Ctrl+o` of this program therefore stops
//! the capture, and the modifier `Shift` of most terminals gives the selection
//! while the capture stands.
//!
//! **This module holds the arithmetic alone, and it opens no terminal.** The
//! render of a frame writes the areas of the lists into
//! [`TheAreasOfTheMouse`], and a report of the mouse then reads them: a
//! function of this shape takes a test with no terminal, with no server, and
//! with no `App` at all.
//!
//! **The areas are the areas of the last frame.** A report of the mouse comes
//! of the terminal after that frame, therefore the user clicked on the screen
//! that those areas describe.

use ratatui::layout::{Position, Rect};

/// The words of the key `Ctrl+o` that starts the mouse again. See T-316.
///
/// **The sentence names the key of the road back** (T-170), and that key is the
/// same key: a user who wants the selection of the text of their terminal
/// presses it again.
pub const THE_MOUSE_STANDS: &str =
    "The program reads the mouse. Press Ctrl+o to stop it and to select the text of your terminal.";

/// The words of the key `Ctrl+o` that stops the mouse. See T-316.
pub const THE_MOUSE_STOPPED: &str =
    "The program does not read the mouse. You can select the text of your terminal again. \
     Press Ctrl+o for the mouse.";

/// The areas of the last frame that a report of the mouse can name.
///
/// A value of `Rect::default()` holds no cell at all, therefore a panel that
/// the frame did not draw takes no click: that is the value of the start, and
/// it is the value of every panel of a view that the road of the panels did not
/// reach yet.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TheAreasOfTheMouse {
    /// The whole block of the panel 1 of the views, with its border.
    pub the_panel_of_the_views: Rect,
    /// The rows of the lines of the panel 1, inside its border.
    pub the_lines_of_the_views: Rect,
    /// The line of the panel 1 that stands on the first row of it.
    pub the_offset_of_the_views: usize,
    /// The number of the lines of the panel 1.
    pub the_views: usize,
    /// The whole block of the panel 2 of the sequence, with its border. See
    /// T-318.
    pub the_panel_of_the_sequence: Rect,
    /// The rows of the lines of the panel 2, inside its border.
    pub the_lines_of_the_sequence: Rect,
    /// The number of the lines of the panel 2.
    pub the_sequences: usize,
    /// The whole block of the panel 3 of the filter, with its border. See
    /// T-318.
    pub the_panel_of_the_filter: Rect,
    /// The rows of the lines of the panel 3, inside its border.
    pub the_lines_of_the_filter: Rect,
    /// The number of the lines of the panel 3.
    pub the_filters: usize,
    /// The whole block of the list of the view, with its border.
    pub the_panel_of_the_list: Rect,
    /// The rows of the lines of the list of the view, inside its border.
    pub the_lines_of_the_list: Rect,
    /// The line of the list that stands on the first row of it.
    pub the_offset_of_the_list: usize,
    /// The number of the lines of the list of the view.
    pub the_lines: usize,
    /// The row of the header of the table of the panel 4. See T-321.
    ///
    /// A view that draws no table gives `Rect::default()` here, and that value
    /// holds no cell of the screen at all.
    pub the_header_of_the_list: Rect,
    /// The whole block of the panel 5 of the cover, with its border. See
    /// T-319.
    ///
    /// A view that draws no cover, and a terminal that is too narrow for one,
    /// gives `Rect::default()` here.
    pub the_panel_of_the_cover: Rect,
}

/// The place of the screen that a report of the mouse names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheTarget {
    /// No panel of the frame holds this point.
    Nothing,
    /// The panel 1 of the views. `the_line` is the line under the pointer, and
    /// `None` says that the point stands on the border of the panel or under
    /// its last line.
    ThePanelOfTheViews { the_line: Option<usize> },
    /// The panel 2 of the sequence. See T-318.
    ThePanelOfTheSequence { the_line: Option<usize> },
    /// The panel 3 of the filter. See T-318.
    ThePanelOfTheFilter { the_line: Option<usize> },
    /// The list of the view, which is the panel 4 of the frame of the panels.
    TheListOfTheView { the_line: Option<usize> },
    /// The row of the header of the table of the panel 4. See T-321.
    TheHeaderOfTheList,
    /// The panel 5 of the cover, and the words of the media inside it. See
    /// T-319.
    ///
    /// **A click of that panel gives it the focus and nothing else**: the panel
    /// holds one media, which is the media of the cursor of the list already,
    /// therefore a click of it moves the cursor of no list.
    ThePanelOfTheCover,
}

/// The line of a list that stands on a row of the screen, and `None` for a row
/// that holds no line of it.
///
/// **The offset is the line of the list that stands on the first row of the
/// area**, and ratatui writes it while it draws the list. A list of ten lines
/// in a panel of five rows therefore gives the line 7 for the third row of its
/// area when the offset is 5.
///
/// **A row under the last line of a list holds no line at all**: a list of
/// three lines in a panel of twenty rows leaves seventeen rows with no line,
/// and a click on one of them must move the cursor of the user nowhere.
pub fn the_line_of_a_row(area: Rect, offset: usize, lines: usize, row: u16) -> Option<usize> {
    if row < area.y || row >= area.y.saturating_add(area.height) {
        return None;
    }

    let the_line = offset.checked_add((row - area.y) as usize)?;

    if the_line >= lines {
        return None;
    }

    Some(the_line)
}

/// The place of the screen that the point of a report of the mouse names.
///
/// **The panel 1 comes first**, because the stack stands at the left of the
/// list and the two of them hold no cell together. `the_stack_stands` is
/// `App::the_stack_of_the_panels_stands`: a terminal under 120 columns draws no
/// stack, and a click of those columns therefore belongs to the list.
///
/// **A click on the border of a panel names that panel**, with no line: the
/// user asked for the focus of it, and the border is a part of it that the user
/// can see. That is the rule of the title of the panel too.
pub fn the_target_of_a_point(
    areas: &TheAreasOfTheMouse,
    the_stack_stands: bool,
    column: u16,
    row: u16,
) -> TheTarget {
    let point = Position::new(column, row);

    if the_stack_stands && areas.the_panel_of_the_views.contains(point) {
        return TheTarget::ThePanelOfTheViews {
            the_line: the_line_of_a_row(
                areas.the_lines_of_the_views,
                areas.the_offset_of_the_views,
                areas.the_views,
                row,
            )
            .filter(|_| areas.the_lines_of_the_views.contains(point)),
        };
    }

    // **The panels 2 and 3 stand under the panel 1, in the same column**
    // (T-318). A panel that the frame did not draw holds `Rect::default()`,
    // which holds no cell of the screen at all, therefore a stack that lost
    // the panel 3 gives its clicks to no panel.
    if the_stack_stands && areas.the_panel_of_the_sequence.contains(point) {
        return TheTarget::ThePanelOfTheSequence {
            the_line: the_line_of_a_row(
                areas.the_lines_of_the_sequence,
                0,
                areas.the_sequences,
                row,
            )
            .filter(|_| areas.the_lines_of_the_sequence.contains(point)),
        };
    }

    if the_stack_stands && areas.the_panel_of_the_filter.contains(point) {
        return TheTarget::ThePanelOfTheFilter {
            the_line: the_line_of_a_row(areas.the_lines_of_the_filter, 0, areas.the_filters, row)
                .filter(|_| areas.the_lines_of_the_filter.contains(point)),
        };
    }

    // **The row of the header stands inside the panel of the list**, therefore
    // it comes before it: a click of that row asks for the sequence and the
    // filter of the view, and it moves the cursor of no line. See T-321.
    if areas.the_header_of_the_list.contains(point) {
        return TheTarget::TheHeaderOfTheList;
    }

    if areas.the_panel_of_the_list.contains(point) {
        return TheTarget::TheListOfTheView {
            the_line: the_line_of_a_row(
                areas.the_lines_of_the_list,
                areas.the_offset_of_the_list,
                areas.the_lines,
                row,
            )
            .filter(|_| areas.the_lines_of_the_list.contains(point)),
        };
    }

    // **The panel 5 comes after the panel 4**, because the two of them hold no
    // cell together: `cover::split_for_covers` leaves one column between them.
    // See T-319.
    //
    // **The focus of a panel belongs to the shape of three columns alone**
    // (T-320), and `the_stack_stands` is that shape: a screen of two columns
    // draws the panel 5 and it takes no key of a panel at all, therefore a
    // click that gave it the focus would show a border of a focus that no key
    // of the user can use (T-79).
    if the_stack_stands && areas.the_panel_of_the_cover.contains(point) {
        return TheTarget::ThePanelOfTheCover;
    }

    TheTarget::Nothing
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The offset of a list moves the line of a row of the screen.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_line_of_a_row_reads_the_offset_of_the_list() {
        let area = Rect::new(10, 4, 30, 5);

        // The first row of the area holds the line of the offset.
        assert_eq!(the_line_of_a_row(area, 0, 20, 4), Some(0));
        assert_eq!(the_line_of_a_row(area, 5, 20, 4), Some(5));

        // A row of the middle counts the rows from the first one.
        assert_eq!(the_line_of_a_row(area, 5, 20, 6), Some(7));

        // The last row of the area. The area holds five rows of the row 4,
        // therefore the row 8 is the last of them.
        assert_eq!(the_line_of_a_row(area, 5, 20, 8), Some(9));

        // A row above the area and a row under it hold no line.
        assert_eq!(the_line_of_a_row(area, 5, 20, 3), None);
        assert_eq!(the_line_of_a_row(area, 5, 20, 9), None);

        // **A row under the last line of a short list holds no line**: the
        // list holds three lines and the area holds five rows.
        assert_eq!(the_line_of_a_row(area, 0, 3, 6), Some(2));
        assert_eq!(the_line_of_a_row(area, 0, 3, 7), None);
        assert_eq!(the_line_of_a_row(area, 0, 0, 4), None);

        // An area of no row at all holds no line. That is the area of the
        // start of the program, before the first frame.
        assert_eq!(the_line_of_a_row(Rect::default(), 0, 20, 0), None);
    }

    /// The areas of the last frame give the panel of a point, and the stack
    /// takes no click of a terminal that draws no stack.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_target_of_a_point_names_the_panel_of_that_point() {
        // The frame of a screen of 160 columns: the stack of 34 columns at the
        // left, and the list of the view after it.
        let areas = TheAreasOfTheMouse {
            the_panel_of_the_views: Rect::new(0, 2, 34, 20),
            the_lines_of_the_views: Rect::new(1, 3, 32, 18),
            the_offset_of_the_views: 0,
            the_views: 14,
            the_panel_of_the_sequence: Rect::default(),
            the_lines_of_the_sequence: Rect::default(),
            the_sequences: 0,
            the_panel_of_the_filter: Rect::default(),
            the_lines_of_the_filter: Rect::default(),
            the_filters: 0,
            the_panel_of_the_list: Rect::new(34, 2, 70, 20),
            the_lines_of_the_list: Rect::new(35, 3, 66, 18),
            the_offset_of_the_list: 0,
            the_lines: 500,
            the_header_of_the_list: Rect::default(),
            the_panel_of_the_cover: Rect::default(),
        };

        // A click on a line of the panel 1.
        assert_eq!(
            the_target_of_a_point(&areas, true, 10, 5),
            TheTarget::ThePanelOfTheViews { the_line: Some(2) }
        );

        // **A click on the border of a panel names that panel and no line of
        // it**: the user asked for the focus.
        assert_eq!(
            the_target_of_a_point(&areas, true, 0, 5),
            TheTarget::ThePanelOfTheViews { the_line: None }
        );

        // A click on a line of the list of the view.
        assert_eq!(
            the_target_of_a_point(&areas, true, 60, 3),
            TheTarget::TheListOfTheView { the_line: Some(0) }
        );
        assert_eq!(
            the_target_of_a_point(&areas, true, 60, 10),
            TheTarget::TheListOfTheView { the_line: Some(7) }
        );

        // **A terminal that draws no stack gives every click to the list.**
        // The panel 1 of such a screen stands in no cell of it, and the areas
        // of that frame hold `Rect::default()` for it.
        let of_one_column = TheAreasOfTheMouse {
            the_panel_of_the_views: Rect::default(),
            the_lines_of_the_views: Rect::default(),
            ..areas
        };
        assert_eq!(
            the_target_of_a_point(&of_one_column, false, 10, 5),
            TheTarget::Nothing,
            "the point stands at the left of the list of that frame"
        );

        // **The row of the header of the table takes the click before the
        // lines of the list** (T-321): it stands inside the panel of the list,
        // and a click of it asks for the sequence and the filter of the view.
        let of_a_table = TheAreasOfTheMouse {
            the_lines_of_the_list: Rect::new(35, 4, 66, 17),
            the_header_of_the_list: Rect::new(35, 3, 68, 1),
            ..areas
        };
        assert_eq!(
            the_target_of_a_point(&of_a_table, true, 60, 3),
            TheTarget::TheHeaderOfTheList
        );
        assert_eq!(
            the_target_of_a_point(&of_a_table, true, 60, 4),
            TheTarget::TheListOfTheView { the_line: Some(0) }
        );

        // **The panels 2 and 3 stand under the panel 1 of the same column**
        // (T-318): the stack of 34 columns holds the views of the row 2 to the
        // row 21, the sequence of the row 22 to the row 31, and the filter of
        // the row 32 to the row 37.
        let of_the_stack = TheAreasOfTheMouse {
            the_panel_of_the_sequence: Rect::new(0, 22, 34, 10),
            the_lines_of_the_sequence: Rect::new(1, 23, 32, 8),
            the_sequences: 8,
            the_panel_of_the_filter: Rect::new(0, 32, 34, 6),
            the_lines_of_the_filter: Rect::new(1, 33, 32, 4),
            the_filters: 4,
            ..areas
        };

        assert_eq!(
            the_target_of_a_point(&of_the_stack, true, 10, 23),
            TheTarget::ThePanelOfTheSequence { the_line: Some(0) }
        );
        assert_eq!(
            the_target_of_a_point(&of_the_stack, true, 10, 30),
            TheTarget::ThePanelOfTheSequence { the_line: Some(7) }
        );
        // A click on the border of the panel 2 names that panel and no line.
        assert_eq!(
            the_target_of_a_point(&of_the_stack, true, 0, 22),
            TheTarget::ThePanelOfTheSequence { the_line: None }
        );
        assert_eq!(
            the_target_of_a_point(&of_the_stack, true, 10, 33),
            TheTarget::ThePanelOfTheFilter { the_line: Some(0) }
        );
        assert_eq!(
            the_target_of_a_point(&of_the_stack, true, 10, 36),
            TheTarget::ThePanelOfTheFilter { the_line: Some(3) }
        );

        // **A stack that lost the panel 3 takes no click of it** (T-79): the
        // areas of that frame hold `Rect::default()`, which holds no cell of
        // the screen at all.
        let of_no_filter = TheAreasOfTheMouse {
            the_panel_of_the_filter: Rect::default(),
            the_lines_of_the_filter: Rect::default(),
            ..of_the_stack
        };
        assert_eq!(
            the_target_of_a_point(&of_no_filter, true, 10, 33),
            TheTarget::Nothing
        );

        // A point that no panel holds.
        assert_eq!(
            the_target_of_a_point(&areas, true, 150, 5),
            TheTarget::Nothing
        );
        assert_eq!(
            the_target_of_a_point(&areas, true, 10, 0),
            TheTarget::Nothing
        );
    }
}

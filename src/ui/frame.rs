//! The frame of the panels, and the focus of a panel. See T-320.
//!
//! **The maintainer chose the mockup 1, the panels, on 2026-08-16**, therefore
//! `docs/mockups/mockup-1.txt` is the design of the program now. This module
//! holds the stage 2 of that road: the frame of the columns, the shape of the
//! frame for a terminal that is not wide, and the focus of a panel.
//!
//! **The screen of today, of the real program v0.8.145 inside tmux**: one
//! column of a list, with the panel of the covers at the right of it when the
//! screen holds 84 columns or more. The design holds three columns — the stack
//! of the panels 1, 2, and 3 at the left, the panel 4 of the list in the
//! middle, and the panels 5 and 6 at the right.
//!
//! ## The three shapes of the frame
//!
//! **The design needs about 120 columns for its three columns, and this fork
//! measures 40 columns as its narrowest** (T-301). The frame therefore holds
//! three shapes, and each of them is a measurement of its own:
//!
//! - **Three columns**, at [`THE_WIDTH_OF_THREE_COLUMNS`] and up: the stack of
//!   the panels stands at the left, and the work of the view takes what stays.
//! - **Two columns**, from [`THE_WIDTH_OF_TWO_COLUMNS`] to one column under
//!   [`THE_WIDTH_OF_THREE_COLUMNS`]: the stack goes away, and the work of the
//!   view takes the whole width. The panel of the covers of T-23 stands at the
//!   right of it, because that panel needs 84 columns already
//!   (`cover::MIN_WIDTH_FOR_COVER`).
//! - **One column**, under [`THE_WIDTH_OF_TWO_COLUMNS`]: the screen is the
//!   screen of today, and the covers go away with it.
//!
//! **A shape that no measurement reached is not a shape of this program**,
//! therefore the item T-320 of `docs/TAKEOVER-BACKLOG.md` holds the screen of
//! the real program at each of the three widths.
//!
//! ## The focus
//!
//! **The focus is the shape of a border and not a colour alone** (the section
//! (c) of `docs/mockups/mockup-1.md`): the panel that holds the focus takes a
//! heavy border `═║`, and every other panel takes a light border `─│`. A
//! terminal of a theme of a low contrast still says where the focus is.
//!
//! **The stages until this one draw six of the seven panels**, therefore
//! [`ThePanel`] holds six values: the views (T-320), the sequence and the
//! filter (T-318), the list (T-320), the cover (T-319), and the gallery
//! (T-327). The panel 7 of the player comes with T-322, and it takes no focus:
//! every key of the player works in every view of this program already. **A key
//! of a panel that no stage drew is a key that does nothing**, and that is a
//! fault of its own (T-79), therefore the digit `7` is not a key of this
//! program yet.

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders},
};

/// The smallest width of the screen that holds the three columns of the design.
///
/// The design of `docs/mockups/mockup-1.txt` gives the stack 34 columns, the
/// panel 4 of the list 66, and the panels of the covers 60. The stack keeps its
/// 34 columns at every width, therefore this number is the width where the work
/// of the view still holds the 84 columns of a panel of a cover
/// (`cover::MIN_WIDTH_FOR_COVER`) with the stack beside it: 34 + 84 = 118, and
/// 120 gives the two columns between them.
pub const THE_WIDTH_OF_THREE_COLUMNS: u16 = 120;

/// The smallest width of the screen that holds two columns.
///
/// This is `cover::MIN_WIDTH_FOR_COVER`, and the reason is the same: a terminal
/// under it has no width for a text and for a cover at the same time.
pub const THE_WIDTH_OF_TWO_COLUMNS: u16 = 84;

/// The width of the stack of the panels 1, 2, and 3, of the design.
pub const THE_WIDTH_OF_THE_STACK: u16 = 34;

/// The shape of the frame of a screen of a width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheShape {
    /// The stack of the panels, the work of the view, and the covers.
    ThreeColumns,
    /// The work of the view and the covers. The stack goes away.
    TwoColumns,
    /// The work of the view alone. This is the screen of today.
    OneColumn,
}

/// Gives the shape of the frame of a screen of this width.
pub fn the_shape_of(width: u16) -> TheShape {
    if width >= THE_WIDTH_OF_THREE_COLUMNS {
        return TheShape::ThreeColumns;
    }

    if width >= THE_WIDTH_OF_TWO_COLUMNS {
        return TheShape::TwoColumns;
    }

    TheShape::OneColumn
}

/// Divides the work of a view into the stack of the panels and the work that
/// stays.
///
/// The stack is `None` for every shape but [`TheShape::ThreeColumns`].
///
/// **The user can hide the stack** (T-323), and `the_user_hid_the_stack` is
/// that mode: the design of `docs/mockups/mockup-1.md`, in its section (f),
/// names the cost of a screen that is always full, and the answer of it is a
/// key that gives the 34 columns of the stack to the panel 4. **That mode is
/// not the mode of the start**, and the panels 4 and 5 keep every key that they
/// hold, because those two panels stand on the screen still.
///
/// **The area of the work goes to `cover::split_for_covers` after this
/// function**, and that function reads the width that it gets: a stack of 34
/// columns at the left therefore takes 34 columns away from the arithmetic of
/// the panel of the covers, and the covers of a screen of 120 columns stand on
/// the 86 columns that stay.
pub fn the_stack_and_the_work(
    area: Rect,
    shape: TheShape,
    the_user_hid_the_stack: bool,
) -> (Option<Rect>, Rect) {
    if the_user_hid_the_stack || shape != TheShape::ThreeColumns {
        return (None, area);
    }

    let [stack, work] = Layout::horizontal([
        Constraint::Length(THE_WIDTH_OF_THE_STACK),
        Constraint::Fill(1),
    ])
    .areas(area);

    (Some(stack), work)
}

/// A panel of the frame that holds the focus of the user.
///
/// **Six of the seven panels of the design stand today**, and the band of the
/// player takes no focus at all. See the head of this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThePanel {
    /// The panel 1: the list of the views of the program, with the key of each.
    TheViews,
    /// The panel 2: the sequence of the library. See T-318.
    TheSequence,
    /// The panel 3: the filter of the library. See T-318.
    TheFilter,
    /// The panel 4: the list of the view. This is the panel of the start.
    #[default]
    TheList,
    /// The panel 5: the cover of the media, and the words of it. See T-319.
    TheCover,
    /// The panel 6: the gallery of the covers of the media around the cursor.
    /// See T-327.
    TheGallery,
}

impl ThePanel {
    /// The number of the panel, which stands at the start of its title.
    pub fn the_number(self) -> u8 {
        match self {
            Self::TheViews => 1,
            Self::TheSequence => 2,
            Self::TheFilter => 3,
            Self::TheList => 4,
            Self::TheCover => 5,
            Self::TheGallery => 6,
        }
    }

    /// The panel of a digit that the user pressed, or `None` when no panel of
    /// this program holds that digit.
    pub fn of_the_digit(digit: char) -> Option<Self> {
        match digit {
            '1' => Some(Self::TheViews),
            '2' => Some(Self::TheSequence),
            '3' => Some(Self::TheFilter),
            '4' => Some(Self::TheList),
            '5' => Some(Self::TheCover),
            '6' => Some(Self::TheGallery),
            _ => None,
        }
    }

    /// The panel at the left of this one. The panel at the left of the first
    /// column is that panel itself.
    pub fn at_the_left(self) -> Self {
        match self {
            Self::TheViews | Self::TheSequence | Self::TheFilter => self,
            Self::TheList => Self::TheViews,
            Self::TheCover | Self::TheGallery => Self::TheList,
        }
    }

    /// The panel at the right of this one. The panel at the right of the last
    /// column is that panel itself.
    ///
    /// **The panel 5 of the cover stands at the right of the panel 4** (T-319),
    /// and a view that draws no cover holds no such panel: the caller reads
    /// `App::a_panel_of_the_frame_stands` before it moves the focus, therefore
    /// a key that names a panel of no cell at all does nothing (T-79).
    pub fn at_the_right(self) -> Self {
        match self {
            Self::TheViews | Self::TheSequence | Self::TheFilter => Self::TheList,
            Self::TheList => Self::TheCover,
            Self::TheCover | Self::TheGallery => self,
        }
    }

    /// The panel under this one, in the stack of the first column. See T-318.
    ///
    /// **The stack of the three panels needs a key that goes down and a key
    /// that goes up**: `Ctrl+h` and `Ctrl+l` move between the columns, and the
    /// panels 1, 2, and 3 stand in one column. The panel under the last panel
    /// of a column is that panel itself.
    pub fn below(self) -> Self {
        match self {
            Self::TheViews => Self::TheSequence,
            Self::TheSequence => Self::TheFilter,
            Self::TheFilter | Self::TheList | Self::TheGallery => self,
            // **The panel 6 of the gallery stands under the panel 5 of the
            // cover**, in the column at the right (T-327).
            Self::TheCover => Self::TheGallery,
        }
    }

    /// The panel above this one, in the stack of the first column. See T-318.
    pub fn above(self) -> Self {
        match self {
            Self::TheFilter => Self::TheSequence,
            Self::TheSequence => Self::TheViews,
            Self::TheViews | Self::TheList | Self::TheCover => self,
            Self::TheGallery => Self::TheCover,
        }
    }

    /// Tells if this panel stands in the stack of the first column.
    pub fn is_of_the_stack(self) -> bool {
        matches!(self, Self::TheViews | Self::TheSequence | Self::TheFilter)
    }
}

/// The block of a panel of the frame, with its number, its name, and the border
/// of its focus.
///
/// **The border says the focus by its shape** (the section (c) of
/// `docs/mockups/mockup-1.md`): [`BorderType::Double`] against
/// [`BorderType::Plain`]. The colour says it a second time, and a terminal that
/// gives no colour at all still shows the focus.
///
/// **The number takes the yellow of ANSI and no colour of RGB** (T-317), and
/// the border of a panel that does not hold the focus takes the modifier `DIM`
/// and no grey: a grey stays grey on a background of a light colour.
pub fn a_panel(number: u8, name: &str, it_holds_the_focus: bool) -> Block<'static> {
    let of_the_border = if it_holds_the_focus {
        Style::default()
            .fg(crate::ui::theme::THE_ACCENT)
            .add_modifier(Modifier::BOLD)
    } else {
        crate::ui::theme::a_quiet_text()
    };

    let of_the_name = if it_holds_the_focus {
        Style::default()
            .fg(crate::ui::theme::THE_ACCENT)
            .add_modifier(Modifier::BOLD)
    } else {
        crate::ui::theme::a_quiet_text()
    };

    let title = Line::from(vec![
        Span::styled(
            number.to_string(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {name} "), of_the_name),
    ]);

    Block::new()
        .borders(Borders::ALL)
        .border_type(if it_holds_the_focus {
            BorderType::Double
        } else {
            BorderType::Plain
        })
        .border_style(of_the_border)
        .title(title)
}

/// The block of the band of the player, which holds no number. See T-322.
///
/// **A number of a panel is the name of a key** (T-118): the digits 1 to 5 give
/// the focus to the panels of the stack, of the list, and of the cover, and no
/// key of this program gives the focus to the band of the player. A band that
/// said `7 Player` would therefore promise the digit `7`, which does nothing.
///
/// **The band takes no focus at all**, and the reason of it is the keys: every
/// key of the player works in every view of this program already (`Space`, `p`,
/// `u`, `P`, `U`, `O`, `I`, `o`, `i`, `t`, and `Y`), therefore a focus of the
/// band would hold no key of its own. The digit `7` of the design comes with
/// the keys of that focus, and not before them.
pub fn a_band(name: &str) -> Block<'static> {
    Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(crate::ui::theme::a_quiet_text())
        .title(Span::styled(
            format!(" {name} "),
            crate::ui::theme::a_quiet_text(),
        ))
}

/// The work of a line of the panel 1.
///
/// **The key `Tab` of this program turns the Home view and the Library view one
/// into the other** (`App::toggle_view`), therefore a line of the panel that
/// sent that key would take the user away from the view that they asked for.
/// Those two lines therefore name the view itself, and every other line names
/// the key of `src/app.rs` that opens its view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheWork {
    /// The Home view.
    TheHomeView,
    /// The Library view.
    TheLibraryView,
    /// The key of `src/app.rs` that opens the view.
    TheKey(char),
}

/// One line of the panel 1: the name of a view, and the key that opens it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AView {
    /// The name of the view, for the user.
    pub name: &'static str,
    /// The key of `src/app.rs` that opens it, for the user to read.
    pub key: &'static str,
    /// What the key `l` of this line does.
    pub work: TheWork,
}

/// The views of the panel 1, in the sequence of the design.
///
/// **The key of each line is the key of `src/app.rs` and not the key of the
/// mockup**: the mockup gave the bookmarks the key `k` and the collections the
/// key `c`, and the key `k` of this program moves a list one line up. A panel
/// that names a key that the program does not hold is the fault of T-118, and a
/// key of the panel that does nothing is the fault of T-79.
pub const THE_VIEWS: &[AView] = &[
    AView {
        name: "Home",
        key: "Tab",
        work: TheWork::TheHomeView,
    },
    AView {
        name: "Library",
        key: "Tab",
        work: TheWork::TheLibraryView,
    },
    AView {
        name: "Sequence and filter",
        key: "f",
        work: TheWork::TheKey('f'),
    },
    AView {
        name: "Authors",
        key: "a",
        work: TheWork::TheKey('a'),
    },
    AView {
        name: "Narrators",
        key: "v",
        work: TheWork::TheKey('v'),
    },
    AView {
        name: "Collections",
        key: "c",
        work: TheWork::TheKey('c'),
    },
    AView {
        name: "Queue",
        key: "q",
        work: TheWork::TheKey('q'),
    },
    AView {
        name: "Downloads",
        key: "d",
        work: TheWork::TheKey('d'),
    },
    AView {
        name: "Chapters",
        key: "C",
        work: TheWork::TheKey('C'),
    },
    AView {
        name: "Bookmarks",
        key: "V",
        work: TheWork::TheKey('V'),
    },
    AView {
        name: "Sessions",
        key: "W",
        work: TheWork::TheKey('W'),
    },
    AView {
        name: "Statistics",
        key: "T",
        work: TheWork::TheKey('T'),
    },
    AView {
        name: "Settings",
        key: "S",
        work: TheWork::TheKey('S'),
    },
    AView {
        name: "Every key",
        key: "?",
        work: TheWork::TheKey('?'),
    },
];

/// The lines of the panel 1, of the width of the inside of that panel.
///
/// The name stands at the left and the key stands at the right, with the space
/// between them. **A name that is longer than the panel loses its end and not
/// its start** (the rule of T-304), because the start of a name says which view
/// it is.
pub fn the_lines_of_the_views(width: u16) -> Vec<String> {
    let width = usize::from(width);

    THE_VIEWS
        .iter()
        .map(|view| {
            // Two columns of the width go to the sign of the cursor of ratatui.
            let of_the_line = width.saturating_sub(2);
            let of_the_key = view.key.chars().count();
            let of_the_name = of_the_line.saturating_sub(of_the_key + 1);

            let name = crate::logic::message::in_one_row(view.name, of_the_name as u16);
            let name = name.chars().count();

            format!(
                "{}{}{}",
                crate::logic::message::in_one_row(view.name, of_the_name as u16),
                " ".repeat(of_the_line.saturating_sub(name + of_the_key)),
                view.key,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The three shapes of the frame, and the width of each of them. See T-320.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_width_of_the_screen_gives_the_shape_of_the_frame() {
        // The narrowest terminal of this fork (T-301), and the screen of today.
        assert_eq!(the_shape_of(40), TheShape::OneColumn);
        assert_eq!(the_shape_of(83), TheShape::OneColumn);

        // The panel of the covers of T-23 comes at 84 columns.
        assert_eq!(the_shape_of(84), TheShape::TwoColumns);
        assert_eq!(the_shape_of(100), TheShape::TwoColumns);
        assert_eq!(the_shape_of(119), TheShape::TwoColumns);

        // The stack of the panels comes at 120 columns.
        assert_eq!(the_shape_of(120), TheShape::ThreeColumns);
        assert_eq!(the_shape_of(160), TheShape::ThreeColumns);

        // **A screen of no width gives one column**, and it stops no program:
        // `App` holds the width of the last frame, and that width is 0 before
        // the first frame of the program (the trap 214).
        assert_eq!(the_shape_of(0), TheShape::OneColumn);
    }

    /// The stack takes its columns of the left of the work of the view, and it
    /// comes for the three columns alone. See T-320.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_stack_stands_at_the_left_of_the_work_of_the_view() {
        let area = Rect::new(0, 2, 160, 30);

        let (stack, work) = the_stack_and_the_work(area, TheShape::ThreeColumns, false);
        let stack = stack.expect("the three columns hold the stack");
        assert_eq!(stack.x, 0);
        assert_eq!(stack.width, THE_WIDTH_OF_THE_STACK);
        assert_eq!(stack.height, area.height);
        assert_eq!(work.x, THE_WIDTH_OF_THE_STACK);
        assert_eq!(work.width, area.width - THE_WIDTH_OF_THE_STACK);

        // The work of the view keeps the 84 columns of a panel of a cover at the
        // smallest width of the three columns.
        let (_, work) = the_stack_and_the_work(
            Rect::new(0, 2, THE_WIDTH_OF_THREE_COLUMNS, 30),
            TheShape::ThreeColumns,
            false,
        );
        assert!(
            work.width >= THE_WIDTH_OF_TWO_COLUMNS,
            "the covers need {THE_WIDTH_OF_TWO_COLUMNS} columns and the work holds {}",
            work.width
        );

        // The two other shapes give the whole area to the work of the view.
        for shape in [TheShape::TwoColumns, TheShape::OneColumn] {
            let (stack, work) = the_stack_and_the_work(area, shape, false);
            assert_eq!(stack, None, "the shape {shape:?} holds no stack");
            assert_eq!(work, area);
        }
    }

    /// The digits and the movement of the focus. See T-320.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_focus_moves_between_the_panels_that_stand() {
        assert_eq!(ThePanel::of_the_digit('1'), Some(ThePanel::TheViews));
        assert_eq!(ThePanel::of_the_digit('2'), Some(ThePanel::TheSequence));
        assert_eq!(ThePanel::of_the_digit('3'), Some(ThePanel::TheFilter));
        assert_eq!(ThePanel::of_the_digit('4'), Some(ThePanel::TheList));
        assert_eq!(ThePanel::of_the_digit('5'), Some(ThePanel::TheCover));

        assert_eq!(ThePanel::of_the_digit('6'), Some(ThePanel::TheGallery));

        // **The band of the player holds no digit** (T-79 and T-322): a key
        // that does nothing is a fault of its own, and every key of the player
        // works in every view of this program already.
        for digit in ['7', '0', '8', '9'] {
            assert_eq!(
                ThePanel::of_the_digit(digit),
                None,
                "the digit {digit} names a panel that this program does not draw"
            );
        }

        // The panel of the start is the list of the view.
        assert_eq!(ThePanel::default(), ThePanel::TheList);

        // The movement stops at the ends, and it never gives a panel that the
        // program does not draw. **The three panels of the stack stand in one
        // column**, therefore the panel at the right of each of them is the
        // list, and the panel at the left of each of them is itself.
        assert_eq!(ThePanel::TheList.at_the_left(), ThePanel::TheViews);

        // **The panel 5 of the cover stands at the right of the panel 4**
        // (T-319), and the panel at the right of it is that panel itself.
        assert_eq!(ThePanel::TheList.at_the_right(), ThePanel::TheCover);
        assert_eq!(ThePanel::TheCover.at_the_right(), ThePanel::TheCover);
        assert_eq!(ThePanel::TheCover.at_the_left(), ThePanel::TheList);
        assert!(!ThePanel::TheCover.is_of_the_stack());

        // **The panel 6 of the gallery stands under the panel 5 of the cover**
        // (T-327), in the column at the right.
        assert_eq!(ThePanel::TheCover.below(), ThePanel::TheGallery);
        assert_eq!(ThePanel::TheCover.above(), ThePanel::TheCover);
        assert_eq!(ThePanel::TheGallery.above(), ThePanel::TheCover);
        assert_eq!(ThePanel::TheGallery.below(), ThePanel::TheGallery);
        assert_eq!(ThePanel::TheGallery.at_the_left(), ThePanel::TheList);
        assert_eq!(ThePanel::TheGallery.at_the_right(), ThePanel::TheGallery);
        assert!(!ThePanel::TheGallery.is_of_the_stack());

        for of_the_stack in [
            ThePanel::TheViews,
            ThePanel::TheSequence,
            ThePanel::TheFilter,
        ] {
            assert!(of_the_stack.is_of_the_stack());
            assert_eq!(of_the_stack.at_the_left(), of_the_stack);
            assert_eq!(of_the_stack.at_the_right(), ThePanel::TheList);
        }

        assert!(!ThePanel::TheList.is_of_the_stack());

        // **The stack of the three panels needs a key that goes down and a key
        // that goes up** (T-318), and each of them stops at the end.
        assert_eq!(ThePanel::TheViews.below(), ThePanel::TheSequence);
        assert_eq!(ThePanel::TheSequence.below(), ThePanel::TheFilter);
        assert_eq!(ThePanel::TheFilter.below(), ThePanel::TheFilter);
        assert_eq!(ThePanel::TheFilter.above(), ThePanel::TheSequence);
        assert_eq!(ThePanel::TheSequence.above(), ThePanel::TheViews);
        assert_eq!(ThePanel::TheViews.above(), ThePanel::TheViews);

        // The panel 4 stands in a column of its own, therefore it moves nowhere.
        assert_eq!(ThePanel::TheList.below(), ThePanel::TheList);
        assert_eq!(ThePanel::TheList.above(), ThePanel::TheList);
    }

    /// Every line of the panel 1 holds the width of that panel, and it names a
    /// key. See T-320.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn every_line_of_the_panel_of_the_views_holds_its_width() {
        // The inside of a stack of 34 columns holds 32 of them.
        let width = THE_WIDTH_OF_THE_STACK - 2;
        let lines = the_lines_of_the_views(width);

        assert_eq!(lines.len(), THE_VIEWS.len());

        for (line, view) in lines.iter().zip(THE_VIEWS) {
            assert!(
                line.chars().count() <= usize::from(width),
                "the line {line:?} of {} characters stands over the {width} columns of the panel",
                line.chars().count()
            );
            assert!(
                line.ends_with(view.key),
                "the line {line:?} does not end with the key {:?}",
                view.key
            );
        }

        // **A panel of a narrow terminal writes no line of a negative width**:
        // the stack stands at the three columns alone, and a width of nothing
        // must give no panic at all.
        assert_eq!(the_lines_of_the_views(0).len(), THE_VIEWS.len());
        assert_eq!(the_lines_of_the_views(3).len(), THE_VIEWS.len());
    }

    /// The border of the panel that holds the focus is heavy, and the border of
    /// every other panel is light. See T-320.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_border_of_the_focus_is_heavy_and_the_other_borders_are_light() {
        use ratatui::{buffer::Buffer, widgets::Widget};

        let area = Rect::new(0, 0, 20, 4);

        let mut of_the_focus = Buffer::empty(area);
        a_panel(4, "Library", true).render(area, &mut of_the_focus);

        let mut of_no_focus = Buffer::empty(area);
        a_panel(4, "Library", false).render(area, &mut of_no_focus);

        // `BorderType::Double` gives `═` and `║`, and `BorderType::Plain` gives
        // `─` and `│`. **The shape of the border says the focus**, therefore a
        // theme of a low contrast still shows it.
        let the_row = |buf: &Buffer, y: u16| -> String {
            (0..area.width)
                .map(|x| buf[(x, y)].symbol().to_owned())
                .collect()
        };

        assert!(
            the_row(&of_the_focus, 3).contains('═'),
            "the border of the focus holds no double line: {:?}",
            the_row(&of_the_focus, 3)
        );
        assert!(
            the_row(&of_no_focus, 3).contains('─'),
            "the border of no focus holds no light line: {:?}",
            the_row(&of_no_focus, 3)
        );
        assert!(
            !the_row(&of_no_focus, 3).contains('═'),
            "the border of no focus holds a double line"
        );

        // The number of the panel stands at the start of the title, and the
        // name of the panel after it.
        assert!(the_row(&of_the_focus, 0).contains("4 Library"));

        // **No colour of a panel is a colour of RGB** (T-317).
        for buf in [&of_the_focus, &of_no_focus] {
            for x in 0..area.width {
                for y in 0..area.height {
                    let style = buf[(x, y)].style();
                    for colour in [style.fg, style.bg] {
                        assert!(
                            !matches!(colour, Some(Color::Rgb(_, _, _))),
                            "the panel holds a colour of RGB that no theme of a terminal gives"
                        );
                    }
                }
            }
        }
    }
}

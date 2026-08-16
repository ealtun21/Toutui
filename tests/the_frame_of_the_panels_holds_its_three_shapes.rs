//! The frame of the panels, its three shapes, and its focus. See T-320.
//!
//! **The maintainer chose the mockup 1, the panels, on 2026-08-16**, and the
//! stage 2 of that road is the frame: the stack of the panels at the left, the
//! panel 4 of the list in the middle, and the panels of the covers at the right.
//!
//! **The design needs about 120 columns and this fork measures 40 columns as
//! its narrowest** (T-301), therefore the frame holds three shapes, and **a
//! shape that no measurement reached is not a shape of this program**. The
//! measurement of the real program inside tmux of 2026-08-16, of the Library
//! view with 500 items of 2056:
//!
//! - **160 columns**: `┌1 Views ─┐╔4 Library [500 items of 2056] ═╗` and the
//!   panel of the covers after it. The stack holds 34 columns and the 14 views
//!   of the program with the key of each.
//! - **120 columns**: the same frame, and the panel 4 holds 50 columns.
//! - **100 columns**: no stack at all, and the screen is
//!   `────Library [500 items of 2056]────` with the panel of the covers, which
//!   is the screen of today.
//! - **40 columns**: no stack and no covers, which is the screen of today.
//!
//! **The focus is the shape of a border and not a colour alone**: the key `1`
//! of that measurement gave `╔1 Views ═╗┌4 Library ─┐`, and the key `Ctrl+h`
//! and the key `Ctrl+l` moved it back and forward.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::{ListState, Widget},
};
use toutui::ui::frame::{
    the_lines_of_the_views, the_shape_of, the_stack_and_the_work, ThePanel, TheShape, THE_VIEWS,
    THE_WIDTH_OF_THE_STACK, THE_WIDTH_OF_THREE_COLUMNS, THE_WIDTH_OF_TWO_COLUMNS,
};

/// The width of the screen gives the shape of the frame, and the stack stands
/// for the three columns alone. See T-320.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_three_shapes_of_the_frame_come_of_the_width_of_the_screen() {
    // The four widths of the measurement of tmux of 2026-08-16.
    assert_eq!(the_shape_of(160), TheShape::ThreeColumns);
    assert_eq!(the_shape_of(120), TheShape::ThreeColumns);
    assert_eq!(the_shape_of(100), TheShape::TwoColumns);
    assert_eq!(the_shape_of(40), TheShape::OneColumn);

    // **The work of the view keeps the columns of a panel of a cover at the
    // smallest width of the three columns** (`cover::MIN_WIDTH_FOR_COVER` of 84
    // columns): a stack that took the width of the covers away would give a
    // screen of three columns with no cover at all.
    let (stack, work) = the_stack_and_the_work(
        Rect::new(0, 2, THE_WIDTH_OF_THREE_COLUMNS, 30),
        TheShape::ThreeColumns,
    );
    assert_eq!(
        stack.map(|one| one.width),
        Some(THE_WIDTH_OF_THE_STACK),
        "the three columns hold the stack of the panels"
    );
    assert!(
        work.width >= THE_WIDTH_OF_TWO_COLUMNS,
        "the work of the view holds {} columns and the covers need {THE_WIDTH_OF_TWO_COLUMNS}",
        work.width
    );

    // The two other shapes give the whole width to the work of the view, which
    // is the screen of today.
    for width in [119u16, 84, 83, 40] {
        let area = Rect::new(0, 2, width, 30);
        let shape = the_shape_of(width);
        let (stack, work) = the_stack_and_the_work(area, shape);

        assert_eq!(
            stack, None,
            "a screen of {width} columns holds no stack of the panels"
        );
        assert_eq!(work, area);
    }
}

/// The panel that holds the focus takes a heavy border, and every other panel
/// takes a light one. See T-320.
///
/// **The shape of the border says the focus, and not a colour alone** (the
/// section (c) of `docs/mockups/mockup-1.md`): a terminal of a theme of a low
/// contrast still says where the focus is, and a terminal that gives no colour
/// at all says it too.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_border_of_the_panel_of_the_focus_holds_another_shape() {
    let area = Rect::new(0, 0, 40, 6);

    let the_borders_of = |it_holds_the_focus: bool| -> String {
        let mut buf = Buffer::empty(area);
        toutui::ui::frame::a_panel(4, "Library [500 items of 2056]", it_holds_the_focus)
            .render(area, &mut buf);

        (0..area.width)
            .map(|x| buf[(x, area.height - 1)].symbol().to_owned())
            .collect()
    };

    let of_the_focus = the_borders_of(true);
    let of_no_focus = the_borders_of(false);

    assert!(
        of_the_focus.contains('═') && !of_the_focus.contains('─'),
        "the border of the focus must be heavy: {of_the_focus:?}"
    );
    assert!(
        of_no_focus.contains('─') && !of_no_focus.contains('═'),
        "the border of a panel that does not hold the focus must be light: {of_no_focus:?}"
    );

    // **The list of a view of a screen that holds no frame of the panels keeps
    // the block of one border at the top that it had**: the road of the panels
    // reaches the Home view and the Library view of a wide terminal, and no
    // other view and no narrow terminal.
    let mut of_no_panel = Buffer::empty(area);
    toutui::ui::the_list_of_a_view::render_the_list(
        area,
        &mut of_no_panel,
        &toutui::config::Colors::default(),
        "Library [500 items of 2056]",
        &["Large Book 2056".to_string()],
        &mut ListState::default(),
    );
    let of_the_foot: String = (0..area.width)
        .map(|x| of_no_panel[(x, area.height - 1)].symbol().to_owned())
        .collect();
    assert!(
        !of_the_foot.contains('═') && !of_the_foot.contains('│'),
        "a list with no panel holds no border at its foot: {of_the_foot:?}"
    );
}

/// The digits name the panels that the frame draws, and no other panel. See
/// T-320.
///
/// **A key that does nothing is a fault of its own** (T-79), therefore the
/// three panels of the design that no stage drew hold no digit: the panels 5
/// and 6 of the covers come with T-319, and the panel 7 of the player comes
/// with T-322. The panel 2 of the sequence and the panel 3 of the filter came
/// with T-318.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_digits_name_the_panels_that_the_frame_draws() {
    assert_eq!(ThePanel::of_the_digit('1'), Some(ThePanel::TheViews));
    assert_eq!(ThePanel::of_the_digit('2'), Some(ThePanel::TheSequence));
    assert_eq!(ThePanel::of_the_digit('3'), Some(ThePanel::TheFilter));
    assert_eq!(ThePanel::of_the_digit('4'), Some(ThePanel::TheList));

    for digit in ['0', '5', '6', '7', '8', '9'] {
        assert_eq!(
            ThePanel::of_the_digit(digit),
            None,
            "the digit {digit} names a panel that no stage of the road of the panels drew"
        );
    }

    // **The panel of the start is the list of the view**, therefore every key of
    // this program does the work that it did until the user moves the focus.
    assert_eq!(ThePanel::default(), ThePanel::TheList);
}

/// Every line of the panel 1 names a key that the handler of this program holds.
/// See T-320.
///
/// **A text must not promise a function that the program does not have**
/// (T-118): the mockup gave the bookmarks the key `k` and the collections the
/// key `c`, and the key `k` of this program moves a list one line up. This test
/// reads `src/app.rs` and it finds the key of every line.
///
/// **The parts of this test stay in one function.**
#[test]
fn every_line_of_the_panel_of_the_views_names_a_key_of_the_handler() {
    let handler = include_str!("../src/app.rs");

    for view in THE_VIEWS {
        // The Home view and the Library view take the key `Tab`, and that key
        // is `KeyCode::Tab` and no character at all.
        if view.key == "Tab" {
            assert!(
                handler.contains("KeyCode::Tab =>"),
                "the panel of the views names the key Tab and the handler holds no arm of it"
            );
            continue;
        }

        let of_the_handler = format!("KeyCode::Char('{}')", view.key);
        assert!(
            handler.contains(&of_the_handler),
            "the panel of the views names the key {:?} of the view {:?}, and \
             src/app.rs holds no arm {of_the_handler}",
            view.key,
            view.name
        );
    }

    // Every line of the panel holds the width of the panel, and it ends with its
    // key: the name stands at the left and the key stands at the right.
    let width = THE_WIDTH_OF_THE_STACK - 2;
    let lines = the_lines_of_the_views(width);

    assert_eq!(lines.len(), THE_VIEWS.len());
    for (line, view) in lines.iter().zip(THE_VIEWS) {
        assert!(
            line.chars().count() <= usize::from(width),
            "the line {line:?} stands over the {width} columns of the panel"
        );
        assert!(line.ends_with(view.key), "the line {line:?} names no key");
    }
}

/// The keys of the frame stand in the list of the keys of the user, and the
/// footer names them. See T-320.
///
/// **A key that exists and that the user cannot find is a fault** (the rule of
/// T-143 in reverse, and the item 1 of T-318): the key `f` of the sequence and
/// of the filter stood in the program of 2026-08-16 and the footer of the
/// Library view named it nowhere.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_keys_of_the_frame_stand_in_the_list_of_the_keys_and_in_the_footer() {
    let written: String = toutui::ui::keys::GROUPS
        .iter()
        .flat_map(|group| group.keys.iter().map(|one| one.key))
        .collect::<Vec<_>>()
        .join("\n");

    for of_the_frame in ["1", "4", "Ctrl+h", "Ctrl+l"] {
        assert!(
            written.lines().any(|line| line == of_the_frame),
            "the key {of_the_frame} of the frame of the panels stands in no group of the keys"
        );
    }

    // **A footer must not promise a key that the view does not hold** (T-143):
    // the frame stands at 120 columns and more, therefore the footer of a
    // narrow terminal names no panel at all.
    let of_the_view = toutui::ui::keys::FOOTER_OF_A_LIBRARY_OF_BOOKS;

    let of_no_frame =
        toutui::ui::keys::the_footer_of_a_panel(of_the_view, false, ThePanel::TheList);
    assert_eq!(
        of_no_frame, of_the_view,
        "the footer of a screen that holds no frame is the footer of the view"
    );

    // The panel 4 holds the focus: the footer of the view stands, and it names
    // the key of the panel of the views beside it.
    let of_the_list = toutui::ui::keys::the_footer_of_a_panel(of_the_view, true, ThePanel::TheList);
    assert!(of_the_list.starts_with(of_the_view));
    assert!(of_the_list.contains('1') && of_the_list.contains("Ctrl+h"));

    // **The panel 1 holds the focus, therefore the footer is the footer of that
    // panel**: the key `l` of that moment opens a view and it plays no media,
    // and a footer that said `l: play or open` would name a work that the key
    // does not do.
    let of_the_views =
        toutui::ui::keys::the_footer_of_a_panel(of_the_view, true, ThePanel::TheViews);
    assert!(
        of_the_views.contains("l: open the view"),
        "the footer of the panel of the views must name the work of the key l: {of_the_views:?}"
    );
    assert!(
        !of_the_views.contains("play or open"),
        "the footer of the panel of the views must not promise a playback: {of_the_views:?}"
    );
}

/// No colour of the frame of the panels is a colour of RGB. See T-317 and
/// T-320.
///
/// **The theme of the terminal of the user is the theme of the program**, and a
/// panel that painted a grey of RGB over it would take that theme away again.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_frame_of_the_panels_holds_no_colour_of_rgb() {
    let area = Rect::new(0, 0, 40, 6);

    for it_holds_the_focus in [true, false] {
        let mut buf = Buffer::empty(area);
        toutui::ui::frame::a_panel(1, "Views", it_holds_the_focus).render(area, &mut buf);

        for x in 0..area.width {
            for y in 0..area.height {
                let style = buf[(x, y)].style();

                for colour in [style.fg, style.bg] {
                    assert!(
                        !matches!(colour, Some(Color::Rgb(_, _, _))),
                        "the panel holds the colour {colour:?}, and no theme of a terminal gives it"
                    );
                }
            }
        }
    }
}

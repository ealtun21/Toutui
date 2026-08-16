//! The mouse of the program, and the panel that a click of it names. See T-316.
//!
//! **The stage 3 of the road of the panels is the mouse**, and this file holds
//! the gate of it. The measurement of the real program inside tmux of
//! 2026-08-16, of the Home view of the library `Large` at 160 columns and 45
//! rows, with `docs/harness/click.sh`:
//!
//! - **v0.8.146, before this stage**: twelve reports of the mouse — a click of
//!   a row of the list, a click of a line of the panel 1, and five steps of the
//!   wheel — and **the screen did not change at all**. The cursor stayed on
//!   `Large Book 0001`, the focus stayed on the panel 4, and the list did not
//!   move. The control of that same run: the key `j` after those reports moved
//!   the cursor to `Large Book 0002`, therefore the bytes reached the program
//!   and `crossterm` read them. **The key `Esc` stops this program** (the trap
//!   69), and the program stood after the twelve of them: `crossterm` therefore
//!   made an `Event::Mouse` of each report, and the loop of `src/main.rs` read
//!   the `Event::Key` alone and it dropped every one of them.
//! - **v0.8.147, after it**: a click of the row 10 gave `➤ Large Book 0006`, a
//!   click of the line `Collections` of the panel 1 gave
//!   `╔1 Views ═╗┌4 Home ─┐` with `➤ Queue` on the row of the pointer, three
//!   steps of the wheel over the panel 1 gave `➤ Bookmarks`, and five steps
//!   over the list gave `➤ Large Book 0006` of `Large Book 0001`. The key
//!   `Ctrl+o` then said `The program does not read the mouse.`, the same click
//!   moved nothing, and the key `Ctrl+o` again gave the mouse back.
//!
//! **The mouse reaches every view**: the same measurement at 100 columns, where
//! the frame draws no stack, gave `➤ Large Book 0002` for a click of the row of
//! that book, and the view of the keys of the key `?` gave the line of its own
//! row for a click of it.

use ratatui::layout::Rect;
use toutui::ui::the_mouse::{
    the_line_of_a_row, the_target_of_a_point, TheAreasOfTheMouse, TheTarget, THE_MOUSE_STANDS,
    THE_MOUSE_STOPPED,
};

/// The areas of the frame of the measurement of tmux at 160 columns.
fn the_areas_of_the_measurement() -> TheAreasOfTheMouse {
    TheAreasOfTheMouse {
        the_panel_of_the_views: Rect::new(0, 2, 34, 40),
        the_lines_of_the_views: Rect::new(1, 3, 32, 38),
        the_offset_of_the_views: 0,
        the_views: 14,
        the_panel_of_the_list: Rect::new(34, 2, 74, 40),
        the_lines_of_the_list: Rect::new(35, 3, 72, 38),
        the_offset_of_the_list: 0,
        the_lines: 20,
        the_header_of_the_list: Rect::default(),
    }
}

/// A click of a row of the panel 4 names the line of that row, and a click of a
/// row of the panel 1 names the line of that one. See T-316.
///
/// **The parts of this test stay in one function.**
#[test]
fn a_click_of_a_row_names_the_line_of_that_row() {
    let areas = the_areas_of_the_measurement();

    // The measurement of tmux: the row 10 of the screen holds
    // `Large Book 0006`, which is the line 7 of the list — the row 4 holds the
    // name of the shelf `Recently Added`, and the row 5 holds the first book.
    // The rows of this test count from 0, and the rows of tmux count from 1.
    assert_eq!(
        the_target_of_a_point(&areas, true, 60, 9),
        TheTarget::TheListOfTheView { the_line: Some(6) },
        "the click of the measurement of tmux names the line of its row"
    );

    // The measurement of tmux: the row 10 of the panel 1 holds `Queue`, which
    // is the line 6 of the 14 views.
    assert_eq!(
        the_target_of_a_point(&areas, true, 10, 9),
        TheTarget::ThePanelOfTheViews { the_line: Some(6) },
        "the click of the panel 1 names the line of its row"
    );

    // **A click on the border of a panel gives the focus of it and no line**:
    // the border is a part of the panel that the user can see.
    assert_eq!(
        the_target_of_a_point(&areas, true, 0, 9),
        TheTarget::ThePanelOfTheViews { the_line: None }
    );
    assert_eq!(
        the_target_of_a_point(&areas, true, 34, 9),
        TheTarget::TheListOfTheView { the_line: None }
    );

    // **A row under the last line of the list holds no line at all**: the Home
    // view of the measurement holds 20 lines, and the panel holds 38 rows.
    assert_eq!(
        the_target_of_a_point(&areas, true, 60, 30),
        TheTarget::TheListOfTheView { the_line: None },
        "a click under the last line of a short list moves no cursor"
    );
}

/// The offset of a list moves the line of a row, therefore a click of a list
/// that the user scrolled names the line that stands on that row. See T-316.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_offset_of_a_list_moves_the_line_of_a_click() {
    let mut areas = the_areas_of_the_measurement();
    areas.the_lines = 2056;
    areas.the_offset_of_the_list = 400;

    // The first row of the panel holds the line 400, and the row after it
    // holds the line 401.
    assert_eq!(
        the_target_of_a_point(&areas, true, 60, 3),
        TheTarget::TheListOfTheView {
            the_line: Some(400)
        }
    );
    assert_eq!(
        the_target_of_a_point(&areas, true, 60, 9),
        TheTarget::TheListOfTheView {
            the_line: Some(406)
        }
    );

    // The same rule for the panel of the views.
    areas.the_offset_of_the_views = 3;
    assert_eq!(
        the_target_of_a_point(&areas, true, 10, 3),
        TheTarget::ThePanelOfTheViews { the_line: Some(3) }
    );

    // The arithmetic of one row, with no panel at all.
    assert_eq!(
        the_line_of_a_row(Rect::new(0, 5, 10, 4), 400, 2056, 5),
        Some(400)
    );
    assert_eq!(
        the_line_of_a_row(Rect::new(0, 5, 10, 4), 400, 2056, 8),
        Some(403)
    );
    assert_eq!(
        the_line_of_a_row(Rect::new(0, 5, 10, 4), 400, 2056, 9),
        None
    );
}

/// A frame that draws no stack takes no click of a stack, and the panel of a
/// view that no frame drew holds no cell of the screen. See T-316.
///
/// **The parts of this test stay in one function.**
#[test]
fn a_panel_that_the_frame_did_not_draw_takes_no_click() {
    // The measurement of tmux at 100 columns: the frame draws no stack, and
    // `the_stack_of_the_panels` writes `Rect::default()` for the panel 1.
    let areas = TheAreasOfTheMouse {
        the_panel_of_the_views: Rect::default(),
        the_lines_of_the_views: Rect::default(),
        the_offset_of_the_views: 0,
        the_views: 14,
        the_panel_of_the_list: Rect::new(0, 2, 59, 30),
        the_lines_of_the_list: Rect::new(0, 3, 58, 29),
        the_offset_of_the_list: 0,
        the_lines: 20,
        the_header_of_the_list: Rect::default(),
    };

    // The measurement of tmux: a click of the column 5 of the row 6 gave
    // `➤ Large Book 0002`, which is the line 2 of the list.
    assert_eq!(
        the_target_of_a_point(&areas, false, 5, 5),
        TheTarget::TheListOfTheView { the_line: Some(2) }
    );

    // **A click of the column 60 of that screen names nothing**: the panel of
    // the covers stands there, and no stage of the road of the panels drew a
    // target of it. The measurement of tmux moved no cursor for that click.
    assert_eq!(
        the_target_of_a_point(&areas, false, 60, 7),
        TheTarget::Nothing
    );

    // **The areas of the start hold no cell at all**, therefore a report of the
    // mouse that comes before the first frame names no panel (the trap 214).
    assert_eq!(
        the_target_of_a_point(&TheAreasOfTheMouse::default(), true, 10, 5),
        TheTarget::Nothing
    );
}

/// The program says what the key `Ctrl+o` did, and the two sentences name the
/// key of the road back. See T-316 and T-170.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_words_of_the_key_of_the_mouse_name_the_road_back() {
    for words in [THE_MOUSE_STANDS, THE_MOUSE_STOPPED] {
        assert!(
            words.contains("Ctrl+o"),
            "the sentence names the key that does the work of it: {}",
            words
        );
        assert!(
            words.ends_with('.'),
            "the sentence of a message of this program ends with a stop: {}",
            words
        );
    }

    // **The words of the mouse that stopped name the selection of the text**,
    // which is the reason of that key: a user who reads
    // `The program does not read the mouse.` alone does not know why they
    // pressed it.
    assert!(THE_MOUSE_STOPPED.contains("select the text"));
    assert!(THE_MOUSE_STANDS.contains("select the text"));
}

/// The list of the keys of the program names the mouse, and the loop of
/// `src/main.rs` gives the report of it to the handler. See T-316 and T-143.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_program_names_the_mouse_and_it_reads_the_report_of_it() {
    // **A key that the list of the keys does not name is a key that the user
    // cannot find** (T-143 in reverse), therefore the view of the key `?` holds
    // the mouse and the key that stops it.
    let of_the_keys: Vec<&str> = toutui::ui::keys::GROUPS
        .iter()
        .flat_map(|group| group.keys.iter().map(|one| one.key))
        .collect();

    for one in ["Click", "Wheel", "Ctrl+o"] {
        assert!(
            of_the_keys.contains(&one),
            "the list of the keys of the program names `{}`",
            one
        );
    }

    // **The loop of the program reads the report of the mouse** (T-316): the
    // loop before this stage held `if let event::Event::Key(key) = event`
    // alone, therefore every report of the mouse went away with no work at all.
    let of_the_loop = std::fs::read_to_string("src/main.rs").expect("src/main.rs");
    assert!(
        of_the_loop.contains("event::Event::Mouse(the_report)")
            && of_the_loop.contains("app.handle_the_mouse(the_report)"),
        "the loop of the program gives the report of the mouse to the handler of it"
    );

    // **The terminal sends no report before the program asks for it**, and the
    // exit gives the mouse back to the terminal: a shell that keeps the capture
    // writes the report of every move of the pointer into the line of the user.
    let of_the_terminal = std::fs::read_to_string("src/utils/the_terminal_of_the_program.rs")
        .expect("src/utils/the_terminal_of_the_program.rs");
    assert!(
        of_the_terminal.contains("EnableMouseCapture")
            && of_the_terminal.contains("DisableMouseCapture"),
        "the program asks the terminal for the reports of the mouse, and it stops them"
    );

    let of_the_exit =
        std::fs::read_to_string("src/utils/exit_app.rs").expect("src/utils/exit_app.rs");
    assert!(
        of_the_exit.contains("DisableMouseCapture"),
        "the exit of the program gives the mouse back to the terminal"
    );
}

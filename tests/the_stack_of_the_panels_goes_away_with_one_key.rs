//! The mode that hides the panels 1, 2, and 3 of the stack. See T-323.
//!
//! **The maintainer chose the mockup 1, the panels, on 2026-08-16**, and the
//! stage 8 of that road is the mode of a quiet screen. The section (f) of
//! `docs/mockups/mockup-1.md` names the cost of the design in its own words:
//! "Today, when nothing plays and there is no message, the screen is mostly
//! empty and calm. This design is always full. A user who wants a small, quiet
//! screen will find it busy. The answer is a key that hides panels 1, 2 and 3
//! and gives their columns to the list, but that is a second mode and not the
//! default."
//!
//! **The screen of the real program v0.8.152 inside tmux**, of the Library view
//! of the library `Books` of the sandbox on a screen of 160 columns and 45
//! rows:
//!
//! ```text
//! ┌1 Views ───────────────────────┐╔4 Library [18 items] ══════════════════╗ ┌5 Cover ─────────┐
//! │➤ Home                      Tab│║    Title              Author  Time Done║ │Author: N/A      │
//! │  Library                   Tab│║➤ ✓ A Book Of An Epub…          <1m done║ │Progress: 100%   │
//! └───────────────────────────────┘╚════════════════════════════════════════╝ └─────────────────┘
//! ```
//!
//! The stack held 34 columns of the 160, the panel 4 of the list held 73 of
//! them, and **the key `z` of that program did nothing and it said nothing at
//! all**: no key of this program gave the user the quiet screen back.
//!
//! **The corrected program, of the same harness, after the key `z`**:
//!
//! ```text
//! 🔗 localhost:13399                          ⇅ The sequence of the server ▣ No filter
//! ╔4 Library [18 items] ══════════════════════════════════════════╗ ┌5 Cover ─────────────────┐
//! ║    Title                             Author       Time    Done║ │Author: N/A              │
//! ║➤ ✓ A Book Of An Epub With No Container              <1m   done║ │Progress: 100%, Finished │
//! ╚═══════════════════════════════════════════════════════════════╝ └─────────────────────────┘
//!                       The panels 1 to 3 are hidden. Press the key z for them.
//!   j/k: move  …  Q: quit  f: sequence  z: the panels 1 to 3
//! ```
//!
//! The panel 4 grew from 73 columns to 93 and the panel 5 from 48 to 62, **the
//! second row of the header took the words of the sequence and of the filter**
//! (T-318 holds that rule for every screen that draws no stack), and the footer
//! named the key of the road back and no digit of a panel that went away.
//!
//! **The digit `1` of that mode does nothing** (T-79): two keys `j` after it
//! moved the list of the panel 4, therefore the focus stayed where the user
//! could see it. **The digit `5` still gives the panel 5 the focus**, because
//! the key `z` takes the stack away and not the frame. The key `z` a second
//! time gave the stack back, and the key `R` of a refresh kept the mode.

use ratatui::layout::Rect;
use toutui::ui::frame::{
    the_stack_and_the_work, TheShape, THE_WIDTH_OF_THE_STACK, THE_WIDTH_OF_THREE_COLUMNS,
};
use toutui::ui::keys::{the_footer_of_a_panel, GROUPS};

/// The area of a screen of 160 columns and 45 rows.
fn the_screen() -> Rect {
    Rect::new(0, 0, 160, 45)
}

/// The key of the user takes the stack away, and the work of the view then
/// holds every column of the screen. See T-323.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_key_of_the_user_takes_the_stack_of_the_panels_away() {
    let area = the_screen();

    // The mode of the start: the stack stands, and it holds the 34 columns of
    // the design.
    let (stack, work) = the_stack_and_the_work(area, TheShape::ThreeColumns, false);
    let stack = stack.expect("the stack of the mode of the start stands");
    assert_eq!(stack.width, THE_WIDTH_OF_THE_STACK);
    assert_eq!(work.width, area.width - THE_WIDTH_OF_THE_STACK);
    assert_eq!(work.x, area.x + THE_WIDTH_OF_THE_STACK);

    // The mode of the user: the stack goes away, and the work takes the whole
    // width of the screen with the 34 columns of the stack inside it.
    let (stack, work) = the_stack_and_the_work(area, TheShape::ThreeColumns, true);
    assert!(
        stack.is_none(),
        "the key of the user must take the stack away"
    );
    assert_eq!(work, area);
    assert_eq!(work.width, area.width);

    // **A screen that draws no stack draws no stack in the two modes**: the two
    // shapes that are not the three columns hold no stack of their own, and the
    // mode of the user changes no area of them at all.
    for shape in [TheShape::TwoColumns, TheShape::OneColumn] {
        for the_user_hid_the_stack in [false, true] {
            let (stack, work) = the_stack_and_the_work(area, shape, the_user_hid_the_stack);
            assert!(stack.is_none(), "the shape {shape:?} holds no stack");
            assert_eq!(work, area);
        }
    }

    // The smallest screen of the three columns keeps the same rule.
    let narrow = Rect::new(0, 0, THE_WIDTH_OF_THREE_COLUMNS, 45);
    let (stack, work) = the_stack_and_the_work(narrow, TheShape::ThreeColumns, true);
    assert!(stack.is_none());
    assert_eq!(work.width, THE_WIDTH_OF_THREE_COLUMNS);
}

/// The footer of the panel 4 names the key of the mode, and it names no digit
/// of a panel that the mode took away. See T-323 and T-143.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_footer_of_the_panel_of_the_list_names_the_key_of_the_stack() {
    use toutui::ui::frame::ThePanel;

    let of_the_view = "j/k: move  l: play or open  Q: quit";

    // The stack stands: the footer names the digit of the panel 1, and it names
    // the key that hides the three panels.
    let with_the_stack = the_footer_of_a_panel(of_the_view, true, true, ThePanel::TheList);
    assert!(with_the_stack.contains("1/Ctrl+h: the panels"));
    assert!(with_the_stack.contains("z: hide them"));

    // The stack is hidden: the digit of the panel 1 does nothing at that
    // moment (T-79), therefore the footer must not promise it (T-143). The key
    // `z` stays, because it is the road back.
    let with_no_stack = the_footer_of_a_panel(of_the_view, true, false, ThePanel::TheList);
    assert!(
        !with_no_stack.contains("1/Ctrl+h"),
        "the footer of the mode that hides the stack must promise no digit of it: {with_no_stack}"
    );
    assert!(with_no_stack.contains("z: the panels 1 to 3"));

    // **A screen that holds no frame of the panels names no panel at all**
    // (T-320): the key `z` does nothing there, therefore the footer of the view
    // stays as it is.
    let with_no_frame = the_footer_of_a_panel(of_the_view, false, false, ThePanel::TheList);
    assert_eq!(with_no_frame, of_the_view);
    assert!(!with_no_frame.contains('z'));
}

/// The view of the key `?` names the key of the mode. See T-323.
///
/// **A key that no view of the user names is a key that the user does not
/// have**, which is the rule of T-143 in reverse.
#[test]
fn the_view_of_every_key_names_the_key_of_the_stack() {
    let of_the_panels = GROUPS
        .iter()
        .find(|group| group.name.starts_with("The panels"))
        .expect("the group of the panels of the frame stands");

    let the_key = of_the_panels
        .keys
        .iter()
        .find(|key| key.key == "z")
        .expect("the group of the panels names the key z");

    assert!(the_key.what.contains("Hide the panels 1 to 3"));
}

/// The key of the mode, the focus of a panel that goes away, and the mode that
/// a refresh keeps. See T-323.
///
/// **A source of this shape has no unit of its own**: `App` needs a database
/// and a server, therefore this gate reads the three rules in the file of the
/// handler. **The block of each rule ends at the function after it** (the trap
/// 209 of `docs/HANDOVER.md`), and no window of a number of characters stands
/// here.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_key_of_the_mode_holds_its_three_rules() {
    let of_the_handler = std::fs::read_to_string("src/app.rs").expect("src/app.rs");

    // 1. The key `z` takes the focus of a panel of the stack to the panel 4:
    // the focus must not stand on a panel that holds no cell of the screen
    // (T-79).
    let at = of_the_handler
        .find("if key.code == KeyCode::Char('z') {")
        .expect("the handler holds the key z of the stack");
    let the_key = &of_the_handler[at..];
    let the_key = &the_key[..the_key
        .find("\n        // **A digit of a panel")
        .expect("the key z stands before the digits of the panels")];
    assert!(the_key.contains("self.the_stack_is_hidden = !self.the_stack_is_hidden;"));
    assert!(the_key.contains("self.the_panel_of_the_focus.is_of_the_stack()"));
    assert!(the_key.contains("self.the_panel_of_the_focus = ThePanel::TheList;"));
    assert!(the_key.contains("The panels 1 to 3 are hidden."));
    assert!(the_key.contains("The panels 1 to 3 stand again."));

    // 2. The panel 1 reads the area of the last frame, as the panels 2, 3, and
    // 5 do: the digit of a panel that the key `z` took away does nothing.
    let at = of_the_handler
        .find("pub fn a_panel_of_the_frame_stands(")
        .expect("the handler holds a_panel_of_the_frame_stands");
    let the_rule = &of_the_handler[at..];
    let the_rule = &the_rule[..the_rule
        .find("\n    /// ")
        .expect("the function after a_panel_of_the_frame_stands")];
    assert!(
        the_rule.contains("crate::ui::frame::ThePanel::TheViews =>"),
        "the panel 1 must read the area of the last frame: {the_rule}"
    );

    // 3. A refresh keeps the mode: the key `R` makes the application again, and
    // a user who hid the stack must not get the busy screen back.
    assert!(of_the_handler.contains("the_stack_is_hidden: self.the_stack_is_hidden,"));
    assert!(of_the_handler.contains("self.the_stack_is_hidden = of_the_old.the_stack_is_hidden;"));
}

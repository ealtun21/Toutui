//! A click of a row of the Chapters view plays that chapter. See T-330.5.
//!
//! **The maintainer read the program v0.8.158 and they gave six points**, and
//! the part 5 of the first of them is the Chapters view of
//! `docs/mockups/mockup-7.txt`. That part holds three rounds: the two bars
//! (v0.8.164), the table of the times (v0.8.165), and the key of a chapter and
//! the click of a row. This round takes the third of them.
//!
//! **The map of the mouse of `docs/mockups/mockup-7.md` says it in one line**:
//! "A click on a row — Plays that chapter". The key `l` of the same row does
//! that work since T-24, and the footer of the view names it.
//!
//! **The first fault.** The measurement of the real program v0.8.165 inside
//! tmux, of the library `Books` of the sandbox at 160 columns and 45 rows, with
//! `A Second Book Of Many Hours` and its 70 chapters at 3:00:08 of its eight
//! hours. A click of the row 12 of the screen, which is the chapter 5, moved
//! the cursor to that chapter, it said nothing at all, and the playback stayed
//! at 3:00:43:
//!
//! ```text
//! ➤    5  Chapter 5 of the second book                     19:08   6m02s
//! ```
//!
//! The key `l` of that same row then said `The playback goes to "Chapter 5 of
//! the second book".` and the playback stood at 19:34. **The click of a row of
//! this view therefore held half of the work of its key.**
//!
//! **The second fault, which the first one hid.** The map of the mouse of this
//! view read the offset 0 at every frame, therefore a click of a row of a list
//! that scrolled gave the chapter of that row of the **first** screen of the
//! list. The key `G` of the same book gave the rows 35 to 70 of the list:
//!
//! ```text
//!     35  Chapter 35 of the second book                   3:50:18   9m40s
//!     36  Chapter 36 of the second book                   3:59:58   4m31s
//! ```
//!
//! A click of the row 9 of that screen, which is the chapter 36, gave the
//! **chapter 2**. `render_chapters` of `src/ui/tui.rs` gave the render a copy of
//! the line of the user (`&mut self.list_state_chapters.clone()`), and ratatui
//! writes the offset of the list into the state while it draws it: the copy took
//! that offset to nowhere, and the map of the mouse then read the offset of the
//! state before the render, which no key of this view ever changes. **A click
//! that plays a chapter of a wrong number is worse than a click that does
//! nothing**, therefore the two corrections belong to one round.
//!
//! **The corrected program of the same harness.** The click of the row 12 gave
//! the chapter 5, the mark `▶` of the chapter that plays moved to it, the
//! playback stood at 19:38, and the row of the message said `The playback goes
//! to "Chapter 5 of the second book".` The key `G` and a click of the row 9 then
//! gave the chapter 36 and the playback at 4:00:28:
//!
//! ```text
//! ➤ ▶ 36  Chapter 36 of the second book                   3:59:58   4m31s
//! ```
//!
//! **The control of the same run**: a click of a row of the Home view moved the
//! cursor of that list and it opened no media at all, which is the rule of
//! T-316.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::ListState;
use toutui::app::AppView;
use toutui::config::Colors;
use toutui::ui::the_mouse::{the_click_of_a_row_opens_it, the_line_of_a_row};

/// Every view of this program. A view that comes after this round takes a line
/// of this list, and the rule of its click then stands in a test.
const THE_VIEWS: &[AppView] = &[
    AppView::Home,
    AppView::Library,
    AppView::SearchBook,
    AppView::PodcastEpisode,
    AppView::Series,
    AppView::SeriesBook,
    AppView::Lists,
    AppView::ListEntries,
    AppView::Reader,
    AppView::Stats,
    AppView::Sessions,
    AppView::SortFilter,
    AppView::Chapters,
    AppView::Bookmarks,
    AppView::Queue,
    AppView::NewPodcast,
    AppView::Authors,
    AppView::Ebooks,
    AppView::Downloads,
    AppView::PutInAList,
    AppView::SendToEreader,
    AppView::Keys,
    AppView::Settings,
    AppView::SettingsAccount,
    AppView::SettingsLibrary,
    AppView::SettingsAbout,
    AppView::SettingsUpdateUninstall,
    AppView::SettingsReader,
];

/// The Chapters view opens the row of a click. See T-330.5.
#[test]
fn a_click_of_a_row_of_the_chapters_view_opens_that_row() {
    assert!(
        the_click_of_a_row_opens_it(AppView::Chapters),
        "the map of the mouse of docs/mockups/mockup-7.md gives the row of the \
         Chapters view the work of the key of that row: a click of it plays \
         that chapter (T-330.5)"
    );
}

/// Every other view keeps the rule of T-316: a click moves the cursor alone.
#[test]
fn a_click_of_a_row_of_every_other_view_moves_the_cursor_alone() {
    for view in THE_VIEWS {
        if matches!(view, AppView::Chapters) {
            continue;
        }

        assert!(
            !the_click_of_a_row_opens_it(*view),
            "a click of a row of {:?} must move the cursor and do nothing else \
             (T-316): the list of a view holds the media of the library, and a \
             click that opened a media at once would give the user the view of \
             a book that they wanted to read the facts of",
            view
        );
    }
}

/// The click of a row of the list takes the road of the key of that row, and
/// the two therefore say the same words. See T-330.5.
///
/// This test reads the source, as the tests of T-135, T-143, and T-164 do: a
/// session that gives the click a `SeekTo` of its own takes the words of the
/// key away from it.
#[test]
fn the_click_of_a_row_takes_the_road_of_the_key_of_that_row() {
    let source = include_str!("../src/app.rs");

    let at = source
        .find("TheTarget::TheListOfTheView { the_line } =>")
        .expect("the map of the mouse must hold the list of the view");

    // The arm ends at the arm after it, which is the row of the header of the
    // table. A window of a number of characters would hold the comments of the
    // arm after it, and the words of a correction then take a line out of that
    // window (the trap 209).
    let end = source[at..]
        .find("TheTarget::TheHeaderOfTheList")
        .expect("the arm of the header of the table stands after this one");
    let arm = &source[at..at + end];

    assert!(
        arm.contains("the_click_of_a_row_opens_it(self.view_state)"),
        "the arm of a click of a row must ask the map of the mouse whether that \
         row opens: the rule belongs to one function, and every view of this \
         program stands in the test of it (T-330.5)"
    );

    assert!(
        arm.contains("self.go_to_the_chapter()"),
        "the click of a row of the Chapters view must call the road of the key \
         `l` of that row: a second road to the same work would say other words \
         and it would take another rule of the media that went away (T-330.5 \
         and T-162)"
    );
}

/// The render of the chapters gives the line of the user itself, and not a copy
/// of it. See T-330.5.
///
/// **ratatui writes the offset of the list into the state while it draws it**,
/// and the map of the mouse reads that offset after the render: a copy takes
/// the offset to nowhere, and every click of a list that scrolled then names a
/// row of the first screen of it.
#[test]
fn the_render_of_the_chapters_keeps_the_offset_of_the_screen() {
    let source = include_str!("../src/ui/tui.rs");

    let at = source
        .find("fn render_chapters(")
        .expect("the render of the chapters must stand");

    // The block ends at the function after this one, and not at a number of
    // characters (the trap 209).
    let end = source[at..]
        .find("fn render_sort_filter(")
        .expect("the render of the sequence and the filter stands after it");
    let block = &source[at..at + end];

    assert!(
        !block.contains("self.list_state_chapters.clone()"),
        "the render of the chapters must give the line of the user itself: a \
         copy of it takes the offset that ratatui writes to nowhere, and a \
         click of a row of a list that scrolled then names the row of the first \
         screen of that list — the key `G` of a book of 70 chapters gave the \
         rows 35 to 70, and a click of the second row of them gave the chapter \
         2 (T-330.5)"
    );

    // **The absence of the copy is not the rule** (T-330.5): a render that gave
    // the map of the mouse the state of `self` beside a state of its own would
    // hold no `clone` at all and it would keep the same fault. The state comes
    // out of `self` for the render, and it goes back after the map of the mouse
    // reads the offset that the render wrote.
    assert!(
        block.contains("std::mem::take(&mut self.list_state_chapters)"),
        "the render of the chapters must take the line of the user out of the \
         application for the render, because the render takes the colours of \
         the application beside it (T-330.5)"
    );

    let of_the_render = block
        .find("std::mem::take(&mut self.list_state_chapters)")
        .expect("the render takes the line of the user");
    let of_the_areas = block
        .find("self.the_areas_of_the_list_of_the_mouse(")
        .expect("the render writes the areas of the mouse");
    let of_the_road_back = block
        .find("self.list_state_chapters = ")
        .expect("the line of the user goes back into the application");

    assert!(
        of_the_render < of_the_areas && of_the_areas < of_the_road_back,
        "the line of the user comes out of the application, the map of the \
         mouse reads the offset that the render wrote, and the line then goes \
         back: a road back before the map of the mouse gives that map the \
         offset of the frame before this one (T-330.5)"
    );
}

/// The render writes the offset of the screen into the state that it takes, and
/// a click of a row reads that offset. See T-330.5.
///
/// **A gate of the source alone says nothing of the render** (the shape of
/// T-256): this test draws the list into a `Buffer` and it then reads the offset
/// that the render wrote.
#[test]
fn a_click_of_a_row_of_a_list_that_scrolled_names_the_row_of_the_screen() {
    // A list of 70 lines, as the book of 70 chapters of the measurement holds.
    let the_lines: Vec<String> = (1..=70)
        .map(|number| format!("Chapter {} of the second book", number))
        .collect();

    // A panel of 12 rows: the border of the block, the row of the header, and
    // ten rows of the list.
    let area = Rect::new(0, 0, 80, 12);
    let mut buf = Buffer::empty(area);
    let mut list_state = ListState::default();
    list_state.select(Some(69));

    let the_lines_of_the_list = toutui::ui::the_list_of_a_view::render_the_list_with_a_header(
        area,
        &mut buf,
        &Colors::default(),
        "The chapters of a book",
        &the_lines,
        Some("     #  Title"),
        &mut list_state,
    );

    let the_offset = list_state.offset();

    assert!(
        the_offset > 0,
        "the render must write the offset of the screen into the state that it \
         takes: the cursor stands at the line 69 of 70 in a panel of ten rows, \
         therefore the first row of that panel holds no line 0 (T-330.5)"
    );

    // The row of the screen that holds the last line of the list.
    let the_row = the_lines_of_the_list.y + the_lines_of_the_list.height - 1;

    assert_eq!(
        the_line_of_a_row(the_lines_of_the_list, the_offset, the_lines.len(), the_row,),
        Some(69),
        "a click of the last row of the panel must name the line of the user, \
         which is the last line of the list (T-330.5)"
    );

    // **The fault of the measurement, in one line**: the offset 0 is the offset
    // of the state before the render, which the copy of `render_chapters` gave
    // the map of the mouse.
    assert_ne!(
        the_line_of_a_row(the_lines_of_the_list, 0, the_lines.len(), the_row),
        Some(69),
        "the offset 0 must name another line: that is the fault of the map of \
         the mouse of the Chapters view, and a test that gives the two offsets \
         one line measures nothing (T-330.5)"
    );
}

/// The footer of the Chapters view names the key of a chapter, and the click of
/// a row does the work of that key with no word of its own. See T-330.5.
#[test]
fn the_footer_of_the_view_keeps_the_key_of_a_chapter() {
    let source = include_str!("../src/ui/tui.rs");

    assert!(
        source.contains(r#"footer_with("go to the chapter", None)"#),
        "the footer of the Chapters view must name the work of the key of a \
         row: a footer must not promise a key that the view does not hold \
         (T-143), and the click of a row takes that same road (T-330.5)"
    );
}

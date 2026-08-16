//! The table of the panel 4, its header, and its columns. See T-321.
//!
//! **The maintainer chose the mockup 1, the panels, on 2026-08-16**, and the
//! stage 4 of that road is the table: the list of the panel 4 held one text of
//! each row, and the design holds a row of a header and the columns `Title`,
//! `Author`, `Time`, and `Done`.
//!
//! **The fault, of the real program v0.8.147 inside tmux**, of the Library view
//! of the library `Large` of the sandbox on a screen of 160 columns:
//!
//! ```text
//! ╔4 Library [500 items of 2056] ═══════════════════════════════════════════╗
//! ║➤     Large Book 2056                                                   █║
//! ║      Large Book 2055                                                   │║
//! ```
//!
//! The panel held 74 columns, the title took 17 of them, and the author, the
//! length, and the place of the user of each media stood in no column at all.
//!
//! **The correction, of the same harness**, of the library `Books`:
//!
//! ```text
//! ╔4 Library [18 items] ════════════════════════════════════════════════════╗
//! ║    Title                                Author               Time  Done ║
//! ║➤ ✓ A Book Of An Epub With No Container                        <1m  done█║
//! ║    A Big Book Of A Scan                 Big Author            <1m   90%█║
//! ║    A Book Of Many Hours                 Many Hours Author    8h00   96%│║
//! ║    The Test Chronicles [3 books]                                       │║
//! ```
//!
//! **The three shapes of the table** (the head of
//! `toutui::ui::the_table_of_a_view`): every column at 160 columns of the
//! screen, the author away at 120, and the list of today under the width of the
//! panel 4.

use ratatui::{buffer::Buffer, layout::Rect, widgets::ListState};
use toutui::config::Colors;
use toutui::ui::the_list_of_a_view::{render_the_table_of_a_panel, TheContentOfAPanel};
use toutui::ui::the_table_of_a_view::{
    the_columns_of_the_table, ARowOfTheTable, THE_AUTHOR, THE_DONE, THE_TIME, THE_TITLE,
};

/// The rows of the measurement of tmux of this round, of the library `Books` of
/// the sandbox.
fn the_rows() -> Vec<ARowOfTheTable> {
    vec![
        ARowOfTheTable {
            the_mark: "✓".to_string(),
            title: "A Book Of An Epub With No Container".to_string(),
            author: String::new(),
            time: "<1m".to_string(),
            done: "done".to_string(),
            the_whole_width: false,
        },
        ARowOfTheTable {
            the_mark: String::new(),
            title: "A Book Of Many Hours".to_string(),
            author: "Many Hours Author".to_string(),
            time: "8h00".to_string(),
            done: "96%".to_string(),
            the_whole_width: false,
        },
        ARowOfTheTable {
            title: "The Test Chronicles [3 books]".to_string(),
            the_whole_width: true,
            ..ARowOfTheTable::default()
        },
    ]
}

/// The lines of the list of today, which the panel draws when the table does
/// not stand.
fn the_lines() -> Vec<String> {
    vec![
        "✓   A Book Of An Epub With No Container".to_string(),
        "96% A Book Of Many Hours".to_string(),
        "    The Test Chronicles [3 books]".to_string(),
    ]
}

/// Draws the panel 4 of a width into a buffer, and gives the rows of it and the
/// row of the header.
fn the_screen_of_the_panel(width: u16, the_table: bool) -> (Vec<String>, Rect) {
    let area = Rect::new(0, 0, width, 8);
    let mut buf = Buffer::empty(area);

    let the_rows = the_rows();
    let mut state = ListState::default();
    state.select(Some(0));

    let (_, the_header) = render_the_table_of_a_panel(
        area,
        &mut buf,
        &Colors::default(),
        TheContentOfAPanel {
            the_panel: Some((4, true)),
            title: "Library [18 items]",
            lines: &the_lines(),
            the_rows: the_table.then_some(the_rows.as_slice()),
        },
        &mut state,
    );

    let the_screen = (0..area.height)
        .map(|row| {
            (0..area.width)
                .map(|column| buf[(column, row)].symbol().to_string())
                .collect::<String>()
        })
        .collect();

    (the_screen, the_header)
}

/// The table of the panel 4 holds a row of a header, and the words of every row
/// stand under the words of it. See T-321.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_table_of_the_panel_4_holds_a_row_of_a_header() {
    // The panel 4 of a screen of 160 columns held 74 columns in the measurement
    // of this round.
    let (the_screen, the_header) = the_screen_of_the_panel(74, true);

    // **The row of the header stands under the border of the panel**, and it
    // names the four columns of the design.
    let header = &the_screen[1];
    for name in [THE_TITLE, THE_AUTHOR, THE_TIME, THE_DONE] {
        assert!(
            header.contains(name),
            "the header of the table names no column {name}: {header}"
        );
    }

    // **The words of a row stand under the words of the header**: the column of
    // the author of the header starts where the author of the row starts.
    let the_row_of_many_hours = the_screen
        .iter()
        .find(|line| line.contains("A Book Of Many Hours"))
        .expect("the table drew no row of that media");

    assert_eq!(
        header.find(THE_AUTHOR),
        the_row_of_many_hours.find("Many Hours Author"),
        "the author of the row stands under no header: {the_row_of_many_hours}"
    );

    // **The numbers stand at the right of their column**, therefore the column
    // of the header ends where the column of the row ends.
    assert_eq!(
        header.find(THE_DONE).map(|at| at + THE_DONE.len()),
        the_row_of_many_hours.find("96%").map(|at| at + 3),
        "the place of the user stands at no column of a header"
    );

    // **The mark of the state holds no percent** (T-321): the column `Done`
    // says that number, and the mark of the table says the end of a media and
    // the media that plays alone.
    let the_row_of_the_end = the_screen
        .iter()
        .find(|line| line.contains("A Book Of An Epub With No Container"))
        .expect("the table drew no row of that media");
    assert!(the_row_of_the_end.contains('✓'));
    assert!(the_row_of_the_end.contains("done"));

    // **A row that names more than one media takes every column of the table**:
    // a series holds no author, no length, and no place of the user.
    let the_row_of_the_series = the_screen
        .iter()
        .find(|line| line.contains("The Test Chronicles"))
        .expect("the table drew no row of the series");
    assert!(!the_row_of_the_series.contains('%'));

    // **The row of the header is a target of the mouse** (T-316 and T-321): it
    // holds the whole width of the panel, and one row of the screen.
    assert_eq!(the_header.height, 1);
    assert_eq!(the_header.y, 1);
    assert!(the_header.width >= 70);
}

/// A panel that is too narrow for the columns of the table draws the list of
/// today, and a caller that gives no row of a table draws it too. See T-321.
///
/// **The parts of this test stay in one function.**
#[test]
fn a_panel_that_holds_no_table_draws_the_list_of_today() {
    // **The author goes away before the title of the media does**: the panel 4
    // of a screen of 120 columns held 50 columns in the measurement of this
    // round, and the column of the author would leave the title nine of them.
    let (the_screen, _) = the_screen_of_the_panel(50, true);
    let header = &the_screen[1];

    assert!(header.contains(THE_TITLE));
    assert!(header.contains(THE_TIME));
    assert!(header.contains(THE_DONE));
    assert!(
        !header.contains(THE_AUTHOR),
        "the author took the columns of the title: {header}"
    );

    // **A panel that holds no column of the place of the user holds no table at
    // all**, and the row of the header then holds no cell of the screen.
    let (the_screen, the_header) = the_screen_of_the_panel(28, true);
    assert_eq!(the_header, Rect::default());
    assert!(
        the_screen[1].contains("A Book Of An Epub"),
        "the panel drew no line of the list of today: {}",
        the_screen[1]
    );

    // **A caller that gives no row of a table draws the list of today at every
    // width**, which is the road of every view but the Home view and the
    // Library view.
    let (the_screen, the_header) = the_screen_of_the_panel(74, false);
    assert_eq!(the_header, Rect::default());
    assert!(
        the_screen[1].contains("A Book Of An Epub"),
        "the panel of a view with no table drew a header: {}",
        the_screen[1]
    );
    assert!(
        !the_screen[1].contains(THE_AUTHOR),
        "the panel of a view with no table drew a column of an author"
    );

    // The arithmetic of the columns says the same thing with no screen at all.
    assert!(the_columns_of_the_table(74 - 3).the_table_stands());
    assert!(!the_columns_of_the_table(20).the_table_stands());
}

//! The cell of the panel 6 holds the picture and its border alone. See T-330.4.
//!
//! **The maintainer read the program v0.8.158 and they gave six points**, and
//! the part 4 of the first of them is this one: a cell of the gallery held the
//! picture, a row of the percentage inside the border, and a row of the title
//! under the box (T-327). The maintainer read the two rows of words as noise,
//! because **the panel 5 says the facts of the media of the cursor already**
//! and the gallery is the picture.
//!
//! The measurement of the real program v0.8.162 inside tmux, of the Library
//! view of the library `Books` of the sandbox at 160 columns and 45 rows. The
//! panel 6 held 18 rows, and **two rows of every eight said words**:
//!
//! ```text
//! ┌6 Gallery ──────────────────────────────────────┐
//! │  ╔════════╗ ┌────────┐ ┌────────┐ ┌────────┐   │
//! │  ║        ║ │        │ │        │ │        │   │
//! │  ║        ║ │        │ │        │ │        │   │
//! │  ║        ║ │        │ │        │ │        │   │
//! │  ║        ║ │        │ │        │ │        │   │
//! │  ║  done  ║ │    -   │ │  done  │ │    -   │   │
//! │  ╚════════╝ └────────┘ └────────┘ └────────┘   │
//! │  A Book Of… A Book Of… A Book Of… A Very La…   │
//! │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐   │
//! │  │        │ │        │ │        │ │        │   │
//! │  │        │ │        │ │        │ │        │   │
//! │  │        │ │        │ │        │ │        │   │
//! │  │        │ │        │ │        │ │        │   │
//! │  │   90%  │ │    -   │ │  done  │ │   50%  │   │
//! │  └────────┘ └────────┘ └────────┘ └────────┘   │
//! │  A Big Boo… A Book Th… A Huge Bo… A Long Te…   │
//! └────────────────────────────────────────────────┘
//! ```
//!
//! The corrected program v0.8.163 of the same harness. The panel holds 20 rows,
//! **no character of a word at all**, and **three** rows of the grid where the
//! program before it held two:
//!
//! ```text
//! ┌6 Gallery ──────────────────────────────────────┐
//! │  ┏━━━━━━━━┓ ┌────────┐ ┌────────┐ ┌────────┐   │
//! │  ┃        ┃ │        │ │        │ │        │   │
//! │  ┃        ┃ │        │ │        │ │        │   │
//! │  ┃        ┃ │        │ │        │ │        │   │
//! │  ┃        ┃ │        │ │        │ │        │   │
//! │  ┗━━━━━━━━┛ └────────┘ └────────┘ └────────┘   │
//! │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐   │
//! │  │        │ │        │ │        │ │        │   │
//! │  │        │ │        │ │        │ │        │   │
//! │  │        │ │        │ │        │ │        │   │
//! │  │        │ │        │ │        │ │        │   │
//! │  └────────┘ └────────┘ └────────┘ └────────┘   │
//! │  ┌────────┐ … one row of the grid more …       │
//! └────────────────────────────────────────────────┘
//! ```
//!
//! The picture of a cover stands inside the box, and `tmux capture-pane` with
//! no `-e` gives no character of it. The same capture with `-e` gave the colour
//! `48;2;…` on eight rows of that panel, therefore the pictures stand.

use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;
use ratatui_image::FontSize;
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;
use toutui::ui::the_panel_of_the_cover::THE_ROWS_OF_THE_FACTS;
use toutui::ui::the_panel_of_the_gallery::{
    plan_the_gallery, the_rows_of_a_box, the_smallest_gallery, the_two_panels,
    THE_WIDTHS_OF_A_CELL, THE_WIDTH_OF_THE_START,
};

/// Nothing listens on this port, therefore every request fails at once and
/// `App::new` gives the offline mode. See T-25.
const NO_SERVER: &str = "http://127.0.0.1:1";

/// The font of the terminal of the measurement of this round: a cell of 10
/// pixels by 20.
const FONT: FontSize = FontSize {
    width: 10,
    height: 20,
};

/// Every character that a cell of the gallery may draw: the two shapes of a
/// border, and the space.
///
/// **A character that stands outside this list is a word**, and the panel 6
/// holds no word at all.
const THE_CHARACTERS_OF_A_CELL: &str = " │─┌┐└┘┃━┏┓┗┛";

fn a_user() -> User {
    User {
        server_address: NO_SERVER.to_string(),
        username: "toutuitest".to_string(),
        token: "not-a-real-token".to_string(),
        is_default_usr: true,
        name_selected_lib: "Books".to_string(),
        id_selected_lib: "a-library".to_string(),
        is_loop_break: "0".to_string(),
        has_played_before: "1".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    }
}

/// The rows of the panel 6 of a screen, from the row of its title to the row of
/// its last border.
fn the_panel_of_the_gallery(screen: &[String]) -> Vec<String> {
    let the_title = screen
        .iter()
        .position(|row| row.contains("6 Gallery"))
        .expect("the screen holds no panel 6 at all");

    // The panel ends at the row that holds the corner at its left.
    //
    // **`char_indices` gives the index of a byte and not the column of the
    // screen** (the trap 245): a row of this screen holds `│`, `║`, and `➤`,
    // and each of them takes three bytes and one column.
    let the_column = screen[the_title]
        .chars()
        .position(|one| one == '6')
        .expect("the row of the title holds no digit 6");

    let the_end = screen
        .iter()
        .enumerate()
        .skip(the_title + 1)
        .find(|(_, row)| row.chars().nth(the_column - 1) == Some('└'))
        .map(|(at, _)| at)
        .expect("the panel 6 holds no last row");

    // **The panel 6 is a part of a row of the screen and not the whole of
    // it**: the panels 1, 2, and 3 and the panel 4 of the list stand at the
    // left of every row of it, and their words belong to them.
    screen[the_title..=the_end]
        .iter()
        .map(|row| row.chars().skip(the_column - 1).collect())
        .collect()
}

/// The panel 6 of the real program draws the pictures and the borders of the
/// cells, and no word at all. See T-330.4.
///
/// **A cell that no picture reaches holds the title of its media** (T-339), and
/// that is no word of the design that this test takes away: the two rows of the
/// percentage and of the title stood **under** the box of every cell, and the
/// title of T-339 stands **inside** the border of a cell that draws no picture
/// at all. The media of this test carry an identity that no answer of the store
/// names, therefore `cover::no_picture_comes` gives `false` for each of them and
/// every cell of this measurement holds a picture.
///
/// **The parts of this test stay in one function**: the test writes
/// `XDG_CONFIG_HOME` of the process, and a second function of this binary would
/// fight it for that box.
///
/// **The runtime of this test holds one thread, and that is the measurement**
/// (T-341). `cover::request` marks a cover `Asked` and it gives the answer at
/// once, because the render is not asynchronous: the task that it spawns then
/// asks a port that no program holds, and the answer of that port is a fault
/// that comes in less than a millisecond. A runtime of many threads therefore
/// runs those tasks **while the frame draws**, and a cell that the render
/// reaches after the task of it came back reads `CoverBytes::Fault`,
/// `no_picture_comes` gives `true`, and the cell holds the title of its media
/// (T-339).
///
/// The measurement of 2026-08-17, of this test with 24 loops of a shell on the
/// processors of the machine: the run 5 of 6 failed with
/// `the panel 6 holds the character 'Z' of a word`, and the first cell of the
/// grid alone held `Zebra Book 1` while the eleven cells after it stood empty.
/// The same test of a clean machine passed 8 runs of the whole gate.
///
/// A runtime of one thread runs no task while the synchronous render stands,
/// therefore every cell of the frame reads the same state of the store.
#[tokio::test]
async fn no_cell_of_the_gallery_holds_a_word() {
    // No line of this test may touch the files of the user.
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();
    std::fs::copy(
        concat!(env!("CARGO_MANIFEST_DIR"), "/config.example.toml"),
        dir.path().join("toutui").join("config.toml"),
    )
    .unwrap();

    let conn = toutui::db::migrate::open_conn().unwrap();
    toutui::db::migrate::run_migrations(&conn).unwrap();
    drop(conn);

    toutui::db::crud::db_insert_usr(&vec![a_user()]).unwrap();

    let pool = EndpointPool::new(vec![Endpoint::new(NO_SERVER, 0)]);
    let api = Arc::new(ApiClient::new(Arc::new(pool), "token".to_string()).unwrap());
    let mut app = App::new(Arc::clone(&api)).await.expect("an application");

    // Twenty books of the library `Books`. **The title of each of them holds
    // the letters that the gate looks for**: a row of the title under the box
    // would write them in the panel 6.
    let titles: Vec<String> = (1..=20)
        .map(|number| format!("Zebra Book {number}"))
        .collect();
    let ids: Vec<String> = (1..=20).map(|number| format!("id-{number}")).collect();

    app.titles_library = titles.clone();
    app.ids_library = ids;
    app.auth_names_library = titles.iter().map(|_| "An Author".to_string()).collect();
    app.duration_library = titles.iter().map(|_| 3600.0).collect();
    app.published_year_library = Vec::new();
    app.desc_library = Vec::new();
    app.library_rows =
        toutui::logic::library_view::group_library(&app.ids_library, &app.series, false);
    app.view_state = AppView::Library;
    app.list_state_library.select(Some(0));

    // The terminal of the measurement: 160 columns and 45 rows, which is the
    // shape of the three columns of the design (T-320).
    let backend = TestBackend::new(160, 45);
    let mut terminal = Terminal::new(backend).expect("a terminal");

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Library view draws");

    let buffer = terminal.backend().buffer().clone();
    let screen: Vec<String> = (0..buffer.area.height)
        .map(|row| {
            (0..buffer.area.width)
                .map(|column| buffer[(column, row)].symbol())
                .collect()
        })
        .collect();

    let panel = the_panel_of_the_gallery(&screen);

    // The row of the title of the panel says `6 Gallery`, and every row under
    // it holds the borders of the cells and the border of the panel alone.
    for row in panel.iter().skip(1) {
        for one in row.chars() {
            assert!(
                THE_CHARACTERS_OF_A_CELL.contains(one),
                "the panel 6 holds the character {one:?} of a word:\n{}",
                panel.join("\n")
            );
        }
    }

    // **A gate that finds no word in a panel of no cell says nothing at all**:
    // the panel must hold the cells of the media too.
    let the_cells = panel
        .iter()
        .filter(|row| row.contains('┌') || row.contains('┏'))
        .count();
    assert!(
        the_cells >= 2,
        "the panel 6 holds {the_cells} rows of the tops of the cells:\n{}",
        panel.join("\n")
    );

    // **The border of the cell of the cursor is heavy, and the border of every
    // other cell is thin** (T-330.4): a colour alone is not the mark of the
    // focus, and a terminal of a theme of few colours draws the two of them
    // near together.
    assert!(
        panel.iter().any(|row| row.contains('┏')),
        "the cell of the cursor holds no heavy border:\n{}",
        panel.join("\n")
    );
    assert!(
        panel.iter().any(|row| row.contains('┌')),
        "no cell of the gallery holds a thin border:\n{}",
        panel.join("\n")
    );

    // The control of the same run: the panel 4 of the list holds the titles of
    // the same media, therefore the words of a media stand in the screen and
    // the gate reads a panel that lost them alone.
    let text = screen.join("\n");
    assert!(
        text.contains("Zebra Book 1"),
        "the panel 4 of the list lost the titles of the media"
    );
}

/// A box of a cell holds the picture and the two rows of its border, and no row
/// of a word at all. See T-330.4.
///
/// **The parts of this test stay in one function.**
#[test]
fn a_box_of_a_cell_holds_the_picture_and_its_border_alone() {
    for of_a_cell in THE_WIDTHS_OF_A_CELL {
        let the_box = the_rows_of_a_box(of_a_cell, FONT);

        // The panel of the smallest grid holds one box and the border of the
        // panel, and no row more.
        assert_eq!(
            the_smallest_gallery(of_a_cell, FONT),
            the_box + 2,
            "the smallest gallery of a cell of {of_a_cell} columns holds a row that no part uses"
        );

        let inside = Rect::new(111, 27, 48, the_box * 3);
        let plan = plan_the_gallery(inside, of_a_cell, FONT, 500, 0);

        for cell in &plan.cells {
            // **The picture holds every row of the box that the border
            // leaves**: a row of the percentage would stand between the picture
            // and the border under it.
            assert_eq!(
                cell.the_picture.height + 2,
                cell.the_box.height,
                "the cell {cell:?} holds a row that the picture does not use"
            );
            assert_eq!(cell.the_picture.y, cell.the_box.y + 1);
            assert_eq!(cell.the_box.height, the_box);

            // Every cell stands inside the panel: no row of a title stands
            // under the box, therefore the box is the whole of the cell.
            assert_eq!(
                inside.union(cell.the_box),
                inside,
                "{cell:?} left the panel"
            );
        }

        // **The rows of the grid stand one after the other**: the row of the
        // title between two of them went away with the words.
        let of_the_first = plan.cells[0].the_box;
        let below = plan
            .cells
            .iter()
            .find(|cell| cell.the_box.x == of_the_first.x && cell.the_box.y > of_the_first.y)
            .expect("the grid of three rows holds a cell under the first one");

        assert_eq!(below.the_box.y, of_the_first.y + of_the_first.height);
    }
}

/// The rows that the words gave back go to the pictures: the column of the
/// measurement holds one row of the grid more. See T-330.4.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_column_of_the_measurement_holds_one_row_of_the_grid_more() {
    // The measurement of this round: the column of the covers of a screen of
    // 160 by 45 stands at the column 111 and it holds 41 rows.
    let column = Rect::new(111, 2, 50, 41);
    let of_a_cell = THE_WIDTHS_OF_A_CELL[THE_WIDTH_OF_THE_START];

    let (cover, gallery) = the_two_panels(column, of_a_cell, FONT, true, THE_ROWS_OF_THE_FACTS);
    let gallery = gallery.expect("a column of 41 rows holds the panel 5 and the panel 6");

    // The panel 6 holds whole rows of the grid alone (T-327).
    let of_a_row = the_rows_of_a_box(of_a_cell, FONT);
    assert_eq!((gallery.height - 2) % of_a_row, 0);

    // **The program v0.8.162 held two rows of the grid and eight cells**: a row
    // of the grid took the box of six rows, a row of the percentage, and a row
    // of the title. The program of this part holds three rows of the grid.
    assert_eq!(
        (gallery.height - 2) / of_a_row,
        3,
        "the panel 6 of {} rows holds {} rows of the grid",
        gallery.height,
        (gallery.height - 2) / of_a_row
    );

    let inside = Rect::new(
        gallery.x + 1,
        gallery.y + 1,
        gallery.width - 2,
        gallery.height - 2,
    );
    let plan = plan_the_gallery(inside, of_a_cell, FONT, 500, 0);

    assert_eq!(plan.the_rows, 3);
    assert_eq!(plan.the_columns, 4);
    assert_eq!(plan.cells.len(), 12);

    // The panel 5 keeps every row that the gallery does not take.
    assert_eq!(cover.height + gallery.height, column.height);
    assert!(
        cover.height >= toutui::ui::the_panel_of_the_cover::THE_SMALLEST_PANEL_OF_THE_WORDS + 2
    );
}

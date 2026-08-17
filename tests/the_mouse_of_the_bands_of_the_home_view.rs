//! The mouse of the bands of covers of the Home view. See T-331 and T-337.
//!
//! The maintainer asked for this view on 2026-08-16, and
//! `docs/superpowers/specs/2026-08-17-the-home-view-of-the-bands-of-covers-design.md`
//! holds the design of it. T-335 gave the bands of the flat list, T-336 drew
//! them in the panel 4 with their keys; **this round gives them the mouse**, and
//! it is the round 3 of the road of that design.
//!
//! **The three faults of the real program v0.8.168 inside tmux**, of 160 columns
//! and 45 rows, of the Home view of the library `Books` of the sandbox, with
//! `docs/harness/click.sh`:
//!
//! 1. **A click of a cell of a band moved the cursor nowhere.** A click of the
//!    fourth cell of the band `Continue Listening`, at the column 72 and the row
//!    7, left the heavy border on the first cell and it left the facts of the
//!    panel 5 at the media of that first cell. **The reason**: the render of the
//!    bands gives the areas of the mouse no line of a list at all, because a
//!    cell of a band holds six rows and ten columns of the screen and the
//!    arithmetic of a row of a list therefore says nothing of it.
//! 2. **One step of the wheel over a band moved the cursor of another band.** A
//!    step over the band `Recently Added`, at the column 60 and the row 13,
//!    moved the cursor from the first cell of `Continue Listening` to the second
//!    one, and the band under the pointer did not move at all: it said
//!    `6 of 10 ›` before that step and after it.
//! 3. **A click of the row of the title of a band did nothing.**
//!
//! **The screen of the correction, of the same harness and of the same run.**
//! The click of the column 72 and the row 7:
//!
//! ```text
//! ║Continue Listening ─────────────────────────────────────────────── 6 of 6║
//! ║┌────────┐ ┌────────┐ ┌────────┐ ┏━━━━━━━━┓ ┌────────┐ ┌────────┐        ║
//! ```
//!
//! Two steps of the wheel over the band `Recently Added` gave
//! `Recently Added ─── ‹ 6 of 10 ›`, with the arrow at the left of a band that
//! moved and with the cursor of the band above where it stood. A click of the
//! row of the title of the band `Discover` took the cursor to the first cell of
//! that band. **Two clicks of one cell inside 400 milliseconds** said
//! `Loading the media...`, which is the work of the key `Enter` of that cell.
//!
//! **This test needs no sandbox and no server.** `App::new` takes a port that
//! nothing listens on, therefore it gives the offline mode (T-25).
//!
//! **The parts of this test stay in one function**: two test functions of one
//! binary take a thread each, and `cargo test` finds a fault of that shape at
//! one run of six (T-144 and T-157).

use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::utils::collect_series::{SeriesBookView, SeriesView};
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;
use toutui::logic::home_view::HomeRow;
use toutui::ui::the_panel_of_the_bands::plan_the_bands;

/// Nothing listens on this port.
const NO_SERVER: &str = "http://127.0.0.1:1";

/// The panel 4 of the measurement of tmux: 71 columns and 39 rows inside the
/// border of it.
const INSIDE: Rect = Rect {
    x: 2,
    y: 3,
    width: 71,
    height: 39,
};

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

/// A report of the button at the left, of one press.
fn a_click(column: u16, row: u16) -> MouseEvent {
    MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

/// A report of one step of the wheel.
fn a_step_of_the_wheel(column: u16, row: u16, forward: bool) -> MouseEvent {
    MouseEvent {
        kind: if forward {
            MouseEventKind::ScrollDown
        } else {
            MouseEventKind::ScrollUp
        },
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }
}

/// The shelves of the measurement, in the shape of the flat list: a shelf of
/// twelve media, and a shelf of one series.
fn the_rows() -> Vec<HomeRow> {
    let mut rows = vec![HomeRow::Shelf {
        label: "Continue Listening".to_string(),
    }];

    for item in 0..12 {
        rows.push(HomeRow::Media { item });
    }

    rows.push(HomeRow::Shelf {
        label: "Recent Series".to_string(),
    });
    rows.push(HomeRow::Series { series: 0 });

    rows
}

#[tokio::test(flavor = "multi_thread")]
async fn the_mouse_of_the_bands_of_the_home_view() {
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

    app.is_podcast = false;
    app.view_state = AppView::Home;
    app.home_rows = the_rows();
    app._ids_cnt_list = (0..13).map(|one| format!("the-media-{}", one)).collect();
    app.series = vec![SeriesView {
        id: "a-series".to_string(),
        name: "A Series".to_string(),
        description: String::new(),
        books: vec![SeriesBookView {
            id: "a-book".to_string(),
            title: "A Book Of A Series".to_string(),
            author: "An Author".to_string(),
            sequence: "1".to_string(),
            duration: 60.0,
            description: String::new(),
        }],
    }];
    app.list_state_cnt_list.select(Some(1));
    app.the_panel_of_the_focus = toutui::ui::frame::ThePanel::TheList;
    app.the_mouse_stands = true;

    // **The areas of the mouse are the areas of the last frame**: the panel 4 of
    // the measurement of tmux held 73 columns and 41 rows with its border, and
    // the render of the bands gives that panel no line of a list at all.
    app.the_areas_of_the_mouse.the_panel_of_the_list = Rect {
        x: INSIDE.x - 1,
        y: INSIDE.y - 1,
        width: INSIDE.width + 2,
        height: INSIDE.height + 2,
    };
    app.the_areas_of_the_mouse.the_lines_of_the_list = Rect::default();
    app.the_areas_of_the_mouse.the_lines = 0;

    // The frame of the last render says which shape the panel drew (T-336).
    let a_frame = |app: &mut App| {
        let bands = toutui::logic::the_bands_of_the_home::the_bands(&app.home_rows);
        let the_line = app.list_state_cnt_list.selected().unwrap_or(0);
        let plan = plan_the_bands(
            INSIDE,
            10,
            (10, 20).into(),
            &bands,
            the_line,
            &app.the_offsets_of_the_bands,
        );

        for band in &plan.bands {
            app.the_offset_of_a_band_goes_to(band.the_band, band.the_first_cell);
        }

        app.the_bands_of_the_last_frame = plan;
    };

    a_frame(&mut app);
    assert!(
        app.the_bands_of_the_last_frame.stands(),
        "the panel of the measurement of tmux draws bands"
    );

    // **A click of a cell moves the cursor to the media of that cell** (T-337).
    // The fourth cell of the first band holds the line 4 of the flat list.
    let of_the_cell = app.the_bands_of_the_last_frame.bands[0].cells[3].the_box;
    app.handle_the_mouse(a_click(of_the_cell.x + 2, of_the_cell.y + 2));
    assert_eq!(
        app.list_state_cnt_list.selected(),
        Some(4),
        "the click of the fourth cell gives the line of that cell"
    );

    // **A click of the row of the title of a band takes the first cell that the
    // band draws**, and it opens nothing at all.
    a_frame(&mut app);
    let of_the_title = app.the_bands_of_the_last_frame.bands[1].the_title;
    app.handle_the_mouse(a_click(of_the_title.x + 2, of_the_title.y));
    assert_eq!(
        app.list_state_cnt_list.selected(),
        Some(14),
        "the click of the title of the second band takes the first cell of it"
    );
    assert_eq!(
        app.view_state,
        AppView::Home,
        "one click of a title opens no view"
    );

    // **One step of the wheel over a band moves that band by one cell**, and it
    // moves the cursor of no other band. The cursor stands in the second band
    // now, and the first band holds twelve cells of six on the screen.
    a_frame(&mut app);
    let of_the_band = app.the_bands_of_the_last_frame.bands[0].the_rows;
    app.handle_the_mouse(a_step_of_the_wheel(
        of_the_band.x + 40,
        of_the_band.y + 2,
        true,
    ));
    assert_eq!(
        app.the_offset_of_a_band(0),
        1,
        "the step of the wheel moves the band under the pointer by one cell"
    );
    assert_eq!(
        app.list_state_cnt_list.selected(),
        Some(14),
        "the wheel over a band that holds no cursor moves no cursor"
    );

    a_frame(&mut app);
    assert_eq!(
        app.the_bands_of_the_last_frame.bands[0].cells[0].the_line, 2,
        "the band of the offset 1 starts at its second cell"
    );
    assert!(
        app.the_bands_of_the_last_frame.bands[0].at_the_left,
        "the band that moved says the arrow at the left"
    );

    // **The wheel back moves that band back**, and it stops at the first cell.
    for _ in 0..4 {
        app.handle_the_mouse(a_step_of_the_wheel(
            of_the_band.x + 40,
            of_the_band.y + 2,
            false,
        ));
        a_frame(&mut app);
    }
    assert_eq!(
        app.the_offset_of_a_band(0),
        0,
        "the wheel back stops at the first cell of the band"
    );

    // **The cursor keeps the screen**: the cursor stands in the first band now,
    // and the wheel over that band takes it with the band.
    app.list_state_cnt_list.select(Some(1));
    a_frame(&mut app);
    app.handle_the_mouse(a_step_of_the_wheel(
        of_the_band.x + 40,
        of_the_band.y + 2,
        true,
    ));
    assert_eq!(
        app.the_offset_of_a_band(0),
        1,
        "the band of the cursor moves too"
    );
    assert_eq!(
        app.list_state_cnt_list.selected(),
        Some(2),
        "the cursor takes the cell of the edge that the band left behind"
    );

    // **The wheel stops at the last cell of a band**: the band holds twelve
    // cells and the panel draws six of them.
    for _ in 0..20 {
        app.handle_the_mouse(a_step_of_the_wheel(
            of_the_band.x + 40,
            of_the_band.y + 2,
            true,
        ));
        a_frame(&mut app);
    }
    assert_eq!(
        app.the_offset_of_a_band(0),
        6,
        "the wheel stops at the last cells of the band"
    );

    // **Two clicks of one cell play that media or they open it** (T-337), which
    // is the work of the key `Enter`. The one cell of the second band is a
    // series, and the key `Enter` of a series opens the books of it (T-22).
    app.list_state_cnt_list.select(Some(1));
    a_frame(&mut app);
    let of_the_series = app.the_bands_of_the_last_frame.bands[1].cells[0].the_box;
    app.handle_the_mouse(a_click(of_the_series.x + 2, of_the_series.y + 2));
    assert_eq!(
        app.view_state,
        AppView::Home,
        "one click of a cell opens nothing, which is the rule of T-316"
    );
    assert_eq!(
        app.list_state_cnt_list.selected(),
        Some(14),
        "one click of a cell takes the media of it"
    );

    app.handle_the_mouse(a_click(of_the_series.x + 2, of_the_series.y + 2));
    assert_eq!(
        app.view_state,
        AppView::SeriesBook,
        "the two clicks of one cell do the work of the key Enter"
    );

    // **A third click starts a first click again**: a user who clicks three
    // times plays the media one time.
    app.view_state = AppView::Home;
    a_frame(&mut app);
    app.handle_the_mouse(a_click(of_the_series.x + 2, of_the_series.y + 2));
    assert_eq!(
        app.view_state,
        AppView::Home,
        "the click after the two clicks of a cell opens nothing"
    );

    // **Two clicks of two cells are two first clicks**: a user who clicks one
    // cover and then another one asked for the two covers.
    a_frame(&mut app);
    let of_a_cover = app.the_bands_of_the_last_frame.bands[0].cells[0].the_box;
    app.handle_the_mouse(a_click(of_a_cover.x + 2, of_a_cover.y + 2));
    app.handle_the_mouse(a_click(of_the_series.x + 2, of_the_series.y + 2));
    assert_eq!(
        app.view_state,
        AppView::Home,
        "the two clicks of two cells open nothing"
    );

    // **A panel that drew the table of today keeps the mouse of that table**
    // (the decision 5 of the maintainer): the click of a row reads the line of
    // that row, and the wheel moves the cursor of the list.
    app.the_bands_of_the_last_frame = Default::default();
    app.the_areas_of_the_mouse.the_lines_of_the_list = Rect {
        x: INSIDE.x,
        y: INSIDE.y,
        width: INSIDE.width,
        height: INSIDE.height,
    };
    app.the_areas_of_the_mouse.the_lines = app.home_rows.len();
    app.list_state_cnt_list.select(Some(1));
    app.handle_the_mouse(a_click(INSIDE.x + 4, INSIDE.y + 3));
    assert_eq!(
        app.list_state_cnt_list.selected(),
        Some(3),
        "the click of a row of the table of today reads the line of that row"
    );
}

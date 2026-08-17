//! The gate of the round 5 of the Home view of the bands of covers. See T-339.
//!
//! `docs/superpowers/specs/2026-08-17-the-home-view-of-the-bands-of-covers-design.md`
//! holds the design, and this file is the gate of the last round of its road.
//! The round gives two rules, and each of them had a measurement of the real
//! program v0.8.170 inside tmux of 160 columns and 45 rows.
//!
//! ## 1. A cell of no picture held a border and nothing at all
//!
//! The Home view of the library `Large` of the sandbox. The server holds no
//! cover of those books, therefore `cover::no_picture_comes` gives `true` for
//! each of them, and the twelve cells of the panel 4 and the twelve cells of
//! the panel 6 stood empty together: **the user read no name of a media in the
//! whole of the two panels.**
//!
//! ```text
//! ║Recently Added ──────────────────────────────────────────────── 6 of 10 ›║
//! ║┏━━━━━━━━┓ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐        ║
//! ║┃        ┃ │        │ │        │ │        │ │        │ │        │        ║
//! ║┃        ┃ │        │ │        │ │        │ │        │ │        │        ║
//! ║┗━━━━━━━━┛ └────────┘ └────────┘ └────────┘ └────────┘ └────────┘        ║
//! ```
//!
//! The corrected program of the same harness:
//!
//! ```text
//! ║Recently Added ──────────────────────────────────────────────── 6 of 10 ›║
//! ║┏━━━━━━━━┓ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐        ║
//! ║┃Large   ┃ │Large   │ │Large   │ │Large   │ │Large   │ │Large   │        ║
//! ║┃Book    ┃ │Book    │ │Book    │ │Book    │ │Book    │ │Book    │        ║
//! ║┃0001    ┃ │0002    │ │0003    │ │0004    │ │0005    │ │0006    │        ║
//! ║┗━━━━━━━━┛ └────────┘ └────────┘ └────────┘ └────────┘ └────────┘        ║
//! ```
//!
//! **A cell that a picture reaches keeps the picture alone** (T-330.4): the
//! measurement of the library `Books` of the same round gave the two shapes in
//! one band, because the sandbox holds a cover for some of those media and for
//! not every one of them.
//!
//! ## 2. The panel 6 held the media of every shelf
//!
//! The same view, with the cursor on the **first** media of the shelf
//! `Discover`. The grid of the panel 6 held the twenty media of the two
//! shelves of the view, therefore the cell of the cursor stood in the third row
//! of it, at the place 10 of 20:
//!
//! ```text
//! │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐   │
//! │  └────────┘ └────────┘ └────────┘ └────────┘   │
//! │  ┌────────┐ ┌────────┐ ┏━━━━━━━━┓ ┌────────┐   │
//! │  └────────┘ └────────┘ ┗━━━━━━━━┛ └────────┘   │
//! ```
//!
//! The corrected program gave the ten media of `Discover` alone, with the
//! cursor at the first cell of the grid, and the titles of the correction 1 say
//! which shelf it is: `1376`, `1459`, `0984`, `1038`, `1476`, `0392`, `1931`,
//! `0754`, `0077`, and `0625`, which are the ten cells of that band.
//!
//! **The parts of this test stay in one function**: the test writes
//! `XDG_CONFIG_HOME` of the process, and a second
//! function of this binary would fight it for that box (T-144 and T-157).
//!
//! **This test needs no sandbox and no server.** `App::new` takes a port that
//! nothing listens on, therefore it gives the offline mode (T-25).

use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;
use toutui::logic::home_view::HomeRow;
use toutui::ui::the_panel_of_the_gallery::the_title_of_a_cell;

/// Nothing listens on this port. See T-25.
const NO_SERVER: &str = "http://127.0.0.1:1";

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

/// The two shelves of the measurement: three media of `Continue Listening`, and
/// four of `Discover`.
fn the_rows() -> Vec<HomeRow> {
    let mut rows = vec![HomeRow::Shelf {
        label: "Continue Listening".to_string(),
    }];

    for item in 0..3 {
        rows.push(HomeRow::Media { item });
    }

    rows.push(HomeRow::Shelf {
        label: "Discover".to_string(),
    });

    for item in 3..7 {
        rows.push(HomeRow::Media { item });
    }

    rows
}

/// The rows of the panel 6 of a screen, from the row of its title to the row of
/// its last border.
///
/// **`char_indices` gives the index of a byte and not the column of the
/// screen** (the trap 245): a row of this screen holds `│`, `║`, and `➤`, and
/// each of them takes three bytes and one column.
fn the_panel_of_the_gallery(screen: &[String]) -> Vec<String> {
    let the_title = screen
        .iter()
        .position(|row| row.contains("6 Gallery"))
        .expect("the screen holds no panel 6 at all");

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

    screen[the_title..=the_end]
        .iter()
        .map(|row| row.chars().skip(the_column - 1).collect())
        .collect()
}

/// The rows of the panel 4 of a screen: the rows of the bands of the covers.
fn the_panel_of_the_list(screen: &[String]) -> Vec<String> {
    let the_title = screen
        .iter()
        .position(|row| row.contains("4 Home"))
        .expect("the screen holds no panel 4 at all");

    let the_column = screen[the_title]
        .chars()
        .position(|one| one == '4')
        .expect("the row of the title holds no digit 4");

    // The panel 6 stands at the right of the panel 4, therefore the rows of the
    // panel 4 end at the border at the right of it.
    let the_last = screen[the_title]
        .chars()
        .skip(the_column)
        .position(|one| one == '╗')
        .expect("the row of the title holds no corner at its right")
        + the_column;

    screen
        .iter()
        .skip(the_title)
        .map(|row| {
            row.chars()
                .skip(the_column - 1)
                .take(the_last - the_column + 2)
                .collect()
        })
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_cell_that_draws_no_picture_says_its_media() {
    // **The pure function first**: the title takes the rows of the picture with
    // the wrap of this program, and a title that needs more rows loses its end
    // to the three points (T-91).
    let of_the_picture = Rect {
        x: 0,
        y: 0,
        width: 8,
        height: 4,
    };

    assert_eq!(
        the_title_of_a_cell("Large Book 0001", of_the_picture),
        "Large Book 0001",
        "a title of three rows of eight columns stands whole"
    );
    assert!(
        the_title_of_a_cell("A Book That Ends Before Its End Of A Name", of_the_picture)
            .ends_with('…'),
        "a title that needs more rows than the cell holds says that the screen cut it"
    );
    assert_eq!(
        the_title_of_a_cell(
            "Large Book 0001",
            Rect {
                width: 0,
                ..of_the_picture
            }
        ),
        "",
        "a cell of no column holds no title"
    );
    assert_eq!(
        the_title_of_a_cell(
            "Large Book 0001",
            Rect {
                height: 0,
                ..of_the_picture
            }
        ),
        "",
        "a cell of no row holds no title"
    );

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

    // **The title of each media names its shelf**, therefore a word of the
    // screen says which shelf the panel 6 draws.
    let titles: Vec<String> = (0..3)
        .map(|number| format!("Alpha Book {number}"))
        .chain((0..4).map(|number| format!("Omega Book {number}")))
        .collect();

    app.is_podcast = false;
    app.view_state = AppView::Home;
    app.home_rows = the_rows();
    // **A media of no identity is the condition of a cell that no picture
    // reaches**, and `cover::no_picture_comes` gives `true` for it with no
    // store and no request of the server at all. The other two roads of that
    // function are `TOUTUI_NO_COVERS`, which takes the whole column of the
    // panels 5 and 6 away (`cover::split_for_covers`), and a media of the store
    // of the state `NoCover`, which needs an answer of a server.
    app._ids_cnt_list = vec![String::new(); 7];
    app._titles_cnt_list = titles.clone();
    app.list_state_cnt_list.select(Some(1));

    // **The gallery of the Home view is the shelf of the cursor alone**
    // (T-339). The cursor stands on the first media of `Continue Listening`.
    let the_media = app.the_media_of_the_gallery();

    assert_eq!(
        the_media.len(),
        3,
        "the shelf of the cursor holds three media, and the view holds seven"
    );
    assert_eq!(
        the_media
            .iter()
            .map(|one| one.the_title.as_str())
            .collect::<Vec<_>>(),
        vec!["Alpha Book 0", "Alpha Book 1", "Alpha Book 2"],
        "the media of the gallery carry the title of each media"
    );
    assert_eq!(
        the_media.iter().map(|one| one.the_line).collect::<Vec<_>>(),
        vec![1, 2, 3],
        "the line of the flat list stays the road back of a click of a cell"
    );
    assert_eq!(
        app.the_media_of_the_cursor_of_the_gallery(&the_media),
        0,
        "the cursor of the first media of the shelf is the first cell of the grid"
    );

    // **The cursor of another shelf gives that shelf**: the line 5 is the
    // second media of `Discover`.
    app.list_state_cnt_list.select(Some(5));
    let the_media = app.the_media_of_the_gallery();

    assert_eq!(
        the_media
            .iter()
            .map(|one| one.the_title.as_str())
            .collect::<Vec<_>>(),
        vec![
            "Omega Book 0",
            "Omega Book 1",
            "Omega Book 2",
            "Omega Book 3"
        ],
        "the gallery of a cursor of the second shelf holds that shelf alone"
    );
    assert_eq!(
        app.the_media_of_the_cursor_of_the_gallery(&the_media),
        0,
        "the first media of the shelf of the cursor is the first cell of the grid"
    );

    // **A cursor that stands on the line of a shelf gives the first band**,
    // which is the rule of `plan_the_bands` (T-336).
    app.list_state_cnt_list.select(Some(0));
    assert_eq!(
        app.the_media_of_the_gallery()
            .first()
            .map(|one| one.the_title.clone()),
        Some("Alpha Book 0".to_string()),
        "a cursor on the row of a shelf gives the first band of the view"
    );

    // The terminal of the measurement: 160 columns and 45 rows, which is the
    // shape of the three columns of the design (T-320).
    app.list_state_cnt_list.select(Some(1));

    let backend = TestBackend::new(160, 45);
    let mut terminal = Terminal::new(backend).expect("a terminal");

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the Home view draws");

    let buffer = terminal.backend().buffer().clone();
    let screen: Vec<String> = (0..buffer.area.height)
        .map(|row| {
            (0..buffer.area.width)
                .map(|column| buffer[(column, row)].symbol())
                .collect()
        })
        .collect();

    // **The cells of the bands of the panel 4 hold the titles of their media.**
    let of_the_list = the_panel_of_the_list(&screen).join("\n");

    assert!(
        of_the_list.contains("Alpha"),
        "a cell of the band of the first shelf says its media:\n{of_the_list}"
    );
    assert!(
        of_the_list.contains("Omega"),
        "a cell of the band of the second shelf says its media:\n{of_the_list}"
    );

    // **The panel 6 holds the shelf of the cursor and no other shelf.**
    let of_the_gallery = the_panel_of_the_gallery(&screen).join("\n");

    assert!(
        of_the_gallery.contains("Alpha"),
        "the panel 6 says the media of the shelf of the cursor:\n{of_the_gallery}"
    );
    assert!(
        !of_the_gallery.contains("Omega"),
        "the panel 6 holds no media of another shelf:\n{of_the_gallery}"
    );
}

//! The keys and the footer of the Home view of the bands of covers. See T-331
//! and T-336.
//!
//! The maintainer asked for this view on 2026-08-16, and
//! `docs/superpowers/specs/2026-08-17-the-home-view-of-the-bands-of-covers-design.md`
//! holds the design of it. T-335 gave the bands of the flat list and the moves
//! over them; **this round draws them in the panel 4, and it gives them their
//! keys and their footer**.
//!
//! **The screen of the fault, of the real program v0.8.167 inside tmux** of 160
//! columns and 45 rows, of the library `Books` of the sandbox:
//!
//! ```text
//! ╔4 Home [35 items] ═══════════════════════════════════════════════════════╗
//! ║    Title                                 Author               Time  Done║
//! ║  ▌ Continue Listening                                                   ║
//! ║➤   A Long Test Book                      Long Author           30m   54%║
//! ║    A Second Book Of Many Hours           Many Hours Author    8h00   63%║
//!
//! j/k: move  l: play or open  Tab: home/library  …
//! ```
//!
//! **The screen of the correction, of the same harness**:
//!
//! ```text
//! ╔4 Home [35 items] ═══════════════════════════════════════════════════════╗
//! ║Continue Listening ─────────────────────────────────────────────── 5 of 5║
//! ║┏━━━━━━━━┓ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐                   ║
//! ║┃        ┃ │        │ │        │ │        │ │        │                   ║
//! ║┗━━━━━━━━┛ └────────┘ └────────┘ └────────┘ └────────┘                   ║
//! ║                                                                         ║
//! ║Recently Added ──────────────────────────────────────────────── 6 of 10 ›║
//!
//! j/k: a shelf  h/l: a cover  Enter: play or open  Tab: home/library  …
//! ```
//!
//! **The two faults that this test holds.**
//!
//! 1. **The key `l` played the media of the cursor**, and the bands need it for
//!    the cell at the right (the decision 2 of the design): a key of two
//!    meanings in one view is a fault of its own. The key `Enter` plays or
//!    opens, and it is an alias of `l` in every other view of this program
//!    already, therefore no other view changes.
//! 2. **The footer said `l: play or open`**, which is a key that the view no
//!    longer holds: a footer must not promise a key that the view does not have
//!    (T-143).
//!
//! **A panel that has no room for one whole band keeps the two of them**: the
//! table of today stands there, and `l` plays and the footer says so. The
//! measurement of tmux of this round, of a screen of 160 columns and 16 rows,
//! gave that table and that footer.
//!
//! **This test needs no sandbox and no server.** `App::new` takes a port that
//! nothing listens on, therefore it gives the offline mode (T-25).
//!
//! **The parts of this test stay in one function**: two test functions of one
//! binary take a thread each, and `cargo test` finds a fault of that shape at
//! one run of six (T-144 and T-157).

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;
use toutui::logic::home_view::HomeRow;
use toutui::ui::the_panel_of_the_bands::{plan_the_bands, TheBandsOfThePanel};

/// Nothing listens on this port.
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

fn a_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::NONE,
        kind: KeyEventKind::Press,
        state: ratatui::crossterm::event::KeyEventState::NONE,
    }
}

/// The shelves of the library `Books` of the sandbox, in the shape of the flat
/// list: `Continue Listening` of five media, and `Discover` of two.
fn the_rows() -> Vec<HomeRow> {
    let mut rows = vec![HomeRow::Shelf {
        label: "Continue Listening".to_string(),
    }];

    for item in 0..5 {
        rows.push(HomeRow::Media { item });
    }

    rows.push(HomeRow::Shelf {
        label: "Discover".to_string(),
    });
    rows.push(HomeRow::Media { item: 5 });
    rows.push(HomeRow::Media { item: 6 });

    rows
}

#[tokio::test(flavor = "multi_thread")]
async fn the_bands_of_the_home_view_hold_their_keys() {
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
    app._ids_cnt_list = (0..7).map(|one| format!("the-media-{}", one)).collect();
    app.list_state_cnt_list.select(Some(1));
    app.the_panel_of_the_focus = toutui::ui::frame::ThePanel::TheList;

    // **The frame of the last render says which shape the panel drew** (T-336),
    // and the keys read it: the panel 4 of the measurement of tmux of this
    // round held 71 columns and 39 rows, and a cell of ten columns of a font of
    // ten by twenty gives six cells of a band.
    let of_the_panel = ratatui::layout::Rect {
        x: 2,
        y: 3,
        width: 71,
        height: 39,
    };
    let bands = toutui::logic::the_bands_of_the_home::the_bands(&app.home_rows);
    app.the_bands_of_the_last_frame = plan_the_bands(of_the_panel, 10, (10, 20).into(), &bands, 1);

    assert!(
        app.the_bands_of_the_last_frame.stands(),
        "the panel of the measurement of tmux draws bands"
    );
    assert_eq!(
        app.the_bands_of_the_last_frame.the_cells_of_a_band, 6,
        "the panel of 71 columns holds six cells of a band"
    );

    // **The key `l` moves to the cell at the right, and it plays no media**
    // (the decision 2 of the design).
    app.handle_key(a_key(KeyCode::Char('l')));
    assert_eq!(
        app.list_state_cnt_list.selected(),
        Some(2),
        "the key l gives the cell at the right"
    );

    // **The move stops at the last cell of the band**, and it does not go to the
    // band under it (the decision 3).
    for _ in 0..8 {
        app.handle_key(a_key(KeyCode::Char('l')));
    }
    assert_eq!(
        app.list_state_cnt_list.selected(),
        Some(5),
        "the key l stops at the last cell of the band"
    );

    // **The key `h` gives the cell at the left, and it stops at the first one.**
    for _ in 0..8 {
        app.handle_key(a_key(KeyCode::Char('h')));
    }
    assert_eq!(
        app.list_state_cnt_list.selected(),
        Some(1),
        "the key h stops at the first cell of the band"
    );

    // **The key `j` gives the band under, and the cell keeps its number**; the
    // last cell of a shorter band takes it.
    app.handle_key(a_key(KeyCode::Char('l')));
    app.handle_key(a_key(KeyCode::Char('l')));
    app.handle_key(a_key(KeyCode::Char('j')));
    assert_eq!(
        app.list_state_cnt_list.selected(),
        Some(8),
        "the band under holds two cells, and the cell 2 of the band above takes the last of them"
    );

    // **The move of the bands goes round** (the rule of `next_line` of today).
    app.handle_key(a_key(KeyCode::Char('j')));
    assert_eq!(
        app.list_state_cnt_list.selected(),
        Some(2),
        "the band under the last one is the first one, and the cell keeps its number"
    );

    // **The keys `g` and `G` are the two ends of the band of the cursor.**
    app.handle_key(a_key(KeyCode::Char('g')));
    assert_eq!(app.list_state_cnt_list.selected(), Some(1), "the key g");
    app.handle_key(a_key(KeyCode::Char('G')));
    assert_eq!(app.list_state_cnt_list.selected(), Some(5), "the key G");

    // **A panel that drew the table of today keeps the keys of that table**
    // (the decision 5 of the maintainer): the key `l` of it plays the media of
    // the cursor, therefore it moves the cursor nowhere.
    app.the_bands_of_the_last_frame = TheBandsOfThePanel::default();
    app.list_state_cnt_list.select(Some(1));
    app.handle_key(a_key(KeyCode::Char('l')));
    assert_eq!(
        app.list_state_cnt_list.selected(),
        Some(1),
        "the key l of the table of today moves no cursor"
    );

    // **The key `j` of that table gives the line under**, over the two shelves
    // and every media of them.
    app.handle_key(a_key(KeyCode::Char('j')));
    assert_eq!(
        app.list_state_cnt_list.selected(),
        Some(2),
        "the key j of the table of today gives the line under"
    );

    // **A view that is not the Home view keeps every key that it had**: the
    // bands stand in the Home view alone.
    app.the_bands_of_the_last_frame = plan_the_bands(of_the_panel, 10, (10, 20).into(), &bands, 1);
    app.view_state = AppView::Library;
    app.list_state_cnt_list.select(Some(1));
    app.handle_key(a_key(KeyCode::Char('l')));
    assert_eq!(
        app.list_state_cnt_list.selected(),
        Some(1),
        "the keys of the bands reach the Home view alone"
    );

    // **The footer must not promise a key that the view does not hold**
    // (T-143). The footer of the bands says the four moves and it says `Enter`,
    // and the footer of the table of today keeps the key `l`.
    for is_podcast in [false, true] {
        let of_the_bands = toutui::ui::keys::the_footer_of_the_home_view(is_podcast, true);
        let of_the_table = toutui::ui::keys::the_footer_of_the_home_view(is_podcast, false);

        assert!(
            of_the_table.contains("j/k: move"),
            "the footer of the table of today keeps the moves that it had: {}",
            of_the_table
        );
        assert!(
            !of_the_table.contains("Enter: "),
            "the footer of the table of today promises no key Enter: {}",
            of_the_table
        );
        assert!(
            of_the_bands.contains("j/k: a shelf"),
            "the footer of the bands names the key of a shelf: {}",
            of_the_bands
        );
        assert!(
            of_the_bands.contains("h/l: a cover"),
            "the footer of the bands names the keys of a cover: {}",
            of_the_bands
        );
        assert!(
            of_the_bands.contains("Enter: "),
            "the footer of the bands names the key that plays: {}",
            of_the_bands
        );
        assert!(
            !of_the_bands.contains("l: play or open") && !of_the_bands.contains("l: the episodes"),
            "the footer of the bands promises no key l of a media: {}",
            of_the_bands
        );
    }

    assert!(
        toutui::ui::keys::the_footer_of_the_home_view(false, true).contains("Enter: play or open"),
        "the footer of a library of books says what the key Enter does"
    );
    assert!(
        toutui::ui::keys::the_footer_of_the_home_view(true, true).contains("Enter: the episodes"),
        "the footer of a library of podcasts says what the key Enter does"
    );
    assert!(
        toutui::ui::keys::the_footer_of_the_home_view(false, false).contains("l: play or open"),
        "the footer of the table of today keeps the key l"
    );
    assert!(
        toutui::ui::keys::the_footer_of_the_home_view(true, false).contains("l: the episodes"),
        "the footer of the table of today of a library of podcasts keeps the key l"
    );
}

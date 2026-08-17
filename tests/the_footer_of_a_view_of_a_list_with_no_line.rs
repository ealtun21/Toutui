//! The gate of T-360: the footer of the six views of a list that T-359 did not
//! reach names no key of a line.
//!
//! **A footer must not promise a key that the view does not hold** (T-143, and
//! T-79 for a key that does nothing). T-359 gave that rule to eleven views, and
//! it left the views whose count of the lines stands **after** the text of the
//! footer of their render function. The measurement of the real program v0.8.190
//! inside tmux, at 160 columns and 45 rows, of the sandbox
//! (`docs/TEST-SERVER.md`):
//!
//! * The bookmarks of "Large Book 0001", a book of the library `Large` that
//!   holds no bookmark, said
//!   `"Large Book 0001" has no bookmark. Press b while it plays.` over the
//!   footer
//!   `j/k: move  l: go to the place  X: remove the bookmark  h: back  ?: every key  Q: quit`.
//!   Three of the six parts named a key that does nothing.
//! * The lists that take a media, of the library `Large` which holds no
//!   collection and no playlist, said
//!   `This library holds no collection and no playlist. Press c or p to make one.`
//!   over the footer
//!   `j/k: move  l: put it here  c: a collection  p: a playlist  h: back  ?: every key  Q: quit`.
//! * The downloads of the server, of the library `Podcasts` whose queue was
//!   empty, said
//!   `The server downloads no episode. Press E on a podcast to get its new episodes.`
//!   over the footer
//!   `j/k: move  X: empty the queue of this podcast  h: back  ?: every key  Q: quit`.
//!
//! **The controls of the same run**: a `POST /api/me/item/:id/bookmark` gave
//! that book one bookmark, and the view of one line kept every key; and the
//! library `Books`, which holds one collection and one playlist, kept
//! `j/k: move  l: put it here` in the view of the lists that take a media.
//!
//! The corrected program of the same harness and of the same road gave
//! `h: back  ?: every key  Q: quit` to the bookmarks and to the downloads, and
//! `c: a collection  p: a playlist  h: back  ?: every key  Q: quit` to the lists
//! that take a media: **the keys `c` and `p` make a list, and they need no
//! line**, therefore they stay.
//!
//! **The statistics and the sessions take no rule of this item.** Their footers
//! name `j/k: move` too, and those two views hold no list at all: the keys move
//! the scroll of a text that the server gave. A text that fits the screen takes
//! no scroll, therefore a rule of that shape would change the footer of those
//! two views with the height of the terminal, and a footer that moves while the
//! user reads it is a fault of its own. See `docs/TAKEOVER-BACKLOG.md`, T-360.
//!
//! **The parts of this test stay in one function**: the test writes
//! `XDG_CONFIG_HOME` of the process, and a second function of this binary would
//! fight it for that box (T-144 and T-157).
//!
//! **This test needs no sandbox and no server.** `App::new` takes a port that
//! nothing listens on, therefore it gives the offline mode (T-25).

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::utils::collect_lists::{ListEntry, ListKind, ListView};
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;

/// Nothing listens on this port. See T-25.
const NO_SERVER: &str = "http://127.0.0.1:1";

fn a_user() -> User {
    User {
        server_address: NO_SERVER.to_string(),
        username: "toutuitest".to_string(),
        token: "not-a-real-token".to_string(),
        is_default_usr: true,
        name_selected_lib: "Large".to_string(),
        id_selected_lib: "a-library".to_string(),
        is_loop_break: "0".to_string(),
        has_played_before: "1".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    }
}

/// The rows of the screen, from the top to the bottom.
fn the_rows_of(terminal: &Terminal<TestBackend>) -> Vec<String> {
    let buffer = terminal.backend().buffer().clone();

    (0..buffer.area.height)
        .map(|row| {
            (0..buffer.area.width)
                .map(|column| buffer[(column, row)].symbol())
                .collect::<String>()
        })
        .collect()
}

/// The whole screen in one text. **A footer of many rows is one text** (T-302):
/// the wrap of a narrow terminal cuts the footer in two rows, therefore a test
/// of a part of it reads the rows together and not one row of them.
fn the_screen_of(terminal: &Terminal<TestBackend>) -> String {
    the_rows_of(terminal).join(" ")
}

#[tokio::test(flavor = "multi_thread")]
async fn the_footer_of_a_view_of_a_list_with_no_line() {
    // ## 1. The rule itself, with no screen at all
    use toutui::ui::keys;

    assert_eq!(
        keys::the_footer_of_a_view_with_no_line(keys::FOOTER_OF_THE_DOWNLOADS),
        "h: back  ?: every key  Q: quit",
        "the footer of the downloads with no line names no key of a line"
    );

    // **The keys `c` and `p` make a list of no line at all**, therefore the
    // footer of that view keeps them.
    assert_eq!(
        keys::the_footer_of_a_view_with_no_line(keys::FOOTER_OF_THE_LISTS_THAT_TAKE_A_MEDIA),
        "c: a collection  p: a playlist  h: back  ?: every key  Q: quit",
        "the footer of the lists that take a media keeps the keys that make a list"
    );

    // **The key `A` writes other words with no answer of the server at all.**
    assert_eq!(
        keys::the_footer_of_a_view_with_no_line(keys::FOOTER_OF_A_NEW_PODCAST),
        "A: other words  h: back  ?: every key  Q: quit",
        "the footer of a new podcast keeps the key that asks the server again"
    );

    assert_eq!(
        keys::the_footer_of_a_view_with_no_line(keys::FOOTER_OF_THE_DEVICES_OF_AN_EREADER),
        "h: back  ?: every key  Q: quit",
        "the footer of the devices of an e-reader with no device names no key of a line"
    );

    // **A view of one line keeps every part**, which is the control of the rule.
    assert_eq!(
        keys::the_footer_of_a_list(keys::FOOTER_OF_THE_DOWNLOADS, 1),
        keys::FOOTER_OF_THE_DOWNLOADS,
        "a view of one line keeps every part of its footer"
    );

    // ## 2. The views of the render
    //
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

    // The terminal of the measurement: 160 columns and 45 rows.
    let mut terminal = Terminal::new(TestBackend::new(160, 45)).expect("a terminal");

    // **The application of `App::new` holds no line of any of these views**: it
    // asked no server for a bookmark, for a device of an e-reader, for a queue
    // of the downloads, for an ebook, and for a podcast, therefore the state of
    // each of them is `Nothing`. The view of the lists that take a media holds
    // the lists of the library, and the offline mode gives it none.
    let the_views_with_no_line = [
        (AppView::Bookmarks, "the bookmarks"),
        (AppView::PutInAList, "the lists that take a media"),
        (AppView::Downloads, "the downloads of the server"),
        (AppView::SendToEreader, "the devices of an e-reader"),
        (AppView::Ebooks, "the ebooks of a media"),
        (AppView::NewPodcast, "a new podcast"),
    ];

    // The parts of a footer that need a line of the list under them. A view with
    // no line must name none of them.
    let the_keys_of_a_line = [
        "j/k: move",
        "l: go to the place",
        "X: remove the bookmark",
        "l: put it here",
        "X: empty the queue of this podcast",
        "l: send the book",
        "l: read this book",
        "l: add the podcast",
    ];

    for (view, name) in the_views_with_no_line {
        app.view_state = view;

        terminal
            .draw(|frame| frame.render_widget(&mut app, frame.area()))
            .expect("the view draws");

        let screen = the_screen_of(&terminal);

        for part in the_keys_of_a_line {
            assert!(
                !screen.contains(part),
                "{name} with no line must not name the key of the part \"{part}\""
            );
        }

        // **A screen that names no key looks like a program that stopped**
        // (T-52), therefore every one of these views keeps a key of its own.
        assert!(
            screen.contains("Q: quit"),
            "{name} with no line keeps the key that stops the program"
        );
    }

    // **The keys that need no line stay**: the view of the lists that take a
    // media holds no list, and the user makes the first one there.
    app.view_state = AppView::PutInAList;

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the lists that take a media draw");

    let screen = the_screen_of(&terminal);

    assert!(
        screen.contains("c: a collection") && screen.contains("p: a playlist"),
        "the lists that take a media with no list keep the keys that make a list"
    );

    // ## 3. The controls of the same run: a view of one line keeps every key
    //
    // **A correction that took the keys of a line away from every view would
    // fail this part.**
    app.view_state = AppView::PutInAList;
    app.lists = vec![ListView {
        id: "a-collection".to_string(),
        kind: ListKind::Collection,
        name: "A Test Collection".to_string(),
        description: String::new(),
        entries: vec![ListEntry {
            id: "a-media".to_string(),
            episode_id: None,
            title: "A Book Of A Test".to_string(),
            author: "A Writer".to_string(),
            duration: 600.0,
            description: String::new(),
        }],
    }];
    app.list_state_put_in_a_list.select(Some(0));

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the lists that take a media of one line draw");

    let screen = the_screen_of(&terminal);

    for part in ["j/k: move", "l: put it here"] {
        assert!(
            screen.contains(part),
            "the lists that take a media of one line keep the part \"{part}\" of its footer"
        );
    }

    // **The bookmarks of one bookmark keep every key too**, which says that the
    // rule reads the lines of each view and not the lines of one of them. The
    // state of that view is a box of its own, and not a field of the
    // application.
    toutui::logic::bookmarks::keep(toutui::logic::bookmarks::State::Ready(vec![
        toutui::api::me::bookmarks::Bookmark {
            library_item_id: "a-media".to_string(),
            time: 42.0,
            title: "The bookmark of the measurement".to_string(),
        },
    ]));

    app.view_state = AppView::Bookmarks;
    app.list_state_bookmarks.select(Some(0));

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the bookmarks of one line draw");

    let screen = the_screen_of(&terminal);

    for part in ["j/k: move", "l: go to the place", "X: remove the bookmark"] {
        assert!(
            screen.contains(part),
            "the bookmarks of one line keep the part \"{part}\" of its footer"
        );
    }
}

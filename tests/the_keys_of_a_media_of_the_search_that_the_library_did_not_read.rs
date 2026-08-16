//! The keys of a media of the view of the search work for a media that the lists
//! of the library do not hold. See T-218.
//!
//! **The view of the search holds a media that the program did not read** (T-113
//! and T-117): the program reads one page of 500 items (T-70), and the server
//! searches the whole library. `selected_download` then asked `ids_library` for
//! the place of that media, and it gave nothing at all for a media of a page that
//! the program did not read.
//!
//! The measurement of the real program of 2026-08-14, of the library `Large` of
//! 2056 items and of the line "Large Book 1200" of the view of the search:
//!
//! ```text
//! ➤ Large Book 1200
//! ```
//!
//! The key `D` wrote no word and no line of the log, the key `X` wrote no word and
//! no line of the log, the key `n` said "This line holds no media.", and the key
//! `m` said "This line holds no book and no episode.". The key `l` of that same
//! line played the book, therefore the line held a media and the four keys said
//! the opposite (T-79 and T-91). The same keys of "Large Book 2000", of the first
//! page of the library, downloaded the book and removed it.
//!
//! This test makes an `App` of a library of three books, and it gives the box of
//! the answer of the search a book that those lists do not hold. The four keys of
//! that line read `selected_download` and `selected_media`, therefore the two
//! functions must give the media of the line.
//!
//! The test needs no server: the address is a port that nothing listens on,
//! therefore `App::new` gives the offline mode (T-25). The box of the answer is a
//! slot of the whole process, therefore **this test stands alone in its binary**
//! and it holds every measurement in one function. See the trap 8 of the harness.

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;
use toutui::logic::download::DownloadTarget;
use toutui::logic::playback::PlaybackTarget;
use toutui::logic::search::from_the_server::{keep, Answer};
use toutui::logic::search::Found;

/// Nothing listens on this port.
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

/// Draws the view of the search. The lists of the view come of the answer at that
/// moment, and the keys of the user read them after the frame.
fn one_frame(app: &mut App) {
    let backend = TestBackend::new(160, 45);
    let mut terminal = Terminal::new(backend).expect("a terminal");

    terminal
        .draw(|frame| frame.render_widget(&mut *app, frame.area()))
        .expect("the view of the search must draw");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_keys_of_the_line_hold_the_media_of_the_answer() {
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

    // The program holds the first page of the library: three books of 2056.
    app.titles_library = vec![
        "Large Book 2056".to_string(),
        "Large Book 2055".to_string(),
        "Large Book 2054".to_string(),
    ];
    app.ids_library = vec![
        "item-2056".to_string(),
        "item-2055".to_string(),
        "item-2054".to_string(),
    ];
    app.auth_names_library = vec!["N/A".to_string(); 3];
    app.published_year_library = vec!["N/A".to_string(); 3];
    app.desc_library = vec!["No description available".to_string(); 3];
    app.duration_library = vec![60.0; 3];
    app.library_total = 2056;
    app.library_rows =
        toutui::logic::library_view::group_library(&app.ids_library, &app.series, false);

    app.view_state = AppView::SearchBook;
    app.search_query = "Large Book 1200".to_string();
    app.list_state_search_results.select(Some(0));

    // **The measurement.** The server found one book, and that book stands on a
    // page that the program did not read.
    keep(Answer {
        words: "Large Book 1200".to_string(),
        media: vec![Found {
            id: "item-1200".to_string(),
            title: "Large Book 1200".to_string(),
            author: "A Test Author".to_string(),
            author_of_a_podcast: "N/A".to_string(),
            year: "2026".to_string(),
            description: "A book of the page 2.".to_string(),
            duration: 1234.0,
            place: None,
        }],
        names: Vec::new(),
    });

    // The lists of the view come of the answer at the moment of the render. The
    // keys of the user read them after that frame.
    one_frame(&mut app);

    assert_eq!(
        app.ids_search_book,
        vec!["item-1200".to_string()],
        "the line of the view holds the identity of the book of the answer"
    );

    // **The keys `D` and `X`.** They said nothing at all: no word for the user,
    // and no line of the log.
    let (target, title, author) = app
        .selected_download()
        .expect("the keys D and X hold the media of the line of the search");

    assert_eq!(
        target,
        DownloadTarget::Book {
            item_id: "item-1200".to_string()
        }
    );
    assert_eq!(title, "Large Book 1200", "the title comes of the answer");
    assert_eq!(author, "A Test Author", "the author comes of the answer");

    // **The keys `n` and `m`.** They said "This line holds no media." and "This
    // line holds no book and no episode." for a line that holds a book.
    let entry = app
        .selected_media()
        .expect("the keys n and m hold the media of the line of the search");

    assert_eq!(
        entry.target,
        PlaybackTarget::Book {
            item_id: "item-1200".to_string(),
            whole_book_duration: Some(1234.0),
        }
    );
    assert_eq!(entry.title, "Large Book 1200");
    assert_eq!(entry.author, "A Test Author");
    assert_eq!(
        entry.duration,
        Some(1234.0),
        "the length of the queue comes of the answer of the search"
    );

    // **A media of the first page keeps its keys.** The lists of the library hold
    // that book, and the road of the correction must not need them.
    keep(Answer {
        words: "Large Book 2055".to_string(),
        media: vec![Found {
            id: "item-2055".to_string(),
            title: "Large Book 2055".to_string(),
            author: "N/A".to_string(),
            author_of_a_podcast: "N/A".to_string(),
            year: "N/A".to_string(),
            description: "No description available".to_string(),
            duration: 60.0,
            place: Some(1),
        }],
        names: Vec::new(),
    });

    app.search_query = "Large Book 2055".to_string();
    one_frame(&mut app);

    let (target, title, _author) = app
        .selected_download()
        .expect("a media of the first page of the library keeps its keys");

    assert_eq!(
        target,
        DownloadTarget::Book {
            item_id: "item-2055".to_string()
        }
    );
    assert_eq!(title, "Large Book 2055");

    // **A library of podcasts holds the episodes of a media, and not the media**
    // (T-113): the user opens the podcast with the key `l`, and the keys of a
    // download stand in the view of the episodes.
    app.is_podcast = true;

    assert_eq!(
        app.selected_download(),
        None,
        "a line of a library of podcasts holds no media of a download"
    );
}

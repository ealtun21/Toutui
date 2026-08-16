//! The view of the search shows the media that the server found, and the program
//! does not need to hold that media. See T-113.
//!
//! **The view held the lists of the library, and it read them with the place of
//! the media in those lists.** The program reads one page of 500 items (T-70),
//! therefore a media of a page that it did not read gave no line at all: the
//! sweep of a library of 2056 items of 2026-08-12 asked the server for
//! "Large Book 0100", the server answered with that book, and the screen said
//! "The server found nothing for "Large Book 0100"".
//!
//! This test makes an `App` of a library of three books, and it gives the box of
//! the answer a book that the library lists do not hold. The screen must then
//! hold the title, the author, and the year of that book.
//!
//! The test needs no server: the address is a port that nothing listens on,
//! therefore `App::new` gives the offline mode (T-25). The box of the answer is a
//! slot of the whole process, therefore **this test stands alone in its binary**
//! and it holds every measurement in one function. See the trap 8 of the
//! harness.

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;
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
        name_selected_lib: "Books".to_string(),
        id_selected_lib: "a-library".to_string(),
        is_loop_break: "0".to_string(),
        has_played_before: "1".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    }
}

/// Draws the view of the search, and it gives the text of the screen.
fn the_screen(app: &mut App) -> String {
    let backend = TestBackend::new(160, 45);
    let mut terminal = Terminal::new(backend).expect("a terminal");

    terminal
        .draw(|frame| frame.render_widget(&mut *app, frame.area()))
        .expect("the view of the search must draw");

    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn the_view_holds_a_media_that_the_program_did_not_read() {
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
    app.auth_names_library_pod = vec!["N/A".to_string(); 3];
    app.published_year_library = vec!["N/A".to_string(); 3];
    app.desc_library = vec!["No description available".to_string(); 3];
    app.duration_library = vec![60.0; 3];
    app.library_total = 2056;
    app.library_rows =
        toutui::logic::library_view::group_library(&app.ids_library, &app.series, false);

    app.view_state = AppView::SearchBook;
    app.search_query = "Large Book 0100".to_string();
    app.list_state_search_results.select(Some(0));

    // **The measurement.** The server found one book, and that book stands on a
    // page that the program did not read.
    keep(Answer {
        words: "Large Book 0100".to_string(),
        media: vec![Found {
            id: "item-100".to_string(),
            title: "Large Book 0100".to_string(),
            author: "A Test Author".to_string(),
            author_of_a_podcast: "N/A".to_string(),
            year: "2026".to_string(),
            description: "A book of the page 4.".to_string(),
            duration: 60.0,
            place: None,
        }],
        names: Vec::new(),
    });

    let screen = the_screen(&mut app);

    assert!(
        screen.contains("Large Book 0100"),
        "the view must hold the book of the server: {}",
        &screen[..screen.len().min(400)]
    );
    assert!(
        !screen.contains("found nothing"),
        "the server found one book: {}",
        &screen[..screen.len().min(400)]
    );

    // The title of the view counts the lines of the answer, and it says that the
    // answer comes from the server.
    assert!(
        screen.contains("Search result [1 item]"),
        "{}",
        &screen[..400]
    );

    // The values of the line come from the answer, and not from the lists of the
    // library.
    assert!(screen.contains("A Test Author"), "the author of the answer");
    assert!(screen.contains("A book of the page 4."), "its description");

    // **The key `l` needs the identity.** A line that the user cannot play is no
    // line at all.
    assert_eq!(app.ids_search_book, vec!["item-100".to_string()]);
    assert_eq!(
        app.auth_names_search_book,
        vec!["A Test Author".to_string()]
    );
    assert_eq!(app.published_year_library_search_book, vec!["2026"]);

    // **The reader of a book needs the title of the line.** The view of the
    // search held no list of the titles, therefore the reader of a PDF said the
    // identity of the item: a measurement of 2026-08-12 read
    // "27c55369-b048-4d68-9e70-17653b4d618f — page 1 of 150". See T-117 and
    // T-54.
    assert_eq!(app.titles_search_book, vec!["Large Book 0100".to_string()]);
    assert_eq!(
        app.selected_item_title(),
        Some("Large Book 0100".to_string()),
        "the reader takes the title of the line of the search"
    );

    // **A library of podcasts reads the episodes of a media with the place of
    // that media in the lists of the library**, therefore a podcast that the
    // program did not read gives no line yet. T-113 left the title of that
    // condition at "The server found nothing", and the sweep of a library of
    // 520 podcasts of 2026-08-12 met it: the server found the podcast, and the
    // screen said the opposite. The title says what the program does now, and
    // the program reads the pages of the library. See T-125 and T-91.
    app.is_podcast = true;

    let of_a_podcast = the_screen(&mut app);

    assert!(
        !of_a_podcast.contains("found nothing"),
        "the server found that podcast: {}",
        &of_a_podcast[..of_a_podcast.len().min(400)]
    );
    assert!(
        of_a_podcast.contains("1 podcast"),
        "the title says how many podcasts come: {}",
        &of_a_podcast[..of_a_podcast.len().min(400)]
    );
    assert!(
        of_a_podcast.contains("reads the pages"),
        "the title says what the program does now: {}",
        &of_a_podcast[..of_a_podcast.len().min(400)]
    );

    // The book of the library stays: the program holds its place, therefore
    // every list of the episodes agrees with the line.
    app.is_podcast = false;
    app.search_query = "Large Book 2055".to_string();
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
            place: None,
        }],
        names: Vec::new(),
    });

    let of_the_library = the_screen(&mut app);

    assert!(of_the_library.contains("Large Book 2055"));
    assert_eq!(app.ids_search_book, vec!["item-2055".to_string()]);
}

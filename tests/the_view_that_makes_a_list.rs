//! The view that puts a media in a list, and that makes a list. See T-88.
//!
//! **A library holds no collection and no playlist until a user makes the first
//! one.** The program refused to open this view in that condition, and it said
//! "The web page of the server makes one" in a message of one row. The user had
//! no way to make a list from the program.
//!
//! The view opens now, and the keys `c` and `p` make a list. **An empty box
//! says nothing**, therefore the title of the view says the condition and it
//! names the two keys.
//!
//! This test sets `XDG_CONFIG_HOME`, therefore it stands alone in its binary.
//! It needs no server: the address is a port that nothing listens on, and
//! `App::new` gives the offline mode. See T-25.

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::api::utils::collect_lists::{ListKind, ListView};
use toutui::app::{App, AppView};
use toutui::db::database_struct::User;

/// Nothing listens on this port, therefore every request fails at once.
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

/// Gives the whole text of one frame of the view.
fn the_screen(app: &mut App) -> String {
    let backend = TestBackend::new(160, 40);
    let mut terminal = Terminal::new(backend).expect("a terminal");

    terminal
        .draw(|frame| frame.render_widget(&mut *app, frame.area()))
        .expect("the view must draw");

    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect()
}

/// The parts of this test stand in one function: it writes `XDG_CONFIG_HOME`,
/// and that value belongs to the whole process. See the trap 8 of the harness.
#[tokio::test(flavor = "multi_thread")]
async fn the_view_says_the_two_keys_that_make_a_list() {
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

    app.view_state = AppView::PutInAList;
    app.the_media_of_the_list = Some((
        "an-item".to_string(),
        None,
        "Alice in Wonderland".to_string(),
    ));

    // 1. The library holds no list. The title says the condition and the two
    //    keys: an empty box says nothing.
    //
    //    **The application of this test is in the offline mode**, because the
    //    address of the server answers nothing. A list stands on the server,
    //    therefore the view says a different sentence in that mode: the test
    //    measures the two conditions, one after the other. See T-91.
    app.lists = Vec::new();
    app.is_offline = true;

    let text = the_screen(&mut app);

    assert!(
        text.contains("The server does not answer. A collection and a playlist stand on the "),
        "the view must not say that the library holds no list when no request \
         gave an answer"
    );
    assert!(
        !text.contains("Press c or p to make one"),
        "a server that does not answer takes no new list"
    );

    app.is_offline = false;

    let text = the_screen(&mut app);

    assert!(
        text.contains(
            "This library holds no collection and no playlist. Press c or p to make one."
        ),
        "the view of a library with no list must say the two keys"
    );

    // 2. The footer names the two keys in every condition of this view.
    assert!(
        text.contains("c: a collection  p: a playlist"),
        "the footer must name the keys that make a list"
    );

    // 3. The library holds a list. The title names the media, and the footer
    //    keeps the two keys.
    app.lists = vec![ListView {
        id: "a-list".to_string(),
        kind: ListKind::Playlist,
        name: "A Test Playlist".to_string(),
        description: String::new(),
        entries: Vec::new(),
    }];

    let text = the_screen(&mut app);

    assert!(
        text.contains("Put \"Alice in Wonderland\" in a list [1 item]"),
        "the title must name the media and the number of the lists"
    );
    assert!(
        text.contains("c: a collection  p: a playlist"),
        "the footer must name the keys that make a list"
    );
    assert!(
        !text.contains("Press c or p to make one"),
        "a library that holds a list must not say that it holds none"
    );
}

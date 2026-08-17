//! The parts of one row of the header do not write over each other. See T-340.
//!
//! **The header holds three parts on one row, and each of them is a paragraph
//! of its own over the whole area** (T-115): the account at the left, the name
//! of the library in the middle, and the name of the program at the right of
//! the first row, and the address of the server at the left, the words of the
//! sequence and of the filter in the middle, and the notice of the key `R` at
//! the right of the second row. **T-115 gave the header a short form and it
//! measured nothing**, and **T-329 corrected the middle of the second row
//! alone** — its own words name this fault "the fault of T-115 one row below".
//!
//! The measurement of the real program v0.8.171 inside tmux, of the Home view
//! of the library `Podcasts` of the sandbox at 40 columns:
//!
//! ```text
//! 👋 toutuitestPodcasts (podcas🦜 v0.8.171
//! ```
//!
//! The same program of the offline mode, at the same width:
//!
//! ```text
//! 📴 toutuiteste: the media on 🦜 v0.8.171
//! 🔗 localhost:133 R: try the server again
//! ```
//!
//! **The second row says an address that the user does not have**: the notice
//! of the key `R` cut `🔗 localhost:13399 does not answer` at the port, and the
//! header then named the port 133.
//!
//! The corrected program v0.8.172 of the same harness, at the same width:
//!
//! ```text
//! 👋 toutuitest  📖 Podcasts (podcast)
//! ```
//!
//! ```text
//!    📴 Offline: the media on the disk
//! 🔗 localhost:13399 does not answer
//! ```

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::sync::Arc;
use toutui::api::client::endpoint::{Endpoint, EndpointPool};
use toutui::api::client::ApiClient;
use toutui::app::App;
use toutui::db::database_struct::User;
use toutui::logic::message::the_columns_of;

/// Nothing listens on this port, therefore every request fails at once and
/// `App::new` gives the offline mode. See T-25.
const NO_SERVER: &str = "http://127.0.0.1:1";

/// The narrowest terminal that this fork measures. See T-301.
const NARROW: u16 = 40;

fn a_user() -> User {
    User {
        server_address: NO_SERVER.to_string(),
        username: "toutuitest".to_string(),
        token: "not-a-real-token".to_string(),
        is_default_usr: true,
        name_selected_lib: "Podcasts".to_string(),
        id_selected_lib: "a-library".to_string(),
        is_loop_break: "0".to_string(),
        has_played_before: "1".to_string(),
        speed_rate: 1.0,
        is_show_key_bindings: "1".to_string(),
    }
}

/// The columns of a row of the screen, from its first character to its last.
fn the_columns_of_the_row(row: &str) -> usize {
    the_columns_of(row.trim_end())
}

/// One row of the screen, as the user reads it.
///
/// **A mark of two columns holds two cells of the buffer**: ratatui writes `👋`
/// in the first of them and it leaves the second one, therefore a row that
/// joins every cell holds one character more than the screen for each mark. The
/// function reads the columns of each symbol and it steps over the cells that
/// the symbol before it took.
fn the_row_of(buffer: &ratatui::buffer::Buffer, row: u16) -> String {
    let mut text = String::new();
    let mut column = 0;

    while column < buffer.area.width {
        let symbol = buffer[(column, row)].symbol().to_string();
        let of_the_symbol = u16::try_from(the_columns_of(&symbol)).unwrap_or(1).max(1);

        text.push_str(&symbol);
        column = column.saturating_add(of_the_symbol);
    }

    text
}

/// No part of a row of the header writes over the part beside it, and a part
/// that has no room does not stand at all. See T-340.
///
/// **The parts of this test stay in one function**: the test writes
/// `XDG_CONFIG_HOME` of the process, and a second function of this binary would
/// fight it for that box.
#[tokio::test(flavor = "multi_thread")]
async fn no_part_of_the_header_writes_over_its_neighbour() {
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

    // **The name of a library is a text of the server** (T-314), and the
    // library of the measurement of this round says 21 columns. The account
    // says 13 of them, and the two of them and the gap of two columns then hold
    // 36 of the 40 columns of the screen: the name of the program has no room
    // at all.
    let the_library = toutui::ui::keys::the_name_of_the_library("Podcasts", "podcast");
    app.lib_name_type = the_library.clone();
    app.server_address_pretty = "localhost:13399".to_string();

    let backend = TestBackend::new(NARROW, 45);
    let mut terminal = Terminal::new(backend).expect("a terminal");

    terminal
        .draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the view draws");

    let buffer = terminal.backend().buffer().clone();
    let the_first_row = the_row_of(&buffer, 0);
    let the_second_row = the_row_of(&buffer, 1);

    // **The name of the library stands whole.** The program before this
    // correction drew the account over the mark `📖` of it and the name of the
    // program over the end of `(podcast)`, and the user read
    // `👋 toutuitestPodcasts (podcas`.
    assert!(
        the_first_row.contains(&the_library),
        "the first row of the header lost a part of the name of the library:\n{}\n{}",
        the_first_row,
        the_second_row
    );

    // **The address of the server stands whole.** A row that cuts it says an
    // address that the user does not have: the measurement of the offline mode
    // at this width read `🔗 localhost:133`.
    let the_address = the_second_row.trim_end().to_string();
    assert!(
        the_address.contains("13399"),
        "the second row of the header cut the address of the server:\n{}\n{}",
        the_first_row,
        the_second_row
    );

    // **A part of the account stands whole too, or it does not stand at all.**
    // The account of this measurement holds 13 columns, and the row therefore
    // holds it and the name of the library together.
    let at = the_first_row
        .find(&the_library)
        .expect("the first row holds the name of the library");

    // **`String::find` gives the index of a byte and not the column of the
    // screen** (the trap 245): the marks `👋` and `📖` take four bytes and two
    // columns each.
    let of_the_library = the_columns_of(&the_first_row[..at]);
    let of_the_account = the_columns_of(the_first_row[..at].trim_end());

    assert!(
        of_the_account > 0,
        "the first row of the header holds no account at all:\n{}",
        the_first_row
    );

    // The gap of two columns of T-329 stands between the two of them.
    assert!(
        of_the_library >= of_the_account + 2,
        "the account of {} columns and the name of the library at the column {} \
         hold a gap of {} columns:\n{}",
        of_the_account,
        of_the_library,
        of_the_library - of_the_account,
        the_first_row
    );

    // **No row of the header is wider than the screen**: a part that the row
    // cannot hold does not stand at all.
    assert!(the_columns_of_the_row(&the_first_row) <= usize::from(NARROW));
    assert!(the_columns_of_the_row(&the_second_row) <= usize::from(NARROW));

    // The control of the same run: a screen of 160 columns holds the three
    // parts of the first row together, and the name of the program is the part
    // that the narrow screen took away.
    let of_the_program = toutui::ui::keys::the_name_of_the_program("0.0.0", 160);
    assert!(
        !the_first_row.contains(&of_the_program),
        "the row of 40 columns holds the name of the program:\n{}",
        the_first_row
    );

    let wide = TestBackend::new(160, 45);
    let mut wide = Terminal::new(wide).expect("a terminal");
    wide.draw(|frame| frame.render_widget(&mut app, frame.area()))
        .expect("the view draws");

    let of_the_wide = wide.backend().buffer().clone();
    let the_row = the_row_of(&of_the_wide, 0);

    assert!(
        the_row.contains(&the_library),
        "the screen of 160 columns lost the name of the library:\n{}",
        the_row
    );
    assert!(
        the_row.contains("toutuitest") && the_row.contains("🦜"),
        "the screen of 160 columns lost a part of the header:\n{}",
        the_row
    );
}

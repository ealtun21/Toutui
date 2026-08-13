//! The key `X` keeps the media that a program of this account plays from the
//! disk. See T-156.
//!
//! **The measurement of 2026-08-14, with the sandbox away and two sessions of
//! tmux of one `XDG_CONFIG_HOME`.** The window A played "A Book Of Many Hours"
//! of 115200330 bytes in the offline mode; the window B pressed `X` on the line
//! of that media at the minute 34 of the playback, and the program said
//! `Removed the local copy of "A Book Of Many Hours".` The directory of the
//! download and its row went away while the user listened to that book:
//!
//! - the playback of A went on, because the engine holds the file open;
//! - the key `l` of A on the same media then said `The server does not answer,
//!   and the disk has no copy of this media.`, and the log said
//!   `the disk has no copy of 6ba57b9a…`;
//! - no key of the program gives that book back while the server is away.
//!
//! **An offline playback opens no session on the server** (T-152), therefore
//! `listening_session` holds no row of it and no second program of the account
//! can see that work. The loop of that playback keeps the place of the user in
//! `pending_progress` at each second since T-152, and that moment is the
//! heartbeat: it is the rule of T-140, of T-148, and of T-153.
//!
//! This test needs no server. It writes `XDG_CONFIG_HOME`, therefore it stands
//! alone in its binary (the trap 8 of the harness, and T-144).

use toutui::db::crud::PendingProgress;
use toutui::db::crud::{
    a_program_keeps_the_place_of_this_media, insert_pending_progress, THE_LIMIT_OF_THE_HEARTBEAT,
};
use toutui::logic::download::{
    text_of_the_media_that_plays_from_the_disk, the_work_of_the_key_that_removes,
    TheWorkOfTheKeyThatRemoves,
};

const USER: &str = "toutuitest";
const SERVER: &str = "http://127.0.0.1:1";
const THE_BOOK: &str = "6ba57b9a-acb5-44f9-b2b6-39ad9107b420";

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_millis() as i64)
        .unwrap_or(0)
}

fn a_place(id_item: &str, updated_at: i64) -> PendingProgress {
    PendingProgress {
        id_item: id_item.to_string(),
        id_pod: String::new(),
        current_time: 2041.0,
        duration: 28800.0,
        is_finished: false,
        updated_at,
    }
}

#[test]
fn the_key_x_keeps_the_book_that_a_program_of_this_account_plays() {
    // No line of this test may touch the files of the user.
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::env::set_var("XDG_DATA_HOME", dir.path());
    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();

    let conn = toutui::db::migrate::open_conn().unwrap();
    toutui::db::migrate::run_migrations(&conn).unwrap();
    drop(conn);

    // No playback of this account wrote a place of this media. The key takes
    // the disk, and that is the rule of T-150.
    assert!(!a_program_keeps_the_place_of_this_media(USER, THE_BOOK, ""));

    // The loop of the offline playback of the window A writes the place at each
    // second (T-152).
    insert_pending_progress(USER, SERVER, &a_place(THE_BOOK, now_ms())).unwrap();

    assert!(
        a_program_keeps_the_place_of_this_media(USER, THE_BOOK, ""),
        "a place of this second belongs to a playback that runs"
    );

    // **A mark of a playback is not for ever** (T-153): a place that stood still
    // for the limit of the heartbeat belongs to a playback that ended, and the
    // key of the user then takes the disk.
    let old = now_ms() - (THE_LIMIT_OF_THE_HEARTBEAT as i64 + 5) * 1000;
    insert_pending_progress(USER, SERVER, &a_place(THE_BOOK, old)).unwrap();

    assert!(
        !a_program_keeps_the_place_of_this_media(USER, THE_BOOK, ""),
        "a place of a playback that ended must not keep the copy for ever"
    );

    // The place of one media says nothing of a different media.
    insert_pending_progress(USER, SERVER, &a_place(THE_BOOK, now_ms())).unwrap();
    assert!(!a_program_keeps_the_place_of_this_media(
        USER,
        "a-different-media",
        ""
    ));

    // The rule of the key, and the sentence that it says.
    assert_eq!(
        the_work_of_the_key_that_removes(
            false,
            false,
            a_program_keeps_the_place_of_this_media(USER, THE_BOOK, "")
        ),
        TheWorkOfTheKeyThatRemoves::AProgramPlaysItFromTheDisk
    );

    let text = text_of_the_media_that_plays_from_the_disk("A Book Of Many Hours");
    assert!(text.contains("A Book Of Many Hours"), "{}", text);

    // **No unit test reaches a key handler of `src/app.rs`**, therefore this
    // part reads the source, as the tests of T-131, T-143, T-149, T-150, T-151,
    // and T-155 do: the key `X` must ask for the place of the media, and it must
    // say the sentence of a media that plays.
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/app.rs"))
        .expect("the test must read src/app.rs");

    let of_the_key = source
        .split_once("let work = crate::logic::download::the_work_of_the_key_that_removes(")
        .expect("the key X must ask for the work of the removal")
        .1;

    let of_the_rule = of_the_key.split_once(");").expect("the call must end").0;

    assert!(
        of_the_rule.contains("a_program_keeps_the_place_of_this_media("),
        "the key X must ask if a program of this account plays this media from the disk"
    );

    assert!(
        of_the_key.contains("text_of_the_media_that_plays_from_the_disk("),
        "the key X must say why it removed nothing (T-79)"
    );
}

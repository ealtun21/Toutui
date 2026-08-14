//! A read of the row of the account that failed is not a setting of the user.
//! See T-209.
//!
//! **The parts of this test stay in one function.** The test writes
//! `XDG_CONFIG_HOME`, and that variable belongs to the process: two test
//! functions of one binary fight for it. See T-144 and T-157.
//!
//! # The condition
//!
//! T-199 and T-200 gave the fault of the disk to the writes of the module of the
//! database, and T-202 to the reads whose default is a **fact** of the user. The
//! three reads of the row of `users` of an account hold the other half of that
//! sweep: their default is a **setting** of the user, and the program then does
//! the work that the user did not ask for.
//!
//! `get_speed_rate` gave the text `Error: unable open database` for a fault of
//! the disk, and its three callers wrote `.parse::<f32>().unwrap_or(1.0)`;
//! `get_library_sort` gave three empty texts; `get_is_show_key_bindings` gave a
//! text that is not `1`.
//!
//! # The measurements of 2026-08-14
//!
//! The real program of the sandbox, with a column of another name (the road of
//! T-203, and the trap 172 of T-207):
//!
//! ```bash
//! sqlite3 "$DB" "ALTER TABLE users RENAME COLUMN speed_rate TO speed_rate_of_an_old_version;"
//! ```
//!
//! 1. **The speed of the user.** The disk of the account held **1.5**, the key
//!    `l` of the Home view played a book of eight hours with the null device of
//!    ALSA, and the row of the player said `Speed: 1.00x`. **The log held no line
//!    at all**, and no word of the screen named the speed.
//! 2. **The sequence and the filter of the library.** The header of the Library
//!    view said `Library [17 items] — The title, the largest first`, and with the
//!    column of another name it said `Library [17 items]`. The log held no line.
//!
//! With the correction, the same two conditions:
//!
//! ```text
//! The program did not read the speed of this account: the database did not
//! answer. This media plays at 1.00x. Press O or I to set the speed again.
//!
//! Toutui stops: it cannot read the accounts of its database.
//! The program did not read the accounts of its database: no such column:
//! library_sort in SELECT library_sort, library_desc, library_filter FROM users
//! WHERE username = ?1 at offset 7
//! ```
//!
//! The key `R` of the same condition kept the application of the user (T-205),
//! and the log held the fault.
//!
//! # The rule
//!
//! **The three reads stand on the row of `users` of this account, which is the
//! row of the accounts of T-199**, therefore a fault of one of them is the fault
//! of the accounts: the start stops with words that name the database, and a
//! refresh keeps the application. The start of a playback is a key of the user
//! that waits for a media and not for a speed, therefore that playback goes on
//! and the program says which speed it plays.
//!
//! The condition of this test is a file that holds no database: it gives the same
//! fault of `open_conn` with no wait at all (T-199 and T-200).

use toutui::db::crud;

#[test]
fn a_read_of_the_row_of_the_account_is_no_setting_of_the_user() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", dir.path());

    std::fs::create_dir_all(dir.path().join("toutui")).unwrap();
    std::fs::write(
        dir.path().join("toutui").join("db.sqlite3"),
        b"this file holds no database at all",
    )
    .unwrap();

    // **A read that failed is not the speed 1.00x.** The old shape gave a text
    // of a fault, and every caller parsed it to 1.0.
    assert!(
        crud::get_speed_rate("toutuitest").is_err(),
        "a read of the speed that failed must not give a speed of this account"
    );

    // **A read that failed is not an account that chose no sequence.**
    assert!(
        crud::get_library_sort("toutuitest").is_err(),
        "a read of the sequence that failed must not give an account of no sequence"
    );

    // **A read that failed is not an account that hid the row of the keys.**
    assert!(
        crud::get_is_show_key_bindings("toutuitest").is_err(),
        "a read of the keys that failed must not give an account that hid them"
    );

    // The words of the playback name the media of now, the speed that it plays,
    // and the two keys of that work (T-79 and T-170). They say no reason that
    // the program does not have (T-91): the disk did not answer.
    let of_the_playback = toutui::logic::playback::THE_SPEED_OF_THE_DISK_DID_NOT_COME;

    assert!(
        of_the_playback.contains("1.00x"),
        "the words must name the speed that the media plays: {}",
        of_the_playback
    );
    assert!(
        of_the_playback.contains("Press O or I"),
        "the words must name the keys of that work: {}",
        of_the_playback
    );

    // **A write that the disk took and a read that failed are two conditions**
    // (T-91). The key `O` of the player writes the row and it reads it again:
    // the disk then holds the new speed and the engine holds the speed of
    // before, therefore the sentence names the media and not the row.
    let of_the_key =
        toutui::player::integrated::handle_key_player::the_words_of_a_speed_that_the_disk_did_not_give(
            "O",
        );

    assert!(
        of_the_key.contains("This media keeps its speed"),
        "the words must say what the media does now: {}",
        of_the_key
    );
    assert!(
        of_the_key.contains("Press O again"),
        "the words must name the key of the view that the user sees: {}",
        of_the_key
    );

    // The sentence of a write that failed (T-206) says the other thing, because
    // the disk holds the speed of before in that condition.
    let of_the_write =
        toutui::player::integrated::handle_key_player::the_words_of_a_speed_that_the_disk_did_not_hold(
            "O",
        );

    assert!(
        of_the_write.contains("The speed does not change"),
        "the words of a write that failed name the row of the disk: {}",
        of_the_write
    );
    assert_ne!(
        of_the_write, of_the_key,
        "a read that failed and a write that failed are two conditions"
    );
}

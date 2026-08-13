//! The key `D` takes the address of the pool, and its requests hold a limit of
//! time. See T-149.
//!
//! **A measurement of 2026-08-13 read the two requests of a download in the log
//! of a proxy that held the address of the login**, while the header of the
//! program said a different address: `pool` decides the address of every other
//! request of the program (T-105, T-107, and T-128), and the key `D` held the
//! address of the row of the database. A user away from home therefore sent the
//! download to the address of their house.
//!
//! **The same measurement with an address that no machine answers gave the user
//! no message, no line of the log, and no bar of the progress**, for ever:
//! `reqwest::Client::new()` holds no limit of time at all.
//!
//! No unit test reaches a key handler of `src/app.rs`, and no test can read the
//! limits of a client of `reqwest`. This test reads the source, as the test of
//! T-131 and the test of T-143 do.

/// The handler of the key `D` must give the download the address of the pool.
#[test]
fn the_key_of_the_download_asks_the_pool_for_the_address() {
    let source = include_str!("../src/app.rs");

    let start = source
        .find("KeyCode::Char('D') => {")
        .expect("the handler of the key D");
    let block = &source[start..start + 2000];

    assert!(
        block.contains("the_address_of_the_download(") && block.contains("pool().an_address()"),
        "the key D must take the address of the pool, and not the address of \
         the login: a user away from home holds the address of their house in \
         the row of the database. See T-149.\n{}",
        &block[..600]
    );
}

/// The client of a download must hold a limit of the connection and a limit of
/// a wait with no byte, and **no limit of the whole download**.
#[test]
fn the_client_of_a_download_holds_the_two_limits_of_time() {
    let source = include_str!("../src/logic/download/mod.rs");

    let start = source
        .find("fn the_client_of_a_download()")
        .expect("the function that makes the client of a download");
    let block = &source[start..start + 500];

    assert!(
        block.contains("connect_timeout"),
        "a download of an address that no machine answers must not wait for \
         ever. See T-149."
    );
    assert!(
        block.contains("read_timeout"),
        "a transfer that stops in the middle must give the user a sentence. \
         See T-149."
    );
    assert!(
        !block.contains(".timeout("),
        "a download must hold no limit of its whole time: a book of 479 \
         megabytes took 36 seconds in the measurement of T-119, and a book of \
         some gigabytes takes much more."
    );

    // The client of no limit is the fault of T-149 itself.
    assert!(
        !source.contains("reqwest::Client::new()"),
        "every request of a download must take the client of \
         `the_client_of_a_download`. See T-149."
    );
}

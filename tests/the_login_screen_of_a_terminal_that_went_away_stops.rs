//! The login screen of a program whose terminal went away stops. See T-272.
//!
//! **T-271 gave the program a watch of its terminal, and that watch starts
//! after the login.** It needs the client of the server, the name of an
//! account, and the name of a server, and the login screen stands before every
//! one of them. Therefore a program that stands on the login screen when its
//! terminal goes away held the whole fault of T-271 still.
//!
//! A measurement of 2026-08-16 of the real program v0.8.100, inside tmux, on a
//! screen of 160 columns and 45 rows, with a `XDG_CONFIG_HOME` that holds no
//! account and with `docs/harness/the_terminal_of_the_program_goes_away.py`:
//! `tmux kill-session` took the terminal away with no `SIGHUP`, and the program
//! stood at 71.7 percent of one processor after 34 seconds, with
//! `/proc/PID/fd/0 -> /dev/pts/5 (deleted)`. `strace -f -tt` of that program
//! named the loop: `read(0, "", 1024) = 0` came **1607222 times in 11.7
//! seconds** inside `crossterm::event::read`, the call of the loop of
//! `AppLogin::auth`. The corrected program of the same condition stopped after
//! **65 milliseconds**, and the log held the line of the fault one time.
//!
//! This test holds the decision of that task and the three lines of `main.rs`
//! that carry it: a session that takes one of them away takes the answer of the
//! program away with it.

use toutui::utils::the_terminal_that_went_away::the_line_of_a_terminal_of_the_login_screen_that_went_away;

/// The program starts the watch of the terminal of the login screen, and it
/// stops that watch at the end of the login screen. See T-272.
///
/// The parts of this test stay in one function.
#[test]
fn the_login_screen_holds_the_watch_of_its_terminal() {
    let main = include_str!("../src/main.rs");

    let the_watch = main
        .find("spawn_the_watch_of_the_terminal_of_the_login_screen()")
        .expect(
            "the program must start the watch of the terminal of the login screen: the loop \
             of that screen stands inside `crossterm::event::read`, a call that reads no byte \
             and that counts no event for a terminal that gives the end of its input, \
             therefore no `?` of that loop reaches the fault and the screen holds a whole \
             processor for ever (T-272)",
        );

    let the_login_screen = main
        .find("app_login.run(terminal)")
        .expect("the program holds the login screen");

    // **The watch starts before the loop of the login screen**, because that
    // loop never comes back for a terminal that went away.
    assert!(
        the_watch < the_login_screen,
        "the watch of the terminal must start before the loop of the login screen (T-272)"
    );

    let the_stop = main
        .find(".abort()")
        .expect("the program must stop the watch of the login screen (T-272)");

    // **The watch of T-271 holds the road after the login**, and that one closes
    // the session of the server and it sends the place of the user. Two watches
    // of one terminal give the road of this one to a program that holds a
    // session, and the place of the user then stays on the disk.
    assert!(
        the_login_screen < the_stop,
        "the watch of the login screen must stop at the end of that screen: the watch of \
         T-271 holds the road after it, and that one closes the session of the server (T-272)"
    );
}

/// The watch of the login screen closes no session, and it reads the `ioctl` of
/// the terminal. See T-272.
///
/// The parts of this test stay in one function.
#[test]
fn the_watch_of_the_login_screen_closes_no_session() {
    let watch = include_str!("../src/utils/the_terminal_that_went_away.rs");

    // The block of this function alone: the watch of T-271 stands above it, and
    // that one closes the session of the server.
    let start = watch
        .find("pub fn spawn_the_watch_of_the_terminal_of_the_login_screen()")
        .expect("the program holds the watch of the terminal of the login screen (T-272)");
    let block = &watch[start..];
    let block = &block[..block.find("\n#[cfg(test)]").unwrap_or(block.len())];

    // **The probe is `window_size` and not `size`** (T-271): a terminal that
    // went away inside tmux keeps the variables `COLUMNS` and `LINES`, and
    // `crossterm::terminal::size` falls back to them.
    assert!(
        block.contains("crossterm::terminal::window_size()"),
        "the watch of the login screen must read the `ioctl` of the terminal alone (T-272)"
    );

    // **The login screen stands before every account.** The program wrote no row
    // of an account here, it holds no client of the server, and it holds no
    // session: a request of this road names nothing at all.
    assert!(
        !block.contains("sync_session_from_database"),
        "the login screen holds no account and no session of the server, therefore this \
         watch closes no session (T-272)"
    );

    assert!(
        block.contains("crate::utils::exit_app::clean_exit()"),
        "a terminal of the login screen that went away must stop the program (T-272)"
    );

    // **The first look comes before the first wait** (T-271): a watch that waits
    // one second first reads a program whose terminal went away inside that
    // second as a program with no terminal at all.
    let the_look = block
        .find("watch.look(")
        .expect("the watch looks at the terminal");
    let the_wait = block
        .find("tokio::time::sleep(THE_TIME_BETWEEN_TWO_LOOKS)")
        .expect("the watch waits between two looks");

    assert!(
        the_look < the_wait,
        "the first look of the watch must come before the first wait (T-272)"
    );

    // The line of the log names the fault, and it says that this road closes no
    // session: the line of T-271 says that it closes one, and a maintainer reads
    // the two of them.
    let line = the_line_of_a_terminal_of_the_login_screen_that_went_away();
    assert!(line.contains("[the terminal]"), "{}", line);
    assert!(line.contains("Toutui stops"), "{}", line);
    assert!(line.contains("login screen"), "{}", line);
    assert!(
        line.contains("closes no session of the server"),
        "the words must say that this road closes no session: {}",
        line
    );
}

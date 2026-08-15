//! A program whose terminal went away stops. See T-271.
//!
//! **A terminal that goes away sends `SIGHUP` to the foreground process group
//! of that terminal alone.** A program that the user put in the background, a
//! program of `nohup`, and a program of a unit of systemd get no signal, and
//! the kernel then stops nothing.
//!
//! A measurement of 2026-08-16 of the real program v0.8.99, inside tmux, with
//! `docs/harness/the_terminal_of_the_program_goes_away.py`: `tmux kill-session`
//! took the terminal away, the program stood at 196 percent of one processor for
//! the whole of the measurement, and the log said nothing at all. `strace -f -tt`
//! of that program named the loop: `epoll_wait` gave `EPOLLHUP`, and
//! `read(0, "", 1024) = 0` came 439442 times in four seconds inside
//! `crossterm::event::poll`, a call that never comes back for a terminal that
//! gives the end of its input. Three programs of the sessions of 2026-08-15
//! stood in that state for three hours each.
//!
//! The correction is a task of one second. This test holds the decision of that
//! task, and the two lines that carry it: a session that takes one of them away
//! takes the answer of the program away with it.

use toutui::utils::the_terminal_that_went_away::{TheAnswerOfTheWatch, TheWatchOfTheTerminal};

/// The watch says what one answer of the terminal means. See T-271.
///
/// The parts of this test stay in one function.
#[test]
fn the_watch_of_the_terminal_holds_the_terminal_that_went_away() {
    let gone = || std::io::Error::other("Input/output error");

    // **The program watches a terminal that answered one time alone.** A child
    // that reads a PDF (T-62) and a run of a test have no terminal, and neither
    // of them must stop.
    let mut watch = TheWatchOfTheTerminal::new();
    assert_eq!(
        TheAnswerOfTheWatch::ThisProgramHasNoTerminal,
        watch.look::<(u16, u16)>(Err(gone())),
        "a program that never had a terminal loses none (T-271)"
    );

    // The terminal of the program answers, and it then goes away.
    let mut watch = TheWatchOfTheTerminal::new();
    assert_eq!(
        TheAnswerOfTheWatch::TheTerminalStays,
        watch.look(Ok((160u16, 45u16)))
    );
    assert_eq!(
        TheAnswerOfTheWatch::TheTerminalWentAway,
        watch.look::<(u16, u16)>(Err(gone())),
        "a terminal that answered and that answers no more went away (T-271)"
    );
}

/// The program starts the watch, and the watch reads the `ioctl` of the
/// terminal. See T-271.
#[test]
fn the_program_holds_the_watch_of_its_terminal() {
    let main = include_str!("../src/main.rs");

    assert!(
        main.contains("spawn_the_watch_of_the_terminal("),
        "the program must start the task that watches its terminal: the loop of \
         the screen stands inside a call of crossterm that never comes back for \
         a terminal that went away, therefore no `?` of that loop reaches the \
         fault (T-271)"
    );

    let watch = include_str!("../src/utils/the_terminal_that_went_away.rs");

    // **The probe is `window_size` and not `size`.**
    // `crossterm::terminal::size` gives a fallback of `tput` and of the
    // variables `COLUMNS` and `LINES`, and a terminal that went away inside
    // tmux keeps those variables: a measurement of 2026-08-16 with that call
    // gave 15 answers of `Ok` after the terminal went away.
    assert!(
        watch.contains("crossterm::terminal::window_size()"),
        "the watch must read the `ioctl` of the terminal alone (T-271)"
    );

    assert!(
        !watch.contains("crossterm::terminal::size()"),
        "`crossterm::terminal::size` answers `Ok` for a terminal that went away, \
         because it falls back to `tput` and to the variables COLUMNS and LINES \
         (T-271)"
    );

    // **The first look comes before the first wait.** A watch that waits one
    // second first reads a program whose terminal went away inside that second
    // as a program with no terminal at all.
    let start = watch
        .find("pub fn spawn_the_watch_of_the_terminal(")
        .expect("the program holds the watch of its terminal");
    let block = &watch[start..];
    let the_look = block
        .find("watch.look(")
        .expect("the watch looks at the terminal");
    let the_wait = block
        .find("tokio::time::sleep(THE_TIME_BETWEEN_TWO_LOOKS)")
        .expect("the watch waits between two looks");

    assert!(
        the_look < the_wait,
        "the first look of the watch must come before the first wait: a terminal \
         that goes away inside that wait then reads as a program with no \
         terminal at all (T-271)"
    );
}

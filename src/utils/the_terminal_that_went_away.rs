//! The terminal of the program goes away, and the program stops. See T-271.
//!
//! **A terminal that goes away sends `SIGHUP` to the foreground process group
//! of that terminal, and to nothing else.** A program that the user put in the
//! background, a program of `nohup`, a program of a unit of systemd, and a
//! program whose shell died with no signal of its own each keep the terminal
//! that no process holds. The kernel then stops nothing at all.
//!
//! A measurement of 2026-08-16 of the real program, inside tmux, with
//! `docs/harness/the_terminal_of_the_program_goes_away.py`: `tmux kill-session`
//! took the terminal away, and the program stood for the whole of the
//! measurement at 196 percent of one processor. `strace -f -tt` of that program
//! named the loop:
//!
//! ```text
//! epoll_wait(19, [{events=EPOLLIN|EPOLLERR|EPOLLHUP, data=0}], 3, 200) = 1
//! read(0, "", 1024)       = 0
//! read(0, "", 1024)       = 0
//! …
//! ```
//!
//! 439442 reads of no byte in four seconds. **`crossterm::event::poll` never
//! comes back on a terminal that gives the end of its input**: it reads no
//! byte, it counts no event, and it waits again. Therefore the `?` of that call
//! reaches nothing, the loop of the screen draws no frame after it, and no
//! write of the program meets the fault of the terminal. Three programs of the
//! sessions of 2026-08-15 stood in that state for three hours each, at 131
//! percent of one processor, with a row of `listening_session` whose heartbeat
//! stayed fresh: a second program of the account cannot take such a row (T-140),
//! therefore the place of the user stands in a program that the user cannot see.
//!
//! **The loop of the screen cannot hold the answer**, because the fault comes
//! while that loop stands inside the call that never comes back. A task of one
//! second holds it: `crossterm::terminal::window_size` is one `ioctl` of the
//! terminal, and that `ioctl` gives `Input/output error` for a terminal of a
//! pseudo-terminal whose other side went away.
//!
//! **The program watches a terminal that answered one time alone.** A program
//! that never had a terminal must lose none: the first answer of the watch is
//! the condition of every answer after it.

use std::io;

/// What one look of the watch says.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheAnswerOfTheWatch {
    /// The terminal answered. The program continues.
    TheTerminalStays,
    /// The terminal answered a look before this one, and it answers no more.
    /// The program stops.
    TheTerminalWentAway,
    /// No look of this watch got an answer. This program has no terminal to
    /// lose, therefore the watch stops nothing.
    ThisProgramHasNoTerminal,
}

/// The watch of the terminal of this program.
///
/// The caller gives the answer of the terminal at each look, therefore a test
/// needs no terminal at all.
#[derive(Debug, Default, Clone, Copy)]
pub struct TheWatchOfTheTerminal {
    the_terminal_answered_one_time: bool,
}

impl TheWatchOfTheTerminal {
    /// Makes a watch that got no answer.
    pub fn new() -> TheWatchOfTheTerminal {
        TheWatchOfTheTerminal::default()
    }

    /// Reads one answer of the terminal, and it says what that answer means.
    pub fn look<T>(&mut self, the_answer: io::Result<T>) -> TheAnswerOfTheWatch {
        match the_answer {
            Ok(_) => {
                self.the_terminal_answered_one_time = true;
                TheAnswerOfTheWatch::TheTerminalStays
            }
            Err(_) if self.the_terminal_answered_one_time => {
                TheAnswerOfTheWatch::TheTerminalWentAway
            }
            Err(_) => TheAnswerOfTheWatch::ThisProgramHasNoTerminal,
        }
    }
}

/// The time between two looks of the watch.
///
/// A program whose terminal went away holds a whole processor. One second of
/// that state costs the user nothing, and a look of each second costs one
/// `ioctl`.
pub const THE_TIME_BETWEEN_TWO_LOOKS: std::time::Duration = std::time::Duration::from_secs(1);

/// The line of the log of a terminal that went away.
///
/// **The user reads no word of this fault**, because the screen of the user is
/// the terminal that went away (T-177). The maintainer reads the log.
pub fn the_line_of_a_terminal_that_went_away() -> String {
    String::from(
        "[the terminal] the terminal of this program went away, and no signal came with it. \
         Toutui stops: a program of a terminal that no process holds gives no screen and no \
         key, and it keeps a whole processor. Toutui closes the session of the server first, \
         therefore the place of the user stays.",
    )
}

/// Starts the task that watches the terminal of this program. See T-271.
///
/// **The task stands on a thread of the runtime**, therefore the loop of the
/// screen that never comes back holds it nowhere.
///
/// A terminal that went away takes the road of the key `Q`: the place of the
/// book of the reader goes to the server (T-292), and
/// `sync_session_from_database` then closes the session of the server, it sends
/// the place of the audio, and it stops the program. The user reads no word of
/// it, because the screen of the user is the terminal that went away (T-177).
pub fn spawn_the_watch_of_the_terminal(
    api: std::sync::Arc<crate::api::client::ApiClient>,
    username: String,
    server: String,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut watch = TheWatchOfTheTerminal::new();

        loop {
            // **The first look comes before the first wait.** The terminal of
            // the program stands at the moment of this line, therefore that
            // look tells the watch that this program has a terminal. A watch
            // that waits one second first meets a terminal that went away
            // inside that second, and it then reads that program as a program
            // with no terminal at all.
            //
            // **The probe is `window_size` and not `size`.**
            // `crossterm::terminal::size` gives the `ioctl` of the terminal
            // first, and a fallback of `tput` and of the variables `COLUMNS`
            // and `LINES` after it: a terminal that went away inside tmux keeps
            // those variables, therefore that call answers `Ok` for ever.
            // `window_size` is the `ioctl` alone, and it gives
            // `Input/output error` for a pseudo-terminal whose other side went
            // away. See T-271.
            match watch.look(crossterm::terminal::window_size()) {
                TheAnswerOfTheWatch::TheTerminalStays => {}

                // A program that never had a terminal loses none. The child
                // that reads a PDF and a run of a test stand here.
                TheAnswerOfTheWatch::ThisProgramHasNoTerminal => {}

                TheAnswerOfTheWatch::TheTerminalWentAway => {
                    log::error!("{}", the_line_of_a_terminal_that_went_away());

                    // The place of the reader goes before the process ends. A
                    // terminal that went away took every line that the user
                    // read with it. See T-292, and T-294 for the place that the
                    // server refuses.
                    crate::logic::reader::the_place_that_waits::
                        the_place_of_the_reader_goes_to_the_server(
                            &api,
                            &username,
                            &server,
                            "the terminal that went away",
                        )
                        .await;

                    crate::logic::sync_session::sync_session_from_database::
                        sync_session_from_database(
                            &api,
                            username.clone(),
                            server.clone(),
                            true,
                            "the terminal that went away",
                        )
                        .await;

                    // `sync_session_from_database` stops the program with the
                    // value `true` above. This line holds the road of a change
                    // of that function.
                    crate::utils::exit_app::clean_exit();
                }
            }

            tokio::time::sleep(THE_TIME_BETWEEN_TWO_LOOKS).await;
        }
    })
}

/// The line of the log of a terminal of the login screen that went away.
///
/// **The login screen stands before every account**, therefore this program
/// holds no name of an account, no address of a server, and no session of the
/// server here. The words say that, because a maintainer who reads
/// `the_line_of_a_terminal_that_went_away` of the line above expects a session
/// that closes, and this road closes none.
pub fn the_line_of_a_terminal_of_the_login_screen_that_went_away() -> String {
    String::from(
        "[the terminal] the terminal of this program went away, and no signal came with it. \
         Toutui stops: the login screen of a terminal that no process holds gives no screen \
         and no key, and it keeps a whole processor. The user has no account on this screen, \
         therefore Toutui closes no session of the server and it sends no place.",
    )
}

/// Starts the task that watches the terminal of the login screen. See T-272.
///
/// **The login screen holds a loop of its own**, and that loop stands inside
/// `crossterm::event::read`, which reads no byte and which counts no event for
/// a terminal that gives the end of its input. The watch of
/// `spawn_the_watch_of_the_terminal` cannot hold this road: it needs the client
/// of the server, the name of an account, and the name of a server, and the
/// login screen stands before every one of them.
///
/// **A terminal of this screen that went away takes the road of the key `Esc`**:
/// the program wrote no row of an account, it holds no session of the server,
/// and it therefore stops with no request at all.
///
/// The caller stops this task at the end of the login screen, because the watch
/// of `spawn_the_watch_of_the_terminal` holds the road after it, and that one
/// closes the session of the server.
pub fn spawn_the_watch_of_the_terminal_of_the_login_screen() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut watch = TheWatchOfTheTerminal::new();

        loop {
            // The first look comes before the first wait, and the probe is
            // `window_size` and not `size`. See the comments of
            // `spawn_the_watch_of_the_terminal` above.
            match watch.look(crossterm::terminal::window_size()) {
                TheAnswerOfTheWatch::TheTerminalStays => {}

                TheAnswerOfTheWatch::ThisProgramHasNoTerminal => {}

                TheAnswerOfTheWatch::TheTerminalWentAway => {
                    log::error!(
                        "{}",
                        the_line_of_a_terminal_of_the_login_screen_that_went_away()
                    );

                    crate::utils::exit_app::clean_exit();
                }
            }

            tokio::time::sleep(THE_TIME_BETWEEN_TWO_LOOKS).await;
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A terminal that answered and that answers no more went away.** T-271.
    ///
    /// The parts of this test stay in one function: the box of the process of a
    /// module holds one slot, and two test functions of one module fight for it.
    #[test]
    fn the_watch_of_the_terminal_says_what_one_look_means() {
        let gone = || io::Error::other("Input/output error");

        // A watch that got no answer at all watches nothing. A program with no
        // terminal must not stop.
        let mut watch = TheWatchOfTheTerminal::new();
        assert_eq!(
            TheAnswerOfTheWatch::ThisProgramHasNoTerminal,
            watch.look::<(u16, u16)>(Err(gone()))
        );
        assert_eq!(
            TheAnswerOfTheWatch::ThisProgramHasNoTerminal,
            watch.look::<(u16, u16)>(Err(gone())),
            "the fault of a program with no terminal stays the same"
        );

        // The terminal of this program answers.
        let mut watch = TheWatchOfTheTerminal::new();
        assert_eq!(
            TheAnswerOfTheWatch::TheTerminalStays,
            watch.look(Ok((160u16, 45u16)))
        );
        assert_eq!(
            TheAnswerOfTheWatch::TheTerminalStays,
            watch.look(Ok((80u16, 24u16))),
            "a terminal of another size stays the same terminal"
        );

        // And it goes away.
        assert_eq!(
            TheAnswerOfTheWatch::TheTerminalWentAway,
            watch.look::<(u16, u16)>(Err(gone()))
        );

        // A terminal that went away does not come back: the program stops at
        // the first answer of this kind, and a second look says the same.
        assert_eq!(
            TheAnswerOfTheWatch::TheTerminalWentAway,
            watch.look::<(u16, u16)>(Err(gone()))
        );

        // The line of the log names the fault and the road of the program.
        let line = the_line_of_a_terminal_that_went_away();
        assert!(line.contains("[the terminal]"), "{}", line);
        assert!(line.contains("Toutui stops"), "{}", line);
        assert!(
            line.contains("closes the session of the server"),
            "{}",
            line
        );

        // The look of each second costs one `ioctl`, and a program that went
        // away holds a whole processor for that second.
        assert_eq!(
            std::time::Duration::from_secs(1),
            THE_TIME_BETWEEN_TWO_LOOKS
        );
    }
}

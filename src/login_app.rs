use crate::config::*;
use color_eyre::Result;
use ratatui::DefaultTerminal;
use std::io;

pub enum AppViewLogin {
    Auth,
}

pub struct AppLogin {
    pub view_state: AppViewLogin,
    pub should_exit: bool,
    pub config: ConfigFile,
    /// The fault of the login screen. See T-275.
    ///
    /// **A `Widget` gives no answer**, therefore the render of this application
    /// held `let _ = self.auth()` and every fault of the login screen went away
    /// there: the loop of `src/main.rs` then made a terminal again, and the
    /// words of the user said that the program found no terminal. The
    /// application keeps that fault here, and `run` gives it to the caller.
    pub the_fault_of_the_screen: Option<io::Error>,
}

/// The answer of one frame of the login screen. See T-275.
///
/// **The login screen draws inside the frame of the loop of `run`**, and it
/// holds a terminal of its own: its fault therefore comes first, because the
/// frame of that loop holds no character of the login screen. A standard output
/// that failed gives the two faults together, and the user must read the one of
/// the screen that they looked at.
pub fn the_end_of_a_frame_of_the_login(
    of_the_screen: Option<io::Error>,
    of_the_frame: io::Result<()>,
) -> io::Result<()> {
    match of_the_screen {
        Some(fault) => Err(fault),
        None => of_the_frame,
    }
}

/// Init app
impl AppLogin {
    pub async fn new() -> Result<Self> {
        // init config
        let config = load_config()?;

        // init view_state
        let view_state = AppViewLogin::Auth;
        Ok(Self {
            should_exit: false,
            view_state,
            config,
            the_fault_of_the_screen: None,
        })
    }

    /// handle events
    ///
    /// **The answer of this function is the fault of the login screen** (T-275).
    /// The caller stops the program with words of that fault: a screen that did
    /// not reach the terminal gives no field and no message to the user, and a
    /// loop that starts it again says nothing at all.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while !self.should_exit {
            let of_the_frame = terminal.draw(|frame| frame.render_widget(&mut self, frame.area()));

            if let Some(fault) = the_end_of_a_frame_of_the_login(
                self.the_fault_of_the_screen.take(),
                of_the_frame.map(|_| ()),
            )
            .err()
            {
                return Err(fault);
            }
        }
        Ok(())
    }
}

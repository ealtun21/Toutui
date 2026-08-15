use crate::login_app::AppLogin;
use crate::login_app::AppViewLogin;
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

/// init widget for selected AppView
impl Widget for &mut AppLogin {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.view_state {
            AppViewLogin::Auth => self.render_auth(area, buf),
        }
    }
}

/// Rendering logic
impl AppLogin {
    fn render_auth(&mut self, _area: Rect, _buf: &mut Buffer) {
        // T-275. The old line of this place dropped every fault of the login
        // screen with an answer of no name: a standard output that failed gave
        // the user no
        // field, no message, and no word of a reason, and the loop of
        // `src/main.rs` then made a terminal again and said that the program
        // found no terminal (T-91). A `Widget` gives no answer, therefore the
        // fault stands in the application, and `AppLogin::run` gives it back.
        if let Err(fault) = self.auth() {
            self.the_fault_of_the_screen = Some(fault);
            self.should_exit = true;
        }
    }
}

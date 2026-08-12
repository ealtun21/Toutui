use crate::api::server::auth_process::*;
use crate::config::rgb_parts;
use crate::db::crud::*;
use crate::login_app::AppLogin;
use crate::utils::exit_app::*;
use crossterm::event::{self, KeyCode, KeyEvent};
use log::{error, info};
use ratatui::backend::CrosstermBackend;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::{Block, Borders};
use ratatui::Terminal;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
};
use std::io;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::ui::text_field::field_view;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The character that the password field shows in place of a character of the
/// password.
const MASK: char = '\u{2022}';

/// The place of the field of the address in the list of the fields.
const ADDRESS_FIELD: usize = 0;

/// The address that the user gave and that the server answered. See T-92.
///
/// **A login that fails starts this screen again**, and every field was empty
/// again: the user wrote the whole address of their server after each wrong
/// password. The address answered `/ping` already, therefore the program keeps
/// it and it writes it in the field.
///
/// The value belongs to the process, and the login screen runs one time at a
/// time. A login that succeeds leaves this loop, therefore no old value stays.
///
/// The first value comes from the environment, because a token that the server
/// refused starts the program again: that address belongs to the process before
/// this one. See T-123.
fn the_address_that_answered() -> &'static std::sync::Mutex<Option<String>> {
    static ADDRESS: std::sync::OnceLock<std::sync::Mutex<Option<String>>> =
        std::sync::OnceLock::new();
    ADDRESS.get_or_init(|| {
        std::sync::Mutex::new(
            std::env::var(THE_ADDRESS_OF_THE_LOGIN)
                .ok()
                .filter(|address| !address.is_empty()),
        )
    })
}

/// The name of the variable of the environment that holds the address of the
/// login screen. See T-123.
///
/// **A user does not write this variable.** The program writes it for itself,
/// when a token that the server refused starts the program again.
pub const THE_ADDRESS_OF_THE_LOGIN: &str = "TOUTUI_THE_ADDRESS_OF_THE_LOGIN";

/// Writes the address that the login screen shows in its first field.
///
/// **A token that the server refused sends the user to the login screen**, and
/// the address of that account is known: the user must not write it again. See
/// T-123 and T-92.
pub fn the_login_starts_with_this_address(address: &str) {
    if address.is_empty() {
        return;
    }

    if let Ok(mut place) = the_address_that_answered().lock() {
        *place = Some(address.to_string());
    }
}

/// Makes the program ready for the login screen, after the server refused the
/// token. See T-123.
///
/// **The old code let the fault leave `main`.** The user then read
/// `Error: The token is not valid. Log in again.`, a location inside the source
/// of the program, and nothing else: no view of the program came, and no key
/// gave a new token. A user who moves from the program before this fork meets
/// this at the first start, because the token of that database is a token of a
/// server that this account no longer holds.
///
/// The three steps:
///
/// - the row of the account goes away, because a row that stays sends the
///   program to the same fault at once. The rows of the downloads, of the
///   queue, and of the positions that wait hold the name of the account only,
///   therefore a login with the same name finds all of them again;
/// - the login screen shows the reason and the address of the server;
/// - the program starts again, and the login screen of a first start comes.
///
/// **The program starts again, and it does not make the login screen inside
/// this process.** A second start of the terminal inside one process gave a
/// screen of no character, and the process also holds tasks of the token that
/// the server refused. See `start_the_program_again`. A system that has no
/// `exec` gives an answer here, and the caller then makes the login screen
/// itself.
pub fn the_program_needs_a_new_token(username: &str, server_address: &str) -> Result<(), String> {
    if let Err(error) = crate::db::crud::remove_the_account(username) {
        // The row stays, therefore the login screen would give the same fault
        // for ever. The caller says the fault of the API instead.
        return Err(format!(
            "The account {} stays in the database: {}",
            username, error
        ));
    }

    let _ = update_login_err("The token is not valid. Log in again.");

    info!(
        "[auth] the server refused the token of {}. The login screen comes again.",
        username
    );

    // The program starts again here, therefore no line after this one runs. The
    // address of the server stands in the database of the login screen of the
    // new process, and not in a value of this process.
    let of_the_system = crate::utils::exit_app::start_the_program_again(server_address);

    error!(
        "[auth] the program does not start again: {}. The login screen comes inside this process.",
        of_the_system
    );

    // The program stays, therefore the login screen of this process needs the
    // address.
    the_login_starts_with_this_address(server_address);

    Ok(())
}

/// Examines the address of the server before the login asks for a password.
///
/// The function looks at the form of the address first, because that needs no
/// network. Then it asks the address for `/ping`. Every Audiobookshelf server
/// answers that path, and it needs no token.
///
/// This function is not asynchronous, because it runs inside the loop of the
/// screen. Therefore the request runs on its own thread with its own runtime,
/// as the login does. See T-15.
fn check_the_address(written: &str) -> Result<String, String> {
    let address = crate::api::server::address::check_shape(written)?;
    let for_the_thread = address.clone();

    let outcome = std::thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => return Err(format!("No runtime for the request: {}", error)),
        };

        runtime.block_on(crate::api::server::address::ask_ping(&for_the_thread))
    })
    .join();

    match outcome {
        Ok(Ok(())) => Ok(address),
        Ok(Err(message)) => Err(message),
        Err(_) => Err("The examination of the address stopped.".to_string()),
    }
}

/// The parts of the login screen that one frame holds.
pub struct TheLoginScreen<'a> {
    /// The name of the field that the user writes in.
    pub title: &'a str,
    /// The text of the field. A field with a mask holds the mask here.
    pub text: &'a str,
    /// An empty field shows the text that tells the user what to write, and
    /// that text is dark: nobody must read it as an answer.
    pub the_text_tells_what_to_write: bool,
    /// The number of columns that the field hides at the left.
    pub scroll: u16,
    /// The column of the cursor inside the field.
    pub cursor: u16,
    /// The message of the login. An empty text is no message.
    pub message: &'a str,
    /// The colour of the border of the field.
    pub of_the_border: (u8, u8, u8),
    /// The colour of the screen behind the field.
    pub of_the_background: (u8, u8, u8),
    /// The colour behind the message.
    pub of_the_message: (u8, u8, u8),
}

/// Gives the area of the field on a screen of one size.
///
/// The function is pure, therefore a test needs no terminal. **The size comes
/// at each frame**, as the box of `crate::logic::prompt` takes it: a terminal
/// that becomes small while the login screen stands would otherwise hold a
/// field outside the screen. See T-115.
pub fn the_area_of_the_field(size: ratatui::layout::Size) -> Rect {
    let width = size.width / 2;

    Rect {
        x: (size.width.saturating_sub(width)) / 2,
        y: size.height.saturating_sub(3) / 2,
        width,
        height: 3,
    }
}

/// Gives the row of the message of the login.
pub fn the_row_of_the_message(size: ratatui::layout::Size) -> u16 {
    size.height.saturating_sub(6)
}

/// Draws one frame of the login screen.
///
/// **Every cell of this screen belongs to the frame of ratatui, and the cursor
/// belongs to the field.** The old code wrote the message with
/// `crossterm::cursor::MoveTo` and `println!` after the frame: the terminal then
/// kept its cursor at the end of that row, and the user wrote their password in
/// a field while the cursor of the terminal stood six rows below it. The message
/// also stood outside the buffer, therefore it could stay after the work of it
/// ended. See T-134, T-42, and T-59.
pub fn draw_the_login(frame: &mut ratatui::Frame, screen: &TheLoginScreen) {
    let whole = frame.area();
    let area = the_area_of_the_field(ratatui::layout::Size::new(whole.width, whole.height));

    let (r, g, b) = screen.of_the_border;
    let block = Block::default()
        .borders(Borders::ALL)
        .title(screen.title.to_string())
        .title_bottom(Line::from(format!("🦜Toutui v{} - Esc to quit.", VERSION)).right_aligned())
        .border_style(Style::default().fg(Color::Rgb(r, g, b)));

    let style = if screen.the_text_tells_what_to_write {
        Style::default().fg(Color::Rgb(128, 128, 128))
    } else {
        Style::default()
    };

    let line = Paragraph::new(screen.text)
        .style(style)
        .scroll((0, screen.scroll))
        .block(block);

    frame.render_widget(line, area);

    let (r, g, b) = screen.of_the_background;
    frame.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(r, g, b))),
        whole,
    );

    // The message stands inside the frame. A frame with no message writes the
    // cells of that row again, therefore no old message stays.
    if !screen.message.is_empty() {
        let (r, g, b) = screen.of_the_message;
        let row = Rect {
            x: 0,
            y: the_row_of_the_message(ratatui::layout::Size::new(whole.width, whole.height)),
            width: whole.width,
            height: 1,
        };

        frame.render_widget(
            Paragraph::new(screen.message).style(Style::default().bg(Color::Rgb(r, g, b))),
            row,
        );
    }

    // The cursor comes last, and it stands in the field.
    frame.set_cursor_position((area.x + 1 + screen.cursor, area.y + 1));
}

/// One field of the login screen.
struct Field {
    title: &'static str,
    /// The text that the field shows when it is empty.
    placeholder: &'static str,
    /// A field with a mask never shows what the user wrote.
    mask: Option<char>,
    input: Input,
}

impl AppLogin {
    pub fn auth(&mut self) -> io::Result<()> {
        info!("[auth_input] Login");

        // init input area
        let stdout = io::stdout();
        let stdout = stdout.lock();

        let backend = CrosstermBackend::new(stdout);
        let mut term = Terminal::new(backend)?;

        let (fg_r, fg_g, fg_b) = rgb_parts(&self.config.colors.login_foreground_color);

        let mut fields = [
            Field {
                title: "Server address",
                placeholder: "http:// or https:// required",
                mask: None,
                // The address of the attempt before this one. See T-92.
                input: match the_address_that_answered().lock() {
                    Ok(place) => match place.clone() {
                        Some(address) => Input::default().with_value(address),
                        None => Input::default(),
                    },
                    Err(_) => Input::default(),
                },
            },
            Field {
                title: "Username",
                placeholder: "",
                mask: None,
                input: Input::default(),
            },
            Field {
                title: "Password",
                placeholder: "",
                mask: Some(MASK),
                input: Input::default(),
            },
        ];

        // init variables
        let mut current_index = 0;
        let mut collected_data: Vec<String> = Vec::new();
        let (log_r, log_g, log_b) = rgb_parts(&self.config.colors.log_background_color);
        let (bg_r, bg_g, bg_b) = rgb_parts(&self.config.colors.background_color);

        loop {
            // **The size comes at each turn of this loop.** The old code took it
            // one time, therefore a terminal that became small held a field
            // outside the screen. See T-115.
            let size = term.size()?;
            // The borders take one column at the left and one column at the
            // right.
            let inner_width = the_area_of_the_field(size).width.saturating_sub(2);

            let field = &fields[current_index];
            let view = field_view(&field.input, inner_width, field.mask);

            // The message of the login stands inside the frame. See T-134.
            let message = match get_others() {
                Ok(Some(value)) => value.login_err,
                Ok(None) => "".to_string(),
                Err(e) => {
                    info!("ERROR: Failed to get login error: {}", e);
                    "".to_string()
                }
            };

            let the_text_tells_what_to_write =
                view.text.is_empty() && !field.placeholder.is_empty();
            let text = if the_text_tells_what_to_write {
                field.placeholder
            } else {
                view.text.as_str()
            };

            let screen = TheLoginScreen {
                title: field.title,
                text,
                the_text_tells_what_to_write,
                scroll: view.scroll,
                cursor: view.cursor,
                message: message.as_str(),
                of_the_border: (fg_r, fg_g, fg_b),
                of_the_background: (log_r, log_g, log_b),
                of_the_message: (bg_r, bg_g, bg_b),
            };

            term.draw(|frame| draw_the_login(frame, &screen))?;

            match crossterm::event::read()? {
                event::Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }) => {
                    // The address of the server is examined here, and not
                    // after the password. The old code sent the three fields
                    // together, therefore an address with no `http://` failed
                    // after the user wrote everything. See T-45.
                    if current_index == ADDRESS_FIELD {
                        let written = fields[ADDRESS_FIELD].input.value().to_string();

                        match check_the_address(&written) {
                            Ok(address) => {
                                let _ = update_login_err("");
                                fields[ADDRESS_FIELD].input =
                                    Input::default().with_value(address.clone());

                                // The next attempt of the login starts with
                                // this address. See T-92.
                                if let Ok(mut place) = the_address_that_answered().lock() {
                                    *place = Some(address.clone());
                                }

                                collected_data.push(address);
                                current_index += 1;
                            }
                            Err(message) => {
                                let _ = update_login_err(message.as_str());
                            }
                        }

                        continue;
                    }

                    // **A field of no letter needs no request.** The field of
                    // the address held this rule already, and the two other
                    // fields did not: the sweep of the login of 2026-08-11
                    // pressed Enter on an empty username, and the view went to
                    // the password with no word. See T-92.
                    let written = fields[current_index].input.value().to_string();

                    if written.is_empty() {
                        let _ = update_login_err(match current_index {
                            1 => "Write your username.",
                            _ => "Write your password.",
                        });

                        continue;
                    }

                    let _ = update_login_err("");

                    if current_index < fields.len() - 1 {
                        // The loop takes the second field here. It takes the
                        // third field after the break.
                        collected_data.push(written);
                        current_index += 1;
                    } else {
                        break;
                    }
                }

                event::Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                }) => {
                    let _ = update_login_err("");
                    clean_exit();
                }

                other => {
                    if let Some(field) = fields.get_mut(current_index) {
                        field.input.handle_event(&other);
                    }
                }
            }
        }

        // save the last input (from the password field)
        collected_data.push(fields[current_index].input.value().to_string());

        // make disappear search_area (the input bar) after the break loop
        term.draw(|f| {
            let empty_block = Block::default();
            let area = the_area_of_the_field(f.area().as_size());
            f.render_widget(empty_block, area);
        })?;

        // Fetch data from api and insert them in database

        // send result
        if let Some(_active_field) = fields.get(current_index) {
            let collected_data_clone = collected_data.clone();

            // The login must finish before the application continues.
            //
            // The old code started the login with `tokio::spawn` and did not
            // wait for it. The application then read the database before the
            // login wrote the user. Therefore the first attempt failed, and
            // the second attempt found the user and worked. See T-15.
            //
            // This function is not asynchronous, because it runs inside
            // `Widget::render`. Therefore the login runs on its own thread
            // with its own runtime, and this thread waits for it.
            let outcome = std::thread::spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(error) => return Err(format!("No runtime for the login: {}", error)),
                };

                runtime.block_on(async move {
                    auth_process(
                        collected_data_clone[1].as_str(), // username
                        collected_data_clone[2].as_str(), // password
                        collected_data_clone[0].as_str(), // server_address
                    )
                    .await
                    .map_err(|error| error.to_string())
                })
            })
            .join();

            match outcome {
                Ok(Ok(())) => {
                    info!("[auth_process] Login successful");
                    let _ = update_login_err("");
                }
                Ok(Err(error)) => {
                    error!("[auth_process] Login failed: {}", error);

                    // **The sentence of the fault reaches the user as it
                    // stands.** The old code wrote "ERROR: " before it, and no
                    // other message of the program holds that word: the two
                    // other messages of this view are "The address must start
                    // with http:// or https://" and "… does not answer. Is the
                    // server running?". See T-92.
                    let _ = update_login_err(error.as_str());
                }
                Err(_) => {
                    error!("[auth_process] The login thread stopped");
                    let _ = update_login_err("The login stopped. Try it again.");
                }
            }

            // to quit the current thread and back to login or home (if connection is successful)
            // should_exit allow to quit the terminal in login_app.rs
            print!("\x1B[2J\x1B[1;1H"); // clean all prints displayed
            self.should_exit = true;

            Ok(())
        } else {
            Err(io::Error::other("Invalid textarea"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::layout::{Position, Size};

    fn a_screen<'a>(text: &'a str, message: &'a str) -> TheLoginScreen<'a> {
        TheLoginScreen {
            title: "Server address",
            text,
            the_text_tells_what_to_write: false,
            scroll: 0,
            cursor: text.chars().count() as u16,
            message,
            of_the_border: (255, 255, 255),
            of_the_background: (0, 0, 0),
            of_the_message: (0, 0, 0),
        }
    }

    fn the_lines(terminal: &Terminal<TestBackend>) -> Vec<String> {
        let buffer = terminal.backend().buffer();

        (0..buffer.area.height)
            .map(|row| {
                (0..buffer.area.width)
                    .map(|column| buffer[(column, row)].symbol())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect()
    }

    /// **The cursor stands in the field, and the message takes it nowhere.**
    ///
    /// The old code wrote the message with `MoveTo` and `println!` after the
    /// frame of ratatui: a measurement of 2026-08-13 in a terminal of 100 by 30
    /// read the cursor of the terminal at the column 69 of the row 25, the end
    /// of the message, while the field of the user stood at the row 15. The
    /// user wrote their password with no cursor beside it. See T-134.
    #[test]
    fn the_cursor_stands_in_the_field_while_a_message_stands() {
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        let message = "The address must start with http:// or https://";

        terminal
            .draw(|frame| draw_the_login(frame, &a_screen("notaurl", message)))
            .unwrap();

        let area = the_area_of_the_field(Size::new(100, 30));
        assert_eq!(
            terminal.get_cursor_position().unwrap(),
            Position::new(area.x + 1 + 7, area.y + 1)
        );

        // The message stands on its own row, inside the frame.
        let lines = the_lines(&terminal);
        assert_eq!(
            lines[usize::from(the_row_of_the_message(Size::new(100, 30)))],
            message
        );
        assert!(lines[usize::from(area.y) + 1].contains("notaurl"));
    }

    /// A frame with no message writes the cells of that row again, therefore no
    /// old message stays. The old code needed `clear_message` for that work, and
    /// a message that was shorter than the message before it kept the end of the
    /// old one. See T-134.
    #[test]
    fn a_message_goes_away_with_the_frame_that_holds_it() {
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();

        terminal
            .draw(|frame| draw_the_login(frame, &a_screen("", "The server took too many attempts")))
            .unwrap();
        terminal
            .draw(|frame| draw_the_login(frame, &a_screen("", "Wait")))
            .unwrap();

        let row = usize::from(the_row_of_the_message(Size::new(100, 30)));
        assert_eq!(the_lines(&terminal)[row], "Wait");

        terminal
            .draw(|frame| draw_the_login(frame, &a_screen("", "")))
            .unwrap();

        assert_eq!(the_lines(&terminal)[row], "");
    }

    /// The field stands on the screen of now. A terminal that becomes small
    /// while the login screen stands holds no field outside it. See T-115.
    #[test]
    fn the_field_stands_inside_the_screen() {
        for (width, height) in [(160u16, 45u16), (80, 24), (20, 4), (2, 1)] {
            let area = the_area_of_the_field(Size::new(width, height));

            assert!(area.x + area.width <= width, "{} by {}", width, height);
            assert!(area.y <= height, "{} by {}", width, height);
        }

        let area = the_area_of_the_field(Size::new(100, 30));
        assert_eq!(area, Rect::new(25, 13, 50, 3));
    }
}

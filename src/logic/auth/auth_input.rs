use crate::login_app::AppLogin;
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::{Block, Borders};
use ratatui::text::Line;
use ratatui::Terminal;
use std::io;
use tui_textarea::TextArea;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
};
use crate::api::server::auth_process::*;
use crossterm::event::{self, KeyEvent, KeyCode};  
use log::{info, error};
use crate::utils::exit_app::*;
use crate::utils::pop_up_message::*;
use crate::db::crud::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");


impl AppLogin {
    pub fn auth(&mut self) -> io::Result<()> {
        info!("[auth_input] Login");

        // init input area
        let stdout = io::stdout();
        let stdout = stdout.lock();

        let backend = CrosstermBackend::new(stdout);
        let mut term = Terminal::new(backend)?;

        let fg_color = self.config.colors.login_foreground_color.clone();

        let mut textarea1 = TextArea::default();
        textarea1.set_block(
            Block::default()
            .borders(Borders::ALL)
            .title("Server address")
            .title_bottom(Line::from(format!("🦜Toutui v{} - Esc to quit.", VERSION)).right_aligned())
            .border_style(Style::default()
                .fg(Color::Rgb(fg_color[0], fg_color[1], fg_color[2])))
        );

        textarea1.set_placeholder_text("http:// or https:// required");

        let mut textarea2 = TextArea::default();
        textarea2.set_block(
            Block::default()
            .borders(Borders::ALL)
            .title("Username")
            .title_bottom(Line::from(format!("🦜Toutui v{} - Esc to quit.", VERSION)).right_aligned())
            .border_style(Style::default()
                .fg(Color::Rgb(fg_color[0], fg_color[1], fg_color[2])))
        );

        let mut textarea3 = TextArea::default();
        textarea3.set_block(
            Block::default()
            .borders(Borders::ALL)
            .title("Password")
            .title_bottom(Line::from(format!("🦜Toutui v{} - Esc to quit.", VERSION)).right_aligned())
            .border_style(Style::default()
                .fg(Color::Rgb(fg_color[0], fg_color[1], fg_color[2])))
        );
        textarea3.set_mask_char('\u{2022}');

        // display 
        let size = term.size()?;
        let input_area = Rect {
            x: (size.width - size.width / 2) / 2,
            y: (size.height - 3) / 2,
            width: size.width / 2,
            height: 3,
        };

        // init variables
        let mut textareas = [textarea1, textarea2, textarea3];
        let mut current_index = 0;
        let mut collected_data : Vec<String> = Vec::new();
        let log_bg_color = self.config.colors.log_background_color.clone();

        loop {
            term.draw(|f| {
                let background = Block::default()
                    .style(Style::default()
                        .bg(Color::Rgb(
                                log_bg_color[0],
                                log_bg_color[1],
                                log_bg_color[2],
                        )));
                f.render_widget(&textareas[current_index], input_area);
                f.render_widget(background, f.area());
            })?;

            // display error message (in any)
            let mut stdout = std::io::stdout();
            let error_message_login = match get_others() {
                Ok(Some(value)) => value.login_err,
                Ok(None) => {
                    "".to_string()
                }
                Err(e) => {
                    info!("ERROR: Failed to get login error: {}", e);
                    "".to_string()
                }};
            let _ = pop_message(&mut stdout, 6, error_message_login.as_str());

            match crossterm::event::read()? {
                event::Event::Key(KeyEvent { code: KeyCode::Enter, .. }) => {
                    if current_index < textareas.len() - 1 {
                        // will just take textarea 1 and 2, 3 will take after break loop

                        collected_data.push(textareas[current_index].lines().join("\n"));
                        current_index += 1;
                    } else {
                        break; 
                    }
                }

                event::Event::Key(KeyEvent { code: KeyCode::Esc, .. }) => {
                    let _ = update_login_err("");
                    clean_exit();
                }

                event::Event::Key(input) => {
                    if let Some(active_textarea) = textareas.get_mut(current_index) {
                        active_textarea.input(input); 
                    }
                }
                _ => {}
            }
        }

        // save the last input (from textearea3)
        collected_data.push(textareas[current_index].lines().join("\n"));

        // make disappear search_area (the input bar) after the break loop
        term.draw(|f| {
            let empty_block = Block::default();
            f.render_widget(empty_block, input_area); 

        })?;


        // Fetch data from api and insert them in database

        // send result
        if let Some(_active_textarea) = textareas.get(current_index) {
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
                    let _ = update_login_err(format!("ERROR: {}", error).as_str());
                }
                Err(_) => {
                    error!("[auth_process] The login thread stopped");
                    let _ = update_login_err("ERROR: The login stopped.");
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


use toutui::config::rgb_parts;
use toutui::{api, app, config, db, login_app, player, ui, utils};

use crate::db::crud::*;
use crate::db::database_struct::Database;
use crate::player::integrated::player_info::*;
use crate::ui::player_tui::*;
use crate::utils::clap::*;
use crate::utils::encrypt_token::decrypt_token;
use crate::utils::logs::*;
use crate::utils::pop_up_message::*;
use app::App;
use color_eyre::Result;
use crossterm::event::{self, KeyCode};
use log::info;
use login_app::AppLogin;
use ratatui::{
    style::{Color, Style},
    widgets::Block,
};
use std::io::stdout;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // clap
    clap().await;

    // A panic must give the terminal back. Without this hook the shell stayed
    // in the raw mode and on the alternate screen, therefore it showed no
    // character that the user typed and it had no cursor. The message of the
    // panic also stayed on the alternate screen, and the user did not read it.
    // See `06e548` and `40f48d` in `known_bugs.md`.
    //
    // The hook comes before every other line that can panic.
    toutui::utils::exit_app::install_panic_hook();

    // this function allow to write all the logs in a file
    setup_logs().expect("Failed to execute logger");

    // The program reads the secret key from `.env`. See `encrypt_token.rs`.
    let env_path = toutui::paths::env_file();
    toutui::utils::encrypt_token::load_env_file(&env_path);

    // Init database
    let mut _database = Database::new().await?;
    let mut _database_ready = false;

    // Wait for the database to be ready, waiting for the user to enter their credentials
    loop {
        _database = Database::new().await?;
        if _database.default_usr.is_empty() {
            let app_login = AppLogin::new().await?;
            let terminal = ratatui::init();
            let _app_result = app_login.run(terminal);
            // The wait stops a fast loop. If the screen of the login comes
            // back at once, for example because the terminal gives an error,
            // this loop would use a full processor without it.
            //
            // The wait is not for the database. The comment here said before
            // that the program could read the database before the login wrote
            // the user, and that the second attempt then worked. That race is
            // closed: `auth_input.rs` waits for the thread of the login, and
            // `auth_process` writes the user before it gives its answer. A
            // test on 2026-08-10 read the database with no wait after a real
            // login and found the user. See T-15 and
            // `tests/login_against_the_sandbox.rs`.
            tokio::time::sleep(Duration::from_secs(1)).await;
        } else {
            // If the database is ready, exit the loop
            print!("\x1B[2J\x1B[1;1H"); // clear all stdout (avoid to sill have the previous print when the app is launched)
            _database_ready = true;
            info!("Database ready");
            break;
        }
    }

    // Once the database is ready, initialize the app
    if _database_ready {
        // init current username
        let mut username: String = String::new();
        if let Some(var_username) = _database.default_usr.first() {
            username = var_username.clone();
        }
        // At the start no playback loop runs. Therefore a new playback must
        // not wait for a loop before it.
        let _ = update_has_played_before("1", username.as_str());

        // Make the HTTP client. The client holds all the addresses of the
        // server. If the address that has the most importance does not answer,
        // the client changes to the next address automatically.
        let server_address = _database.default_usr.get(1).cloned().unwrap_or_default();

        // The database holds the token in an encrypted form. The client needs
        // the plain token one time only.
        let encrypted_token = _database.default_usr.get(2).cloned().unwrap_or_default();
        let token = match decrypt_token(encrypted_token.as_str()) {
            Ok(token) => token,
            Err(e) => {
                println!("Error: {}", e);
                String::new()
            }
        };

        let config_file = config::load_config()?;
        let pool = config::pool_for_address(&config_file.servers, &server_address);
        info!("[main][api] The pool has {} address(es).", pool.len());

        let api = std::sync::Arc::new(api::client::ApiClient::new(
            std::sync::Arc::new(pool),
            token,
        )?);

        // The probe task gives an address the state `Up` again when the
        // address answers. Therefore the application returns to the local
        // address without a restart.
        api::client::probe::spawn_probe_task(std::sync::Arc::clone(&api));

        // The application plays a local copy when the server does not answer.
        // This task sends the positions when the server answers again, thus
        // the user does not start the application again. See T-25.
        //
        // A user can have an account on more than one server. The task sends
        // the positions of this server only.
        let server_key = config::server_key(&config_file.servers, &server_address);

        toutui::logic::offline::spawn_flush_task(
            std::sync::Arc::clone(&api),
            username.clone(),
            server_key,
        );

        let mut app = App::new(std::sync::Arc::clone(&api)).await?;

        let mut terminal = ratatui::init();

        // The picker asks the terminal for the protocol of the pictures and
        // for the size of the font. A terminal that does not answer gives
        // blocks of Unicode. See T-23.
        //
        // The question must come AFTER `ratatui::init`, and this sequence is
        // not free. `Picker::from_query_stdio` reads the answer on its own
        // thread. That thread makes the terminal raw, and it gives the old
        // condition back when it stops. A terminal that never answers stops
        // that thread two seconds later. With the question before the
        // application, the thread then gave the terminal the condition of the
        // shell back, and the application read no key at all. A measurement on
        // 2026-08-10 in a pseudo terminal showed the fault: `ICANON` and `ECHO`
        // stayed on for ever, and every key went to a line buffer. With the
        // question after `ratatui::init`, the thread reads a terminal that is
        // already raw, therefore it gives a raw terminal back.
        let picker = toutui::ui::cover::picker();
        info!(
            "[main][cover] the terminal uses {:?} with a font of {} by {} pixels",
            picker.protocol_type(),
            picker.font_size().width,
            picker.font_size().height,
        );

        // Running the app in a loop
        loop {
            // The engine gives the state. The panel is visible only when the
            // engine holds a media.
            let playback = app.player.state();
            let is_playing = playback.status != toutui::player::engine::PlaybackStatus::Stopped;
            let player_notice = playback.notice.clone();
            let player_info = player_info(app.username.as_str(), &playback);

            terminal.draw(|frame| {
                let (bg_r, bg_g, bg_b) = rgb_parts(&app.config.colors.background_color);
                let bg_color_player = app.config.colors.player_background_color.clone();
                // global background
                let background =
                    Block::default().style(Style::default().bg(Color::Rgb(bg_r, bg_g, bg_b)));

                frame.render_widget(background, frame.area());

                if is_playing {
                    let area = frame.area();
                    // render for the player (automatically refreshed)
                    render_player(
                        area,
                        frame.buffer_mut(),
                        player_info,
                        bg_color_player,
                        app.username.as_str(),
                        player_notice,
                    );
                }

                // render widget for general app :
                // Will be manually refresh by pressing `R`
                // If `app` variable is reinitialized below (`app = App::new().await?`), it will be taken into account and data will be refreshed
                // Otherwise, the current `app` variable will still be used.
                frame.render_widget(&mut app, frame.area());
            })?;

            // Checking if any key is pressed (waiting for events with a 200ms delay here)
            if crossterm::event::poll(Duration::from_millis(200))? {
                if let event::Event::Key(key) = crossterm::event::read()? {
                    app.handle_key(key);
                    // If the 'R' key is pressed, refresh the app
                    if let KeyCode::Char('R') = key.code {
                        // pop up message
                        let mut stdout = stdout();
                        let _ = clear_message(&mut stdout, 3); // clear a message, if any, before print the message bellow
                        let _ = pop_message(&mut stdout, 3, "Refreshing app...");
                        // Reinitialize app to refresh
                        app = App::new(std::sync::Arc::clone(&api)).await?;
                        // clear message above
                        let _ = clear_message(&mut stdout, 3);
                    }
                }
            }

            // Short pause between event checks
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    // Restore the terminal state before exiting the application
    ratatui::restore();
    Ok(())
}

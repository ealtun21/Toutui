use toutui::{api, app, config, db, logic, login_app, player, utils};

use crate::db::crud::*;
use crate::db::database_struct::Database;
use crate::utils::clap::*;
use crate::utils::encrypt_token::decrypt_token;
use crate::utils::logs::*;
use app::App;
use color_eyre::Result;
use crossterm::event::{self, KeyCode};
use log::{error, info};
use login_app::AppLogin;
use ratatui::{style::Style, widgets::Block};
use std::time::Duration;

/// Gives the terminal back, says why the program stops, and it stops.
///
/// **A user must not read a line of the source of this program.** The report
/// that left `main` gave `Error: The server reported a fault. Status 500.` with
/// `Location: src/app.rs:644:44`, and that text names no road at all. T-123
/// closed this for a token that the server refused, and every other fault of the
/// first request kept it. See T-172.
///
/// The whole report goes to the log, therefore no evidence goes away.
///
/// The function never comes back.
fn the_program_stops_with_words(
    report: color_eyre::eyre::Report,
    username: &str,
    server: &str,
) -> ! {
    log::error!("[app] the program stops: {:?}", report);

    // The application stands on the alternate screen. The words of the user
    // must stand on the screen of their shell.
    utils::the_terminal_of_the_program::the_program_gives_the_terminal_back();

    eprintln!(
        "{}",
        api::client::error::the_words_of_a_program_that_stops(&report, username, server)
    );

    std::process::exit(1);
}

#[tokio::main]
async fn main() -> Result<()> {
    // **The child that reads a PDF comes first.** The program spawns itself
    // with this flag, and that child must open no terminal, make no database,
    // and play nothing: it reads one book, it writes the pages, and it stops.
    // The peak of the memory of `lopdf` and every fault of that crate stay in
    // this process. See T-62.
    if let Some(code) = toutui::logic::reader::pdf_of_a_child::the_child_of_the_line_of_command() {
        std::process::exit(code);
    }

    // The program of the user runs, therefore a child may read a PDF. A test
    // and a program that takes this library never reach this line, and their
    // `current_exe` knows no flag of that module. See T-62.
    toutui::logic::reader::pdf_of_a_child::the_program_of_the_user_runs();

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

    // **A program that has no key keeps no token.** `install.sh` makes the key,
    // and a user who builds the program with `cargo`, with `nix`, or with a
    // package of their system gets no such file: the login of that user asked
    // the server, took the token, and wrote nothing. The next start showed the
    // login screen again. The program makes the key itself now. See T-133.
    match toutui::utils::encrypt_token::the_program_makes_a_key_if_it_has_none(
        &toutui::paths::config_dir(),
    ) {
        Ok(true) => info!(
            "[main] the machine had no secret key. The program made one in {}.",
            env_path.display()
        ),
        Ok(false) => {}
        // The login says the same thing to the user, because it is the login
        // that needs the key. This line holds the reason of the system.
        Err(error) => log::error!("[main] the program made no secret key: {}", error),
    }

    // **A token that the server refused opens the login screen**, and the
    // program then makes everything of the startup again with the new token.
    // Every turn of this loop is one account: the login screen, the client, the
    // tasks, and the views. See T-123.
    'the_session: loop {
        // **A database of accounts where no account holds the start gives the
        // login screen**, and the token of every one of those accounts stands on
        // the disk. No key of the program gave the mark back, in any view and
        // after every start: that is the shape of T-136, and the start is the
        // place of the answer. The first account takes the start then. See
        // T-155.
        match toutui::db::crud::an_account_takes_the_start_when_none_holds_it() {
            Ok(Some(name)) => info!(
                "[main] no account held the start of the program. The account {} takes it.",
                name
            ),
            Ok(None) => {}
            Err(error) => log::error!("[main] the account of the start: {}", error),
        }

        // Init database
        //
        // T-268. `Database::new` reads the accounts of the disk, and T-199 gave
        // that read a fault of its own. The `?` of these two lines gave that
        // report to the runtime of Rust: a measurement of 2026-08-16 with
        // `docs/harness/hold_the_lock.py` gave the user
        //
        // ```text
        // Error: The program did not read the accounts of its database: database is locked
        // Location:
        //     src/db/database_struct.rs:68:27
        // ```
        //
        // Those words name a line of the source of this program, which no user
        // must read (T-172), they hold no sentence of Toutui and no road back,
        // and `the_program_stops_with_words` did no work at all: the log of that
        // run held no line of a program that stops.
        //
        // **This read stands before the login screen**, therefore the program
        // holds no name of an account and no address of a server here, and the
        // words name neither of them (T-91, and the shape of T-267).
        let mut _database = match Database::new().await {
            Ok(database) => database,
            Err(report) => the_program_stops_with_words(report, "", ""),
        };
        let mut _database_ready = false;

        // Wait for the database to be ready, waiting for the user to enter their credentials
        loop {
            _database = match Database::new().await {
                Ok(database) => database,
                Err(report) => the_program_stops_with_words(report, "", ""),
            };
            // **The login screen comes for two reasons now.** The database
            // holds no account, or the user asked for a new account with the
            // key `a` of the view of the accounts: that key starts the program
            // again, and the variable of the environment carries the request
            // through `exec`. The request lives for one login only, therefore
            // the program forgets it here. See T-124.
            let the_user_adds_an_account = logic::the_accounts::the_program_adds_an_account();
            if _database.default_usr.is_empty() || the_user_adds_an_account {
                std::env::remove_var(logic::the_accounts::THE_PROGRAM_ADDS_AN_ACCOUNT);
                // T-267. `AppLogin::new` reads the configuration file, and it
                // reads nothing else. The `?` of this line gave the report of
                // that file to the runtime: the screen then held the words of
                // the crate, a line of the source of this program, which no
                // user must read (T-172), and no sentence of Toutui at all.
                // The log held no word of that fault either. The words of
                // T-265 stand on this road too.
                //
                // **The login screen stands before every account.** The name
                // of an account and the address of a server therefore hold no
                // character here, and the words of a fault of the file of the
                // user name neither of them (T-91).
                let app_login = match AppLogin::new().await {
                    Ok(app_login) => app_login,
                    Err(report) => the_program_stops_with_words(report, "", ""),
                };
                // T-273. `ratatui::init()` panics for a machine that gives no
                // terminal, and the hook of the panic of T-197 then says that a
                // part of the program had an internal fault: a reason that the
                // program does not have (T-91).
                let terminal = utils::the_terminal_of_the_program::the_terminal_of_the_program();
                // T-272. The loop of the login screen stands inside
                // `crossterm::event::read`, and that call reads no byte and it
                // counts no event for a terminal that gives the end of its
                // input: a login screen whose terminal went away holds a whole
                // processor for ever, and the `?` of that call reaches nothing.
                // The watch of T-271 cannot hold this road, because it needs the
                // client of the server, the name of an account, and the name of
                // a server, and the login screen stands before every one of
                // them.
                let the_watch_of_the_login_screen =
                    utils::the_terminal_that_went_away::spawn_the_watch_of_the_terminal_of_the_login_screen();
                // T-275. The old line of this place dropped the answer of the
                // login screen. A standard output that failed gave that screen a
                // fault at each frame, and the loop then waited one second and
                // it made a terminal again: the words of the user came of T-273,
                // and they said that the program found no terminal while the
                // user stood in one (T-91).
                let the_end_of_the_login = app_login.run(terminal);
                // The watch of T-271 holds the road after the login, and that
                // one closes the session of the server. Two watches of one
                // terminal give the road of this one to a program that holds a
                // session, therefore this task stops here.
                the_watch_of_the_login_screen.abort();
                // The login screen holds the loop of the keys of the user, and
                // it comes back for a login that succeeded, for a login that
                // failed, and for the key Esc. Every other answer of it is a
                // screen that did not reach the terminal (T-275).
                if let Err(fault) = the_end_of_the_login {
                    utils::the_terminal_of_the_program::
                        the_program_stops_for_a_screen_that_did_not_reach_the_terminal(&fault);
                }
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
                // The clear takes the prints of the login away, and it goes to
                // the terminal at once: a clear that waits in the buffer of
                // `stdout` comes out at the exit of the program, and it then
                // takes the words of a program that stops with it. See T-265.
                if let Err(error) =
                    utils::startup::clear_the_screen_of_the_shell(&mut std::io::stdout())
                {
                    log::warn!("[main] the screen of the shell takes no clear: {}", error);
                }
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
            //
            // **A caller that reads no answer of its write says nothing at all**
            // (T-207). A disk that takes no write leaves the row of an older
            // program, and the first playback of this program then waits the
            // whole limit of time of `wait_prev_session_finished` for a loop that
            // no program holds. The write holds no key of the user, therefore its
            // fault takes a line of the log (T-177).
            if let Err(error) = update_has_played_before("1", username.as_str()) {
                log::error!(
                    "[main] the disk did not take the mark of the start of {}: {}. The first \
                     playback of this program can wait for a loop that no program holds.",
                    username,
                    error
                );
            }

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

            // T-265. A configuration file that the program cannot read at all
            // gives no value of the user, therefore the program stops. The
            // report of `?` gave the words of the crate `config` and a line of
            // the source of this program, and the clear of the start then took
            // those words off the screen: the user read nothing at all. These
            // words name the file and the road back.
            let config_file = match config::load_config() {
                Ok(file) => file,
                Err(report) => {
                    the_program_stops_with_words(report, username.as_str(), server_address.as_str())
                }
            };

            // T-264. A value of the file that the program does not use took a
            // line of the log alone, and the user who wrote that value saw
            // nothing at all. The message stands here, before the first frame:
            // the box of the message belongs to no `App`, therefore it waits
            // for the frame that draws it.
            config::say_the_values_that_the_program_does_not_use(&config_file);

            // T-298. A log out of the account that starts the program gives the
            // start to another account, and the program starts again with
            // `exec`: the box of the message goes away with the process, and
            // that program holds an account, therefore it draws no login screen
            // and the disk of T-270 reaches nobody. The environment of the new
            // process carries those words, and they stand here — above the
            // values of the file, because the user pressed a key for them.
            logic::message::say_the_words_of_the_start();

            // The cache of the ebooks runs inside a task, and that task holds no
            // `App`. Therefore the limit of the configuration file goes to its slot
            // here, one time. See T-72.
            logic::reader::cache::keep_the_limit_of_the_configuration(
                config_file.reader.ebook_cache_mb,
            );

            // The user must be able to see which of the three sources gave the
            // limit: the variable of the environment, `config.toml`, or the program.
            info!(
                "[main][cache] the cache of the ebooks holds {} byte(s) at the most.",
                logic::reader::cache::the_limit()
            );

            let pool = config::pool_for_address(&config_file.servers, &server_address);
            info!("[main][api] The pool has {} address(es).", pool.len());

            // The task of the live messages needs the plain token, and the client
            // keeps its own token for itself. See T-47.
            let token_of_the_live_task = token.clone();

            let api = std::sync::Arc::new(api::client::ApiClient::new(
                std::sync::Arc::new(pool),
                token,
            )?);

            // **Every task of this account holds its handle here.** A login that
            // comes again makes new tasks with the new token, and a task of the
            // token before it must stop: two live tasks write one state of the
            // program, therefore the header would say the fault of the old token.
            // See T-123.
            let mut the_tasks_of_the_account: Vec<tokio::task::JoinHandle<()>> = Vec::new();

            // The probe task gives an address the state `Up` again when the
            // address answers. Therefore the application returns to the local
            // address without a restart.
            the_tasks_of_the_account.push(api::client::probe::spawn_probe_task(
                std::sync::Arc::clone(&api),
            ));

            // The live messages of the server. Audiobookshelf sends every change
            // of every client over socket.io, and the transport `polling` of that
            // protocol is plain HTTP. Therefore this task needs no new dependency.
            // See T-47.
            //
            // The task takes the token here, because `ApiClient` keeps its token
            // for itself.
            the_tasks_of_the_account.push(api::live::spawn_the_live_task(
                api.pool(),
                token_of_the_live_task,
            ));

            // The application plays a local copy when the server does not answer.
            // This task sends the positions when the server answers again, thus
            // the user does not start the application again. See T-25.
            //
            // A user can have an account on more than one server. The task sends
            // the positions of this server only.
            let server_key = config::server_key(&config_file.servers, &server_address);

            // The queue of the media stands on the disk. The program reads the queue
            // of this account and of this server before the first frame. See T-56.
            toutui::logic::queue::read_the_queue_of_the_account(&username, &server_key);

            // **The loop of the program reads the accounts of a box** (T-204),
            // and this task fills that box. The rule of T-159 stays: the disk is
            // the truth, and a program whose account stands in no row starts
            // again.
            the_tasks_of_the_account.push(
                toutui::logic::the_accounts::the_box_of_the_accounts::spawn_the_task_of_the_accounts(),
            );

            the_tasks_of_the_account.push(toutui::logic::offline::spawn_flush_task(
                std::sync::Arc::clone(&api),
                username.clone(),
                server_key.clone(),
            ));

            // **The terminal of this program can go away, and no signal comes
            // with it** (T-271). The loop of the screen cannot see it: a
            // measurement of 2026-08-16 with
            // `docs/harness/the_terminal_of_the_program_goes_away.py` gave
            // 439442 reads of no byte in four seconds inside
            // `crossterm::event::poll`, a call that never comes back on a
            // terminal that gives the end of its input. The program then holds
            // a whole processor for ever, and its row of `listening_session`
            // keeps a heartbeat that no second program of the account can take
            // (T-140).
            //
            // This task holds the answer, because it stands on a thread of the
            // runtime and not in the loop of the screen. It stops the program
            // on the road of the key `Q`: the session of the server closes, and
            // the place of the user stays.
            the_tasks_of_the_account.push(
                toutui::utils::the_terminal_that_went_away::spawn_the_watch_of_the_terminal(
                    std::sync::Arc::clone(&api),
                    username.clone(),
                    server_key,
                ),
            );

            // The terminal comes first, and a screen comes before the requests.
            //
            // `App::new` asks the server many times: the libraries, the list
            // Continue Listening, the position of each book of that list, the
            // series, every item, and the lists. The old code drew nothing until
            // all of that finished. A measurement on 2026-08-10 gave a server that
            // accepts a connection and answers nothing: the screen stayed black
            // for 15 seconds, the whole timeout of one request. A slow server with
            // many books gives a black screen for much longer, and the user cannot
            // tell a slow server from a program that stopped. See T-40.
            // The words of a machine that gives no terminal stand here too. See
            // T-273.
            let mut terminal =
                toutui::utils::the_terminal_of_the_program::the_terminal_of_the_program();

            let started = std::time::Instant::now();
            let server_name = _database
                .default_usr
                .get(1)
                .cloned()
                .unwrap_or_else(|| String::from("the server"));

            let made = {
                let making = App::new(std::sync::Arc::clone(&api));
                tokio::pin!(making);
                let mut tick: usize = 0;

                loop {
                    tokio::select! {
                        biased;
                        result = &mut making => break result,
                        _ = tokio::time::sleep(Duration::from_millis(120)) => {
                            tick += 1;
                            terminal.draw(|frame| {
                                toutui::ui::loading::render(
                                    frame,
                                    &server_name,
                                    tick,
                                    started.elapsed().as_secs(),
                                )
                            })?;

                            // The user can stop the program while it waits.
                            if crossterm::event::poll(Duration::from_millis(0))? {
                                if let event::Event::Key(key) = crossterm::event::read()? {
                                    if key.kind == event::KeyEventKind::Press
                                        && matches!(key.code, KeyCode::Char('Q') | KeyCode::Esc)
                                    {
                                        utils::the_terminal_of_the_program::the_program_gives_the_terminal_back();
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            };

            // **The first request of the program tells a token that is not valid.**
            // The user goes to the login screen, and not to a line of the source of
            // this program. See T-123.
            let mut app = match made {
                Ok(app) => app,
                Err(report) if api::client::error::the_token_is_not_valid(&report) => {
                    for task in &the_tasks_of_the_account {
                        task.abort();
                    }

                    // T-269. The removal of the row of the account is the one
                    // road of this fault: a row that stays sends the login
                    // screen to the same answer of the server for ever. The
                    // `map_err(Report::msg)?` of this line gave the report to
                    // the runtime of Rust, and a measurement of 2026-08-16 with
                    // a trigger of `BEFORE DELETE ON users` gave the user
                    //
                    // ```text
                    // Error: The account toutuitest stays in the database: the disk takes no removal of the account
                    // Location:
                    //     …/library/core/src/ops/function.rs:250:5
                    // ```
                    //
                    // That text names a line of a file of the standard library
                    // of Rust, which no user must read (T-172), it holds no
                    // sentence of Toutui and no road back, and the log of that
                    // run held no line of a program that stops.
                    if let Err(fault) = logic::auth::auth_input::the_program_needs_a_new_token(
                        &username,
                        &server_address,
                    ) {
                        the_program_stops_with_words(
                            color_eyre::eyre::Report::new(fault),
                            username.as_str(),
                            server_address.as_str(),
                        )
                    }

                    continue 'the_session;
                }
                // **Every other fault of the first request kept the road that
                // T-123 closed.** A server that answers 500 to
                // `GET /api/libraries` gave the user `Error: The server reported
                // a fault. Status 500.` with `Location: src/app.rs:644:44`, and
                // no screen of the program came. See T-172.
                Err(report) => {
                    for task in &the_tasks_of_the_account {
                        task.abort();
                    }

                    the_program_stops_with_words(report, &username, &server_address);
                }
            };

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
                // The timer for sleep works here, because the loop of the
                // program runs at each frame and it holds the handle of the
                // engine. See T-24.
                app.tick_the_timer_for_sleep();

                // **The queue changes while its view stands open, and no key of
                // the user does it**: a media that comes to its end takes the
                // media of the front of the queue away, and a second program of
                // the account takes a media out with the key `X`. The cursor of
                // the user goes with the media of its line, and it goes to
                // nobody when that media leaves the queue. See T-161.
                app.the_line_of_the_queue_holds_its_media();

                // **The media that plays changes while the view of its chapters
                // stands open, and no key of the user does it**: the media comes
                // to its end, and the queue starts the media of its front. The
                // list is then the list of another media, therefore the line
                // goes to nobody and the key `l` seeks in no media that the user
                // did not choose. See T-162.
                app.the_view_of_the_chapters_holds_its_media();

                // **The queue of the downloads of the server changes while its
                // view stands open, and no key of any user does it**: the
                // server takes an episode out when it downloaded it, and a
                // second program of the library empties that queue. The cursor
                // of the user goes with the episode of its line, and it goes to
                // nobody when that episode leaves the queue: the key `X` then
                // names no podcast that the user did not choose. See T-166.
                app.the_line_of_the_downloads_holds_its_episode();

                terminal.draw(|frame| {
                    // global background
                    let background =
                        Block::default().style(Style::default().bg(app.config.colors.background()));

                    frame.render_widget(background, frame.area());

                    // **The band of the player belongs to the render of the
                    // frame** (T-322): this loop drew it at 9 rows above the end
                    // of the screen, and a view of a footer of three rows then
                    // held two numbers of one row. `App::render` draws it under
                    // the work of the view now, with the layout of that view.

                    // render widget for general app :
                    // Will be manually refresh by pressing `R`
                    // If `app` variable is reinitialized below (`app = App::new().await?`), it will be taken into account and data will be refreshed
                    // Otherwise, the current `app` variable will still be used.
                    frame.render_widget(&mut app, frame.area());
                })?;

                // The loop waits for a key, and it then takes every key that
                // waits.
                //
                // The old code took one key for each turn of the loop, and the
                // loop then drew the screen and waited 50 milliseconds. A key that
                // repeats gives about 30 keys each second, therefore the keys made
                // a queue. The list moved slowly, and it went on moving after the
                // user let the key go. See T-39.
                //
                // The limit stops a very long queue from holding the screen.
                const KEYS_FOR_ONE_FRAME: usize = 64;
                let mut taken = 0;

                if crossterm::event::poll(Duration::from_millis(200))? {
                    while taken < KEYS_FOR_ONE_FRAME
                        && crossterm::event::poll(Duration::from_millis(0))?
                    {
                        let event = crossterm::event::read()?;
                        taken += 1;

                        // **A report of the mouse is not a key** (T-316). The
                        // terminal sends it when the program asks for the
                        // capture, and the loop before this one read every one
                        // of them and it did nothing at all: a click of the
                        // user moved no cursor and it gave no panel the focus.
                        if let event::Event::Mouse(the_report) = event {
                            app.handle_the_mouse(the_report);
                            continue;
                        }

                        if let event::Event::Key(key) = event {
                            // A terminal that reports the release of a key sends
                            // two events for one press. The application must act
                            // one time.
                            if key.kind != event::KeyEventKind::Press {
                                continue;
                            }

                            app.handle_key(key);

                            // **The key `Ctrl+o` starts the reports of the
                            // mouse and it stops them** (T-316). `App` holds no
                            // terminal, therefore the request stands here, with
                            // the standard output of this loop.
                            if app.the_capture_of_the_mouse_must_change {
                                app.the_capture_of_the_mouse_must_change = false;
                                utils::the_terminal_of_the_program::the_program_reads_the_mouse(
                                    app.the_mouse_stands,
                                );
                            }

                            // A box that took a text wrote on the cells of the
                            // view below it. ratatui writes the cells that changed
                            // only, therefore those cells need a clear: the next
                            // draw then writes every one of them. See T-89.
                            if app.the_screen_must_be_drawn_again {
                                app.the_screen_must_be_drawn_again = false;
                                terminal.clear()?;
                            }

                            // **The account of this program can go away while
                            // the program runs.** A second program of one
                            // account logs out with the key `l` of the view of
                            // the accounts, and the row of `users` goes away
                            // then (T-155). The program held the token of that
                            // account in its client and it went on, and every
                            // key that refreshes the screen gave a program of
                            // no name at all. **The disk is the truth, and the
                            // program reads it at the moment of the use**: a
                            // key is that moment, and the program that starts
                            // again takes the account of the disk. See T-159.
                            if app.the_program_starts_again.is_none() {
                                // **A read that failed is not a row that went
                                // away.** The old line was
                                // `select_every_usr().unwrap_or_default()`,
                                // therefore a database that a second program of
                                // this account held gave a list of no account:
                                // the program said "the account … stands in no
                                // row of the disk", it started itself again, and
                                // it wrote the sentence of an account that is
                                // gone in the row of the message of the login.
                                // The row of that account stood on the disk all
                                // the time.
                                //
                                // A fault of this read takes a line of the log
                                // and no word for the user: the account of this
                                // program stays, and the next key reads the disk
                                // again. See T-199 and T-177.
                                // **A read of the disk holds the thread of the
                                // screen** (T-204): this line was
                                // `select_every_usr()`, and five presses of the
                                // key `j` moved no cursor for the 30 seconds of
                                // a lock of a second program of the account. The
                                // task of the box reads the disk each second on
                                // a thread of its own, and no read of the disk
                                // stands between a key of the user and the next
                                // frame.
                                //
                                // A box with nothing in it is a read that did
                                // not come or that failed, and the account of
                                // this program then stays (T-199).
                                if let Some(of_the_disk) =
                                    logic::the_accounts::the_box_of_the_accounts::the_accounts_of_the_disk()
                                {
                                    if matches!(
                                        logic::the_accounts::the_account_of_the_line(
                                            &of_the_disk,
                                            &app.username
                                        ),
                                        logic::the_accounts::TheAccountOfTheLine::ItIsGone
                                    ) {
                                        app.the_account_of_this_program_is_gone();
                                    }
                                }
                            }

                            // A key of the view of the accounts starts the
                            // program again, and **the position of a playback
                            // that it stops must reach the server first**.
                            //
                            // `exec` takes every task of this process away,
                            // therefore a task that sends the position never
                            // finishes: the key `a` of a book at the minute 13
                            // left the server at the minute 13:23, and the
                            // program held 13:31. The loop is the place of that
                            // work, because a key handler cannot wait for the
                            // server. See T-139.
                            if let Some(request) = app.the_program_starts_again.take() {
                                logic::message::say(
                                    "The program sends the place of the playback, and it starts \
                                     again…",
                                );
                                terminal
                                    .draw(|frame| frame.render_widget(&mut app, frame.area()))?;

                                // The engine stops first: the loop of the
                                // playback then writes no position after the
                                // one that goes to the server.
                                app.player.send(player::engine::PlayerCommand::Stop);

                                logic::sync_session::sync_session_from_database::sync_session_from_database(
                                    &api,
                                    app.username.clone(),
                                    app.server_key.clone(),
                                    false,
                                    "the accounts",
                                )
                                .await;

                                let variables: Vec<(&str, &str)> = request
                                    .variables
                                    .iter()
                                    .map(|(name, value)| (name.as_str(), value.as_str()))
                                    .collect();

                                let of_the_system =
                                    utils::exit_app::start_the_program_again_with(&variables);

                                // The program stays. A system that has no `exec`
                                // says why, and the user reads the way that
                                // works there.
                                log::error!(
                                    "[the accounts] the program does not start again: {}",
                                    of_the_system
                                );

                                logic::message::say(request.message.as_str());
                            }

                            // The key `R` refreshes the application. A new
                            // sequence or a new filter of the library needs the
                            // same work, because every list of the library comes
                            // from one request. See T-24.
                            //
                            // A change of the sequence keeps the user in the
                            // Library view. A new application starts at the Home
                            // view, therefore this line puts it back.
                            let from_the_sequence = app.must_refresh;

                            if matches!(key.code, KeyCode::Char('R')) || from_the_sequence {
                                // The values of the filter come from the library.
                                // A new request must ask for them again.
                                logic::sort_filter::from_the_server::forget();
                                logic::authors::forget();

                                // The store of the covers lives outside `App`,
                                // therefore a new application keeps every cover
                                // that the program read before. **A request of a
                                // cover that came back with a fault stayed there
                                // for the whole life of the program**, and no key
                                // of the user could correct it: this key asks the
                                // server for every cover again. See T-185.
                                toutui::ui::cover::forget();

                                // This request asks for every list again, therefore
                                // no list of the screen is old after it. See T-47.
                                logic::live::the_lists_are_new_again();

                                // The refresh asks the server many times, and the
                                // loop draws no frame while it waits. Therefore the
                                // program says the message and it draws **one** frame
                                // before the work. The message stands inside that
                                // frame, therefore no byte of it stays on the screen.
                                // See T-59 and T-42.
                                logic::message::say("The program asks the server again…");
                                terminal
                                    .draw(|frame| frame.render_widget(&mut app, frame.area()))?;

                                // Reinitialize app to refresh
                                //
                                // A server can take the token away while the program
                                // runs. The refresh then meets it, and the login
                                // screen comes: the program must not stop with a
                                // fault. See T-123.
                                // **The engine of the playback stays.** A new
                                // application started a new engine of the sound,
                                // and the old engine kept the playback: the row
                                // of the player went away while the media played,
                                // and no key of the user reached that playback.
                                // See T-131.
                                let the_engine =
                                    Some((app.player.clone(), app.audio_fault.clone()));

                                // **The timer for sleep stays too.** The engine
                                // of T-131 kept the playback, and the timer of
                                // the user stayed with the application that went
                                // away: the media that they set to stop played
                                // on, and the row of the player held no timer.
                                // See T-135.
                                let the_state_of_the_user = app.the_state_that_a_refresh_keeps();

                                let the_new_application = App::new_with_the_engine(
                                    std::sync::Arc::clone(&api),
                                    the_engine,
                                )
                                .await;

                                // **A refresh is not a start** (T-205). T-199
                                // gave the read of the accounts a fault of its
                                // own, and the start of the program stops with
                                // it: a program with no account can do no work.
                                // **A refresh holds the account, the token, every
                                // list, and the playback of the user already**,
                                // and a second Toutui of this account writes the
                                // database of it (T-140): a measurement of
                                // 2026-08-14 with `docs/harness/hold_the_lock.py`
                                // took the whole program away at the key `R` and
                                // at the key of the next library, and the words of
                                // that stop said "Toutui changed nothing".
                                //
                                // The application of the user stays here, and the
                                // row of the message says why the screen did not
                                // change.
                                let the_database_said_nothing = matches!(
                                    &the_new_application,
                                    Err(report)
                                        if api::client::error::the_accounts_did_not_come(report)
                                );

                                // **A refresh is not a start**, and the
                                // configuration file holds that rule too (T-266).
                                // The key `R` reads `config.toml` again (T-142),
                                // therefore the user who changes one colour of
                                // that file and who leaves one bracket out meets
                                // this road. A measurement of 2026-08-15 of the
                                // real program v0.8.94: that key took the whole
                                // program away with the status 1, and the
                                // playback, the queue, and every list of the user
                                // went with it, for one character of a file that
                                // the program read one time already. The values of
                                // the file that the application holds stay good,
                                // therefore that application stays.
                                let the_file_did_not_come = matches!(
                                    &the_new_application,
                                    Err(report)
                                        if api::client::error
                                            ::the_configuration_file_did_not_come(report)
                                );

                                if the_database_said_nothing || the_file_did_not_come {
                                    if let Err(report) = &the_new_application {
                                        if the_file_did_not_come {
                                            error!(
                                                "[the refresh] the program cannot read its \
                                                 configuration file: {}. The application of the \
                                                 user stays.",
                                                report
                                            );
                                        } else {
                                            error!(
                                                "[the refresh] the program did not read the \
                                                 accounts of its database: {}. The application of \
                                                 the user stays.",
                                                report
                                            );
                                        }
                                    }

                                    // **The mark of the refresh goes away** (T-205).
                                    // The key of the sequence and the key of the
                                    // next library write it, and the loop reads it
                                    // at each key: a mark that stays gives a refresh
                                    // of every key after this one.
                                    app.must_refresh = false;

                                    logic::message::forget();

                                    // **The screen did not change, therefore
                                    // the words of the key are a lie** (T-308).
                                    // The key of the next library and the key
                                    // `l` of the settings each say that the
                                    // program shows another library now, and
                                    // this road keeps the application of the
                                    // user: the sentence below is the one
                                    // answer of that key.
                                    logic::message::forget_the_words_of_the_refresh();

                                    logic::message::say(if the_file_did_not_come {
                                        toutui::ui::keys::
                                            THE_REFRESH_DID_NOT_READ_THE_CONFIGURATION_FILE
                                    } else {
                                        toutui::ui::keys::THE_REFRESH_DID_NOT_READ_THE_DATABASE
                                    });
                                } else {
                                    app = match the_new_application {
                                        Ok(new) => new,
                                        Err(report)
                                            if api::client::error::the_token_is_not_valid(
                                                &report,
                                            ) =>
                                        {
                                            for task in &the_tasks_of_the_account {
                                                task.abort();
                                            }

                                            // The key `R` holds the same road as
                                            // the start. See T-269.
                                            if let Err(fault) =
                                                logic::auth::auth_input::the_program_needs_a_new_token(
                                                    &username,
                                                    &server_address,
                                                )
                                            {
                                                the_program_stops_with_words(
                                                    color_eyre::eyre::Report::new(fault),
                                                    username.as_str(),
                                                    server_address.as_str(),
                                                )
                                            }

                                            continue 'the_session;
                                        }
                                        // The key `R` makes the same requests as the
                                        // start, therefore it holds the same road.
                                        // See T-172.
                                        Err(report) => {
                                            for task in &the_tasks_of_the_account {
                                                task.abort();
                                            }

                                            the_program_stops_with_words(
                                                report,
                                                &username,
                                                &server_address,
                                            );
                                        }
                                    };

                                    app.keep_the_state_of_the_application_before(
                                        the_state_of_the_user,
                                    );

                                    if from_the_sequence {
                                        app.view_state = app::AppView::Library;
                                    }

                                    logic::message::forget();

                                    // T-264. The key `R` reads the
                                    // configuration file again, therefore the
                                    // user who corrected that file reads the
                                    // answer of that key here. The line above
                                    // removes the message of the refresh,
                                    // therefore this sentence comes after it:
                                    // a `say` inside `App::new_with_the_engine`
                                    // said the words, and the `forget` of the
                                    // refresh then took them away before the
                                    // first frame.
                                    config::say_the_values_that_the_program_does_not_use(
                                        &app.config,
                                    );

                                    // **A key that refreshes the program says
                                    // its words here** (T-308). The `forget`
                                    // above takes away every message that the
                                    // key wrote, therefore the two keys of a
                                    // new library keep their sentence in a slot
                                    // that stands outside it. The sentence
                                    // comes last, because it is the answer of
                                    // the key that the user pressed.
                                    logic::message::the_words_of_the_refresh_come();
                                }

                                // A refresh makes a new application, therefore every
                                // view can change. A clear makes the next draw write
                                // every cell. See T-42.
                                terminal.clear()?;
                            }

                            if app.should_exit {
                                break;
                            }
                        }
                    }
                }

                // A place that the server took is a place that the reader
                // sends no second time. The task of the send writes it, and
                // this call gives it to the reader. See T-291.
                app.take_the_place_that_the_server_took();

                // The reader sends the place of the user to the server while
                // they read. The rule of the time lives in the reader: it sends
                // when the place changed and 30 seconds went by. See T-10.
                app.send_the_place_of_the_reader_if_it_is_time();

                // The reader holds no table of the disk, therefore the place of
                // the user reaches the two roads of a program that stops — the
                // key `Q` and the terminal that went away — through the box of
                // the process alone. See T-292.
                app.say_the_place_of_the_reader_that_waits();

                // A second window of this account removes the books of the
                // cache of the ebooks with no key of this window, and the
                // removal of that window cannot see the book that this window
                // reads. The reader writes the time of its file every 15
                // seconds, and the removal keeps every book of a recent time.
                // See T-153.
                app.say_that_this_program_reads_its_book();

                // Short pause between event checks. A turn that took keys draws at
                // once, therefore the screen follows the user.
                if taken == 0 {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                }
            }
        }

        // The loop of the views above ends when the user stops the program, and
        // `continue 'the_session` is the one way back to the login screen.
        break;
    }

    // Restore the terminal state before exiting the application
    utils::the_terminal_of_the_program::the_program_gives_the_terminal_back();
    Ok(())
}

//! The login screen of a configuration file that the program cannot read says
//! why, with the words of the program. See T-267.
//!
//! **The parts of this test stay in one function.** See T-144 and T-157.
//!
//! The measurement of 2026-08-15 of the real program v0.8.95 inside tmux, on a
//! screen of 160 columns and 45 rows, with a `XDG_CONFIG_HOME` of a database
//! that holds no account (the trap 135) and a `config.toml` with one bracket
//! out (`background_color = [40, 40, 40`):
//!
//! | The program | The screen of the user |
//! |---|---|
//! | v0.8.95 | `Error: The program cannot read the configuration file …`, and `Location: src/config.rs:212:13` |
//! | v0.8.95, the log of that run | no line of that fault at all |
//! | the correction | `Toutui stops: it cannot read its configuration file.`, the reason, and `Correct that file, or give it a different name: Toutui then makes a new file.` |
//! | the correction, with a file of no fault | the login screen, with the field `Server address` |
//!
//! T-265 gave those words to the road of `src/main.rs` that reads the file
//! after the login, and T-266 gave the road of the key `R` its own answer.
//! **The login screen is the third reader of that file**, and it stood before
//! both of them: `AppLogin::new` of `src/login_app.rs` reads the file with `?`,
//! and that `?` of `src/main.rs` gave the report to the runtime. The words of
//! the runtime name a line of the source of this program, which no user must
//! read (T-172), and they hold no sentence of Toutui and no road back.
//!
//! **The login screen stands before every account**, therefore the name of an
//! account and the address of a server hold no character on this road, and the
//! words of a fault of the file of the user name neither of them (T-91).

#[test]
fn the_login_screen_of_a_file_that_the_program_cannot_read_says_why() {
    // 1. The whole chain: a file of a shape that the crate refuses gives the
    //    report, and the words of a program that stops read it.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, b"[colors]\nbackground_color = [40, 40, 40\n").unwrap();

    let report = toutui::config::load_config_from(&path)
        .expect_err("a file of a shape that the crate refuses gives no value of the user");

    // The login screen holds no account and no server, therefore the two names
    // of this call hold no character.
    let the_words = toutui::api::client::error::the_words_of_a_program_that_stops(&report, "", "");

    for the_word in [
        "Toutui stops: it cannot read its configuration file.",
        "Correct that file, or give it a different name",
        "Toutui changed nothing.",
    ] {
        assert!(
            the_words.contains(the_word),
            "the words of the login screen say \"{}\": {}",
            the_word,
            the_words
        );
    }

    assert!(
        the_words.contains(path.to_str().unwrap()),
        "the words name the file of the user: {}",
        the_words
    );

    // **A view never says a reason that the program does not have** (T-91). A
    // fault of the file of the user is no fault of the server, and the login
    // screen holds no account at all: a sentence of an account with no name is
    // a sentence of nobody.
    for the_word in ["server", "account", "administrator", "database"] {
        assert!(
            !the_words.to_lowercase().contains(the_word),
            "a fault of the file of the user says no reason that the program does not have \
             (\"{}\"): {}",
            the_word,
            the_words
        );
    }

    // **No line of the source of this program belongs to the user** (T-172).
    assert!(
        !the_words.contains("src/config.rs") && !the_words.contains("Location:"),
        "the words hold no line of the source of this program: {}",
        the_words
    );

    // 2. The road of the login screen of `src/main.rs` reads that report. **A
    //    test of the source is the road of a decision of the loop of the
    //    program** (T-135, T-143, and T-204), and the block of this test ends
    //    at the line after the login screen, therefore no comment of a
    //    correction takes an assertion of it away (the trap 209).
    let of_the_loop = std::fs::read_to_string("src/main.rs").unwrap();

    assert!(
        !of_the_loop.contains("AppLogin::new().await?"),
        "the report of the login screen goes to no runtime: the words of the runtime name a \
         line of the source of this program"
    );

    let the_login_screen = of_the_loop
        .split("let app_login")
        .nth(1)
        .expect("the loop of the start makes the application of the login screen")
        .split("let terminal = ratatui::init();")
        .next()
        .expect("the terminal of the login screen comes after that application");

    assert!(
        the_login_screen.contains("AppLogin::new().await"),
        "the login screen makes its application there: {}",
        the_login_screen
    );
    assert!(
        the_login_screen.contains("the_program_stops_with_words"),
        "a login screen that got no configuration file stops the program with the words of \
         Toutui: {}",
        the_login_screen
    );
}

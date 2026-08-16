//! The colours of the start are the colours of the terminal of the user. See
//! T-317.
//!
//! **The program painted a dark grey over the terminal of every user.**
//! `Colors::default` of `src/config.rs` gave each of the eleven settings of the
//! file a grey of RGB of its own — `background_color` was `[40, 40, 40]`,
//! `list_background_color` was `[50, 50, 50]`, `header_background_color` was
//! `[60, 60, 60]`, and `player_background_color` was `[80, 80, 80]` — and
//! `config.example.toml`, which the program writes for a user who holds no file
//! (T-122), held the same values again. A user of a theme of a light colour
//! therefore got the theme of this program and not the theme of their terminal.
//!
//! **The measurement of the fault**, of the real program v0.8.144 inside tmux
//! at 100 columns and 30 rows, with `tmux capture-pane -e` and a count of the
//! escapes of the first frame:
//!
//! ```text
//!      12 [48;2;40;40;40m
//!      10 [48;2;50;50;50m
//!       5 [48;2;60;60;60m
//!       2 [38;2;180;180;180m
//!       1 [48;2;80;80;80m
//!       1 [39m
//! ```
//!
//! **28 escapes of a background of RGB, and no escape of the background of the
//! terminal at all.**
//!
//! **The measurement of the corrected program**, of the same terminal and the
//! same keys, with the block `colors` of the file off:
//!
//! ```text
//!       1 [48;5;6m
//!       1 [38;5;0m
//! ```
//!
//! **No escape of RGB at all.** The two that stay are the row of the cursor: the
//! accent of the program (the cyan of the terminal, `48;5;6`) and the letters on
//! it (the black of the terminal, `38;5;0`). Every other cell of the screen
//! holds the background and the foreground of the terminal of the user.
//!
//! **The eleven settings keep their work**: a user who writes
//! `background_color = [40, 40, 40]` gets that colour, and the tests below hold
//! that rule too.
//!
//! These tests need no network, no sandbox, and no terminal.

use ratatui::style::Color;
use toutui::config::Colors;

/// Says that a colour is one that the theme of the terminal of the user can
/// give: the default of the terminal, or one of the 16 names of ANSI.
fn of_the_terminal(colour: Color) -> bool {
    !matches!(colour, Color::Rgb(_, _, _) | Color::Indexed(_))
}

/// Every colour of the start comes of the terminal of the user. See T-317.
///
/// **The parts of this test stay in one function.**
#[test]
fn every_colour_of_the_start_comes_of_the_terminal() {
    let of_the_start = Colors::default();

    let all = [
        ("background", of_the_start.background()),
        ("log_background", of_the_start.log_background()),
        ("header_background", of_the_start.header_background()),
        ("line_header", of_the_start.line_header()),
        ("list_background", of_the_start.list_background()),
        (
            "list_background_alt_row",
            of_the_start.list_background_alt_row(),
        ),
        (
            "list_selected_background",
            of_the_start.list_selected_background(),
        ),
        (
            "list_selected_foreground",
            of_the_start.list_selected_foreground(),
        ),
        (
            "search_bar_foreground",
            of_the_start.search_bar_foreground(),
        ),
        ("login_foreground", of_the_start.login_foreground()),
        ("player_background", of_the_start.player_background()),
    ];

    for (name, colour) in all {
        assert!(
            of_the_terminal(colour),
            "the colour of the start of {name} is not a colour of the terminal of the user: {colour:?}"
        );
    }

    // **No background of the program paints over the terminal.** The five
    // backgrounds are the background of the terminal itself.
    assert_eq!(of_the_start.background(), Color::Reset);
    assert_eq!(of_the_start.log_background(), Color::Reset);
    assert_eq!(of_the_start.header_background(), Color::Reset);
    assert_eq!(of_the_start.list_background(), Color::Reset);
    assert_eq!(of_the_start.player_background(), Color::Reset);

    // **The two rows of a list hold one colour**, therefore no row of a list
    // stands out and the row of the cursor alone holds the accent.
    assert_eq!(of_the_start.list_background_alt_row(), Color::Reset);

    // The row of the cursor is the one row of a colour of its own, and the
    // letters of it stand on that accent.
    assert_eq!(of_the_start.list_selected_background(), Color::Cyan);
    assert_eq!(of_the_start.list_selected_foreground(), Color::Black);
}

/// A colour of the file of the user reaches the screen. See T-317.
///
/// **The parts of this test stay in one function.**
#[test]
fn a_colour_of_the_file_of_the_user_stays() {
    let of_the_user = Colors {
        background_color: vec![40, 40, 40],
        list_selected_background_color: vec![80, 80, 80],
        ..Colors::default()
    };

    assert_eq!(of_the_user.background(), Color::Rgb(40, 40, 40));
    assert_eq!(
        of_the_user.list_selected_background(),
        Color::Rgb(80, 80, 80)
    );

    // A key that the file does not hold keeps the colour of the terminal, and
    // the block of one key does not take the other colours away (T-122).
    assert_eq!(of_the_user.header_background(), Color::Reset);

    // **A list of fewer than three numbers keeps the rule of T-257**: the last
    // number of the list gives every component that the list does not hold.
    let of_two_numbers = Colors {
        background_color: vec![50, 50],
        ..Colors::default()
    };
    assert_eq!(of_two_numbers.background(), Color::Rgb(50, 50, 50));
}

/// The file that the program writes for a new user holds no colour of its own.
/// See T-317.
///
/// **This test is the gate of the real default.** `Colors::default` is the
/// value of `serde` for a key that the file does not hold, but the program
/// **writes** `config.example.toml` for a user who holds no file (T-122), and
/// that file held the eleven greys of RGB. A change of `Colors::default` alone
/// therefore reached no user at all: the first measurement of the corrected
/// program gave the same 28 escapes of RGB as the measurement of the fault.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_file_of_a_new_user_holds_no_colour_of_the_program() {
    let example = toutui::config::THE_EXAMPLE_OF_THE_CONFIGURATION;

    assert!(
        example.contains("[colors]"),
        "the example of the configuration names no block of the colours"
    );

    // Every key of the colours stands in the file for the user who wants a
    // colour of their own, and every one of them is off.
    let keys = [
        "background_color",
        "log_background_color",
        "header_background_color",
        "line_header_color",
        "list_background_color",
        "list_background_color_alt_row",
        "list_selected_background_color",
        "list_selected_foreground_color",
        "search_bar_foreground_color",
        "login_foreground_color",
        "player_background_color",
    ];

    for key in keys {
        let of_this_key: Vec<&str> = example
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with(key) || line.starts_with(&format!("#{key}")))
            .collect();

        assert!(
            !of_this_key.is_empty(),
            "the example of the configuration names no key {key}"
        );

        for line in of_this_key {
            assert!(
                line.starts_with('#'),
                "the file of a new user gives a colour of the program to {key}: {line:?}"
            );
        }
    }
}

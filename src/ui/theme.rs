//! The colours of the program.
//!
//! **The theme of the terminal of the user is the theme of the start** (T-317).
//! The program used 38 values of `Color::Rgb` and six names of ANSI before this
//! module, and `Colors::default` of `src/config.rs` gave each of the eleven
//! settings of the file a grey of RGB of its own: `background_color` was
//! `[40, 40, 40]`, `list_background_color` was `[50, 50, 50]`, and so on. The
//! program therefore painted a dark grey over the background of the terminal,
//! and a user of a theme of a light colour got the theme of this program and
//! not the theme of their terminal.
//!
//! **The measurement of the fault**, of the real program v0.8.144 inside tmux at
//! 100 columns and 30 rows, with `tmux capture-pane -e`: the first frame held
//! **28** escapes of a background of RGB — twelve of `48;2;40;40;40`, ten of
//! `48;2;50;50;50`, five of `48;2;60;60;60`, and one of `48;2;80;80;80` — and
//! **no escape of the background of the terminal at all**.
//!
//! **The two rules of this module**, of the section (d) of
//! `docs/mockups/mockup-1.md`:
//!
//! 1. **A colour of the program is one of eighteen values**: the foreground of
//!    the terminal, the background of the terminal, and the 16 names of ANSI.
//!    `Color::Reset` is the name of the two first ones for ratatui.
//! 2. **A grey is not a colour of a theme**: a grey of RGB stays grey on a
//!    background of a light colour and it then reads badly, therefore a text
//!    that must be quiet takes the foreground of the terminal with the modifier
//!    `DIM`, which follows the theme of the user.
//!
//! **The eleven settings of the file keep their work**: a user who writes
//! `background_color = [40, 40, 40]` gets that colour, and a value that the file
//! does not hold is the colour of the terminal. See `Colors::the_colour_of` of
//! `src/config.rs`.

use ratatui::style::{Color, Modifier, Style};

/// The one accent of the program.
///
/// **One accent alone** (the section (d) of `docs/mockups/mockup-1.md`): the
/// row of the cursor, the panel that holds the focus, the bar of the scroll,
/// and the part of the bar of the player that played take this colour, and no
/// other colour of this program holds more than one work.
pub const THE_ACCENT: Color = Color::Cyan;

/// The colour of a fault, and of a value that the user must see.
pub const A_FAULT: Color = Color::Red;

/// The colour of a media that is finished, and of a work that ended well.
pub const AN_END_THAT_IS_GOOD: Color = Color::Green;

/// The style of a title of a block, and of a name of a group.
pub fn a_title() -> Style {
    Style::default().fg(THE_ACCENT).add_modifier(Modifier::BOLD)
}

/// The style of a text that must be quiet.
///
/// **This is the grey of this program**, and it holds no grey at all: the
/// foreground of the terminal with the modifier `DIM` follows the theme of the
/// user, and a grey of RGB does not.
pub fn a_quiet_text() -> Style {
    Style::default()
        .fg(Color::Reset)
        .add_modifier(Modifier::DIM)
}

/// The style of a text of a fault.
pub fn a_text_of_a_fault() -> Style {
    Style::default().fg(A_FAULT)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every colour of this module is a colour of the terminal of the user or a
    /// name of ANSI. See T-317.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn every_colour_of_the_theme_is_a_colour_of_the_terminal() {
        let of_the_theme = [
            THE_ACCENT,
            A_FAULT,
            AN_END_THAT_IS_GOOD,
            a_title().fg.unwrap(),
            a_quiet_text().fg.unwrap(),
            a_text_of_a_fault().fg.unwrap(),
        ];

        for one in of_the_theme {
            assert!(
                !matches!(one, Color::Rgb(_, _, _) | Color::Indexed(_)),
                "the theme holds a colour that the theme of the terminal cannot give: {one:?}"
            );
        }

        // **The quiet text is the foreground of the terminal and not a grey**:
        // a grey of RGB stays grey on a background of a light colour.
        assert_eq!(a_quiet_text().fg, Some(Color::Reset));
        assert!(a_quiet_text().add_modifier.contains(Modifier::DIM));
    }
}

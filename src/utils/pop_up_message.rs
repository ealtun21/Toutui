//! A message that stands **outside** the buffer of ratatui.
//!
//! **The screen of the login is the only user of this module. See T-59.**
//!
//! A message of these functions goes to a row of the terminal, and ratatui knows
//! nothing about that row. Two faults come of it:
//!
//! 1. A view that draws the row takes the message away, therefore the user reads
//!    nothing.
//! 2. A view that draws no cell of that row keeps the bytes of the message,
//!    therefore an old message answers for a new key. See T-42.
//!
//! `crate::logic::message` is the answer for every view of the application: the
//! work writes the message in a slot, and the render draws it inside the frame.
//! **Do not call the two functions below from a view of the application.**
//!
//! The screen of the login draws no frame of the application. That screen holds
//! its own loop, and it writes its own line of the terminal. Therefore it keeps
//! these functions.

use crate::config::*;
use crossterm::{
    cursor, execute,
    style::{Color, SetBackgroundColor},
    terminal,
};
use std::io::{Result, Stdout};

// pop up message
pub fn pop_message(stdout: &mut Stdout, lines_from_bottom: u16, message: &str) -> Result<()> {
    // import backgorund color
    let mut color = Vec::new();
    if let Ok(cfg) = load_config() {
        color = cfg.colors.background_color;
    }

    let (_cols, rows) = terminal::size()?;
    let target_row = rows.saturating_sub(lines_from_bottom);
    let (r, g, b) = rgb_parts(&color);
    let bg_color = Color::Rgb { r, g, b };

    execute!(
        stdout,
        cursor::MoveTo(0, target_row),
        SetBackgroundColor(bg_color),
    )?;

    println!("{}", message);

    Ok(())
}

// to clear a pop up message
pub fn clear_message(stdout: &mut Stdout, lines_from_bottom: u16) -> Result<()> {
    // import backgorund color
    let mut color = Vec::new();
    if let Ok(cfg) = load_config() {
        color = cfg.colors.background_color;
    }
    let (_cols, rows) = terminal::size()?;
    let target_row = rows.saturating_sub(lines_from_bottom);
    let (r, g, b) = rgb_parts(&color);
    let bg_color = Color::Rgb { r, g, b };

    execute!(
        stdout,
        cursor::MoveTo(0, target_row),
        SetBackgroundColor(bg_color),
        terminal::Clear(terminal::ClearType::CurrentLine),
    )?;

    Ok(())
}

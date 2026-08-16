use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Paragraph, Widget},
};

/// Draws the panel of the player.
///
/// `notice` holds a short message of the engine. An example is "Reconnected".
pub fn render_player(
    area: Rect,
    buf: &mut ratatui::buffer::Buffer,
    player_info: Vec<String>,
    bg_color: Vec<u8>,
    // The user asked for the row of the keys of the player with the key `B`.
    // **The render reads no disk** (T-204): the `App` holds this value, and the
    // key `B` writes it and the disk together.
    the_key_bindings_stand: bool,
    notice: Option<String>,
    // The time that the timer for sleep has left, if a timer runs. See T-24.
    sleep: Option<String>,
) {
    let block_width = area.width;
    let new_y = area.y + area.height.saturating_sub(9); // the line number where player start
    let block_height = 4; // number of line of the player (in lines)

    // Create the background block with background color
    let bg_color_player = crate::config::Colors::the_colour_of(&bg_color, Color::Reset);
    let block_area = Rect::new(area.x, new_y, block_width, block_height);
    let block = Block::default().style(Style::default().bg(bg_color_player));

    // Text area
    let text_area_width = block_width - 6;
    let text_area_x = (area.width.saturating_sub(text_area_width)) / 2; // Center the text
    let text_area = Rect::new(text_area_x, new_y, text_area_width, block_height);

    // The engine waits for data, or the data came again. Tell the user.
    let notice = match notice {
        Some(message) => format!(" | {}", message),
        None => String::new(),
    };

    // The timer for sleep. The user must see the time that is left, or they
    // do not know why the playback stopped. See T-24.
    let sleep = match sleep {
        Some(text) => format!(" | {}", text),
        None => String::new(),
    };

    let mut key_bindings = "".to_string();
    if the_key_bindings_stand {
        key_bindings = "Spc: pause/play | p/u: +/−10s | P/U: nxt/prev ch. | O/I: spd +/− | o/i: vol +/− | t: sleep | Y: quit".to_string();
    }

    // **The panel of the player holds four rows, and the title, the author, and
    // the chapter come from the server** (T-312). A text of an end of a line
    // takes a row of its own, therefore the row of the keys of the player goes
    // outside the panel and no user sees it. `in_one_line` gives every end of a
    // line one space, and the four rows then stand.
    let title = crate::logic::message::in_one_line(&player_info[0]);
    let author = crate::logic::message::in_one_line(&player_info[1]);
    let chapter = crate::logic::message::in_one_line(&player_info[2]);

    // Create the paragraph
    let paragraph = Paragraph::new(format!(
        "\n{} by {} | {}{} \n {} {} / {} | Elapsed: {} | Left: {} ({}%) | Speed: {}x{}{}\n{}",
        title,   // Title
        author,  // Author
        chapter, // Chapter
        notice,  // The message of the engine
        match player_info[3].as_str() {
            "false" => "⏸".to_string(),
            "true" => "▶".to_string(),
            _ => "".to_string(),
        },
        player_info[4],  // Current time
        player_info[5],  // Total duration
        player_info[6],  // Elapsed time
        player_info[7],  // Remaining time
        player_info[8],  // Percent progress
        player_info[9],  // Speed rate
        player_info[10], // The volume, when it is not the volume of the file
        sleep,           // The timer for sleep
        key_bindings
    ))
    .centered()
    .block(Block::default());

    // Render the paragraph and background block
    paragraph.render(text_area, buf);
    block.render(block_area, buf);
}

//! The render of the sentence of a view that holds no line. See T-278.
//!
//! **A message that a view cuts is a message that says nothing** (T-277): the
//! reader of that item drew the reason of the machine into a `Paragraph` with
//! no `wrap`, and the user read `The machine said: [CannotRead` and no more.
//! The sweep of that item over the whole of `src/ui/` found two more sites of
//! the same shape, and both of them draw the sentence of a view of the episodes
//! of a podcast.
//!
//! The measurement, of the real program v0.8.106 inside tmux against the
//! sandbox, in a terminal of 80 columns: the podcast `Letters of Two Brides`
//! with `docs/harness/one_path_fails.py` on its path of the item gave
//!
//! ```text
//! The server did not give the episodes of this podcast: The server reported a faul
//!                                Press h to go back.
//! ```
//!
//! The whole sentence holds 94 characters, and the log held the reason that the
//! screen lost: `The server reported a fault. Status 500.` The same sentence at
//! 160 columns stood whole.
//!
//! The sentence of the offline mode of that view holds 94 characters too, and
//! it needs no server of a fault at all.
//!
//! The render of such a message needs the area, the text, and no other part of
//! `App`. It stands here as a function of its own, and the tests of this module
//! draw it into a `Buffer` and they read the characters of that buffer. A
//! `Buffer` needs no terminal and no screen, therefore those tests run in the
//! gate.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};

/// Draws the sentence of a view that holds no line, under the name of the list
/// that holds no line.
///
/// **The `wrap` is the rule of this function** (T-278): a sentence of a view
/// says why the view holds no line, and that sentence holds the words of the
/// server. A terminal of 80 columns is common, and a sentence of the program
/// that names what the server said goes past it. Without the `wrap` the widget
/// draws one row and it cuts the rest away, therefore the user loses the reason
/// and the program says a part of a sentence.
///
/// **The title is the second rule of this function** (T-358). The block held
/// one border at the top and no word in it, therefore the Episodes view of a
/// podcast that gave no episode drew a rule of 160 columns with nothing in it,
/// and no word of the screen said which list holds no line. `render_list` of
/// `crate::ui::the_list_of_a_view` draws the same rule with the same title for
/// a view of lines: the two roads of one view therefore say one name.
pub fn render_the_message(title: &str, text: &str, area: Rect, buf: &mut Buffer) {
    // **The title of a view with no line keeps its start** (T-373, and the
    // rule of T-304): ratatui gives a centered title that is wider than the
    // block a smaller area and it draws the title right-aligned in it,
    // therefore the title loses its start and its end together, with no mark
    // of the cut. The three points say that the screen cut it.
    let title = crate::logic::message::in_one_row(title, area.width);

    Paragraph::new(text)
        .centered()
        .wrap(Wrap { trim: true })
        .block(
            Block::new()
                .title(Line::raw(title).centered())
                .borders(Borders::TOP)
                .border_style(Style::new().fg(Color::DarkGray)),
        )
        .render(area, buf);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The words of a buffer, with no line and no colour.
    ///
    /// The wrap breaks a sentence at a space and the render puts each row in
    /// the middle of the width, therefore this function makes one space of
    /// every run of spaces and of every end of a row: a sentence of two rows
    /// then reads as one sentence.
    fn the_words_of(buffer: &Buffer) -> String {
        let mut words = String::new();

        for row in 0..buffer.area.height {
            for column in 0..buffer.area.width {
                words.push_str(buffer[(column, row)].symbol());
            }
            words.push(' ');
        }

        words.split_whitespace().collect::<Vec<&str>>().join(" ")
    }

    /// The sentence of the measurement of T-278, of a request that the server
    /// answered with the status 500.
    fn the_sentence_of_the_measurement() -> String {
        format!(
            "{}\nPress h to go back.",
            crate::logic::the_episodes::the_reason_of_no_episode(
                false,
                false,
                Some("The server reported a fault. Status 500."),
            )
        )
    }

    /// A sentence that is longer than the width of the terminal stands whole.
    ///
    /// The parts of this test stay in one function: the three widths are one
    /// measurement of one sentence.
    #[test]
    fn a_sentence_longer_than_the_screen_stands_on_more_than_one_row() {
        let text = the_sentence_of_the_measurement();

        // The whole sentence of the fault, with no room for a cut. The `wrap`
        // breaks it at a space, therefore the words of the buffer hold it in
        // one piece after `the_words_of` joins the rows.
        for width in [40u16, 60, 80] {
            let mut buffer = Buffer::empty(Rect::new(0, 0, width, 20));

            render_the_message(
                "Episodes [0 items]",
                &text,
                Rect::new(0, 0, width, 20),
                &mut buffer,
            );

            let words = the_words_of(&buffer);

            assert!(
                words.contains(
                    "The server did not give the episodes of this podcast: The server \
                     reported a fault. Status 500."
                ),
                "the screen of {width} columns holds the whole sentence: {words}"
            );
            assert!(
                words.contains("Press h to go back."),
                "the screen of {width} columns names the key of the road back: {words}"
            );
        }
    }

    /// A title that is wider than the screen keeps its start, and the three
    /// points say that the screen cut it. See T-373 and T-304.
    ///
    /// The words of a row of the buffer, in the sequence of the screen: the
    /// title stands on the first row, over the border.
    #[test]
    fn a_title_longer_than_the_screen_keeps_its_start() {
        let title = "The bookmarks of \"A Book Of An Epub With No Container\" [0 items]";
        let mut buffer = Buffer::empty(Rect::new(0, 0, 40, 20));

        render_the_message(
            title,
            "Press h to go back.",
            Rect::new(0, 0, 40, 20),
            &mut buffer,
        );

        let the_title_row: String = (0..40).map(|column| buffer[(column, 0)].symbol()).collect();

        assert!(
            the_title_row.contains("The bookmarks of"),
            "the title of 40 columns keeps its start: {the_title_row}"
        );
        assert!(
            the_title_row.contains('…'),
            "the three points say that the screen cut the title: {the_title_row}"
        );
    }

    /// The sentence of the offline mode of that view is 94 characters long, and
    /// it needs no server of a fault at all.
    #[test]
    fn the_sentence_of_the_offline_mode_stands_whole_at_80_columns() {
        let text = format!(
            "{}\nPress h to go back.",
            crate::logic::the_episodes::the_reason_of_no_episode(false, true, None)
        );

        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 20));

        render_the_message(
            "Episodes [0 items]",
            &text,
            Rect::new(0, 0, 80, 20),
            &mut buffer,
        );

        let words = the_words_of(&buffer);

        assert!(
            words.contains(
                "The server does not answer, therefore this program does not have the \
                 episodes of this podcast."
            ),
            "the screen holds the whole sentence of the offline mode: {words}"
        );
    }
}

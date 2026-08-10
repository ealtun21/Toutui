//! One line of text that the user writes.
//!
//! `tui-input` holds the text and the position of the cursor. It draws
//! nothing. This module makes the three values that a `Paragraph` needs: the
//! text to show, the number of columns that the field hides at the left, and
//! the column of the cursor inside the field.
//!
//! Every function here is pure. Therefore a test can examine the cursor, the
//! horizontal scroll, and the mask with no terminal. See T-33.

use tui_input::Input;

/// The three values that the caller gives to a `Paragraph` and to
/// `Frame::set_cursor_position`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldView {
    /// The text to show. A field with a mask shows the mask character.
    pub text: String,
    /// The number of columns that the field hides at the left.
    pub scroll: u16,
    /// The column of the cursor. The count starts at the left of the field,
    /// and it is always less than the width of the field.
    pub cursor: u16,
}

/// Makes the text that a field with a mask shows.
///
/// The function gives one mask character for each character of the value.
/// Therefore the length of the answer tells the length of the value, and the
/// answer tells nothing more.
pub fn mask_value(value: &str, mask: char) -> String {
    mask.to_string().repeat(value.chars().count())
}

/// Gives the number of columns that the field hides at the left.
///
/// The rule is simple: the cursor must stay inside the field. The function
/// hides the smallest number of columns that keeps the cursor visible.
pub fn field_scroll(cursor_column: usize, width: u16) -> u16 {
    if width == 0 {
        return 0;
    }
    let last_column = usize::from(width - 1);
    let scroll = cursor_column.saturating_sub(last_column);
    u16::try_from(scroll).unwrap_or(u16::MAX)
}

/// Makes the view of one field.
///
/// `width` is the width of the space inside the borders. `mask` is the
/// character that a password field shows in place of every character.
pub fn field_view(input: &Input, width: u16, mask: Option<char>) -> FieldView {
    // A field with a mask shows one column for each character, therefore the
    // count of the characters before the cursor is the column of the cursor.
    // A field with no mask can hold a wide character, therefore the crate
    // gives that column.
    let (text, cursor_column) = match mask {
        Some(mask) => (mask_value(input.value(), mask), input.cursor()),
        None => (input.value().to_string(), input.visual_cursor()),
    };

    let scroll = field_scroll(cursor_column, width);
    let cursor = u16::try_from(cursor_column - usize::from(scroll)).unwrap_or(u16::MAX);

    FieldView {
        text,
        scroll,
        cursor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input_at(value: &str, cursor: usize) -> Input {
        Input::new(value.to_string()).with_cursor(cursor)
    }

    #[test]
    fn a_mask_gives_one_character_for_each_character() {
        assert_eq!(mask_value("secret", '\u{2022}'), "••••••");
        assert_eq!(mask_value("", '\u{2022}'), "");
    }

    #[test]
    fn a_mask_counts_a_wide_character_one_time() {
        // A password can hold any character. The mask must not tell the width
        // of the character that it hides.
        assert_eq!(mask_value("日本", '*'), "**");
    }

    #[test]
    fn a_short_text_needs_no_scroll() {
        assert_eq!(field_scroll(0, 10), 0);
        assert_eq!(field_scroll(9, 10), 0);
    }

    #[test]
    fn a_long_text_hides_the_columns_at_the_left() {
        // The field is 10 columns wide, therefore the cursor can stand at
        // column 9 at the most. A cursor at column 12 needs 3 hidden columns.
        assert_eq!(field_scroll(10, 10), 1);
        assert_eq!(field_scroll(12, 10), 3);
    }

    #[test]
    fn a_field_with_no_width_gives_no_scroll() {
        assert_eq!(field_scroll(100, 0), 0);
    }

    #[test]
    fn the_cursor_stays_inside_the_field() {
        for cursor_column in 0..200 {
            for width in 1..40u16 {
                let scroll = field_scroll(cursor_column, width);
                let column = cursor_column - usize::from(scroll);
                assert!(
                    column < usize::from(width),
                    "the cursor left the field: column {} of {} at {} columns",
                    column,
                    width,
                    cursor_column
                );
            }
        }
    }

    #[test]
    fn a_field_shows_the_text_that_the_user_wrote() {
        let view = field_view(&input_at("hello", 5), 20, None);
        assert_eq!(
            view,
            FieldView {
                text: "hello".to_string(),
                scroll: 0,
                cursor: 5,
            }
        );
    }

    #[test]
    fn a_field_moves_with_the_cursor_of_a_long_text() {
        // 20 characters in a field of 10 columns. The cursor stands at the
        // end, therefore the field shows the last 10 columns.
        let value = "abcdefghijklmnopqrst";
        let view = field_view(&input_at(value, 20), 10, None);
        assert_eq!(view.text, value);
        assert_eq!(view.scroll, 11);
        assert_eq!(view.cursor, 9);
    }

    #[test]
    fn a_field_shows_the_start_when_the_cursor_goes_back() {
        let view = field_view(&input_at("abcdefghijklmnopqrst", 0), 10, None);
        assert_eq!(view.scroll, 0);
        assert_eq!(view.cursor, 0);
    }

    #[test]
    fn a_wide_character_takes_two_columns() {
        // Each of the two characters takes two columns, therefore the cursor
        // after them stands at column 4.
        let view = field_view(&input_at("日本", 2), 20, None);
        assert_eq!(view.cursor, 4);
    }

    #[test]
    fn a_password_field_shows_the_mask_and_not_the_password() {
        let view = field_view(&input_at("secret", 6), 20, Some('\u{2022}'));
        assert_eq!(view.text, "••••••");
        assert!(!view.text.contains("secret"));
        assert_eq!(view.cursor, 6);
    }

    #[test]
    fn a_long_password_moves_with_the_cursor() {
        let view = field_view(&input_at("0123456789abcdef", 16), 10, Some('*'));
        assert_eq!(view.text, "****************");
        assert_eq!(view.scroll, 7);
        assert_eq!(view.cursor, 9);
    }
}

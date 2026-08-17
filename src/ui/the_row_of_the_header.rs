//! The parts of one row of the header, and the room that the row has for them.
//! See T-340.
//!
//! **The header holds three parts on one row, and each of them is a paragraph
//! of its own over the whole area** (T-115): the account at the left, the name
//! of the library in the middle, and the name of the program at the right of
//! the first row, and the address of the server at the left, the words of the
//! sequence and of the filter in the middle, and the notice of the key `R` at
//! the right of the second row. A part that is too long therefore writes on the
//! letters of its neighbour.
//!
//! **T-115 gave the header a short form, and it measured nothing** (see
//! `crate::ui::keys::THE_WIDTH_OF_THE_LONG_HEADER`): the short form takes fewer
//! columns, and it still writes over its neighbours at every width under about
//! 54 columns. **T-329 corrected the middle of the second row alone**
//! (`crate::ui::the_panels_of_the_stack::the_column_of_the_words`), and its own
//! words name this fault "the fault of T-115 one row below".
//!
//! The measurement of the real program v0.8.171 inside tmux, of the Home view
//! of the library `Podcasts` of the sandbox at 40 columns:
//!
//! ```text
//! 👋 toutuitestPodcasts (podcas🦜 v0.8.171
//! ```
//!
//! The account wrote over the mark `📖` of the library, and the name of the
//! program wrote over the end of `(podcast)`. The same program of the offline
//! mode, at the same width:
//!
//! ```text
//! 📴 toutuiteste: the media on 🦜 v0.8.171
//! 🔗 localhost:133 R: try the server again
//! ```
//!
//! **The second row says an address that the user does not have**: the notice
//! of the key `R` cut `🔗 localhost:13399 does not answer` at the port, and the
//! header then named the port 133. A text that the row cuts says nothing to the
//! user (T-91), and a text that the row cuts into a **different** address says
//! something that is not true.
//!
//! The rule of this module is the rule of T-329, for every part of the two
//! rows: **a part stands whole with a gap of two columns from its neighbours,
//! or it does not stand at all**, and each row names the sequence in which its
//! parts go away.

use crate::logic::message::the_columns_of;

/// The gap between two parts of a row of the header.
///
/// **It is the gap of T-329**, which is the gap that the row of the address
/// held at 80 columns before that item: a screen that stood stands in the same
/// shape.
pub const THE_GAP: u16 = crate::ui::the_panels_of_the_stack::THE_GAP_OF_THE_WORDS;

/// One part of a row of the header.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThePart {
    AtTheLeft,
    InTheMiddle,
    AtTheRight,
}

/// The column where each part of a row starts, and `None` for a part that the
/// row does not hold.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ThePlaces {
    pub at_the_left: Option<u16>,
    pub in_the_middle: Option<u16>,
    pub at_the_right: Option<u16>,
}

/// The columns of a text of the header.
///
/// **`String::len` gives the bytes and not the columns** (the trap 245): the
/// marks `👋`, `📖`, `📴`, and `🦜` of this header take four bytes and two
/// columns each.
pub fn the_columns(text: &str) -> u16 {
    u16::try_from(the_columns_of(text)).unwrap_or(u16::MAX)
}

/// The room that a set of parts needs, with a gap between each two of them.
fn the_room_that_they_need(of_the_left: u16, of_the_middle: u16, of_the_right: u16) -> u16 {
    let the_parts = [of_the_left, of_the_middle, of_the_right];
    let how_many = the_parts.iter().filter(|part| **part > 0).count();

    let the_texts = the_parts
        .iter()
        .fold(0u16, |all, part| all.saturating_add(*part));

    // A row of one part holds no gap at all, and a row of three parts holds
    // two of them.
    let the_gaps = THE_GAP.saturating_mul(u16::try_from(how_many.saturating_sub(1)).unwrap_or(0));

    the_texts.saturating_add(the_gaps)
}

/// Says where each part of one row of the header stands.
///
/// The columns of the three parts come from [`the_columns`], and a part of 0
/// columns is a part that the row has nothing to draw for.
///
/// `the_sequence_of_the_going_away` names the parts in the sequence in which
/// they leave the row: the first of them goes away first. Each row of the
/// header names a sequence of its own, because the value of a part belongs to
/// the row and not to this function.
///
/// **The middle keeps the middle of the row while the middle is free** (T-329),
/// therefore every screen that stands today stands in the same shape. It stands
/// beside the part at the left when the middle is not free.
pub fn the_places_of_a_row(
    width: u16,
    of_the_left: u16,
    of_the_middle: u16,
    of_the_right: u16,
    the_sequence_of_the_going_away: [ThePart; 3],
) -> ThePlaces {
    let mut left = of_the_left;
    let mut middle = of_the_middle;
    let mut right = of_the_right;

    // A part goes away while the row has no room for the parts that stand.
    for one in the_sequence_of_the_going_away {
        if the_room_that_they_need(left, middle, right) <= width {
            break;
        }

        match one {
            ThePart::AtTheLeft => left = 0,
            ThePart::InTheMiddle => middle = 0,
            ThePart::AtTheRight => right = 0,
        }
    }

    // A row that has no room for the last part of the sequence holds no part at
    // all: a text that the row cuts says nothing to the user (T-91).
    if the_room_that_they_need(left, middle, right) > width {
        return ThePlaces::default();
    }

    // **A part that is away leaves no gap at the edge of the row**: the gap
    // stands between two texts, and the border of the screen is no text.
    let first = if left > 0 {
        left.saturating_add(THE_GAP)
    } else {
        0
    };

    let after_the_last = width.saturating_sub(if right > 0 {
        right.saturating_add(THE_GAP)
    } else {
        0
    });

    let in_the_middle = (middle > 0).then(|| {
        // The middle of the whole row, which is the place that
        // `Paragraph::centered` of ratatui gives.
        let of_the_middle = width.saturating_sub(middle) / 2;

        if of_the_middle >= first && of_the_middle.saturating_add(middle) <= after_the_last {
            of_the_middle
        } else {
            first
        }
    });

    ThePlaces {
        at_the_left: (left > 0).then_some(0),
        in_the_middle,
        at_the_right: (right > 0).then(|| width.saturating_sub(right)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The sequence of the going away of the first row of the header: the name
    /// of the program first, and the account after it. See T-340.
    const OF_THE_FIRST_ROW: [ThePart; 3] = [
        ThePart::AtTheRight,
        ThePart::AtTheLeft,
        ThePart::InTheMiddle,
    ];

    /// A row that holds every part draws it in the place that it had before
    /// this correction. See T-340.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn a_row_of_room_keeps_the_places_of_today() {
        // The header of 160 columns: the account of 27 columns, the library of
        // 15, and the name of the program of 18.
        let places = the_places_of_a_row(160, 27, 15, 18, OF_THE_FIRST_ROW);

        assert_eq!(places.at_the_left, Some(0));
        assert_eq!(places.in_the_middle, Some((160 - 15) / 2));
        assert_eq!(places.at_the_right, Some(160 - 18));

        // The middle stands where `Paragraph::centered` puts it, therefore no
        // screen of this width changes.
        assert!(places.in_the_middle.unwrap() >= 27 + THE_GAP);
        assert!(places.in_the_middle.unwrap() + 15 <= 160 - 18 - THE_GAP);
    }

    /// The gap of two columns is the one that decides, and not the letters
    /// alone. See T-340.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn two_parts_that_touch_take_the_gap_of_two_columns() {
        // The three parts hold 40 columns together, and the two gaps need four
        // columns more.
        assert_eq!(the_room_that_they_need(13, 15, 12), 44);

        // The row of 40 columns therefore loses the part at the right, and the
        // two parts that stay hold a gap of two columns.
        let places = the_places_of_a_row(40, 13, 15, 12, OF_THE_FIRST_ROW);

        assert_eq!(places.at_the_left, Some(0));
        assert_eq!(places.at_the_right, None);

        // The middle of the row stands at the column 12, and the account ends
        // at the column 13: the middle therefore leaves the middle and it
        // stands beside the account.
        assert_eq!(places.in_the_middle, Some(13 + THE_GAP));

        // A row of 44 columns holds the three of them.
        let of_the_room = the_places_of_a_row(44, 13, 15, 12, OF_THE_FIRST_ROW);
        assert_eq!(of_the_room.at_the_right, Some(44 - 12));
        assert_eq!(of_the_room.in_the_middle, Some(13 + THE_GAP));
        assert_eq!(of_the_room.in_the_middle.unwrap() + 15, 44 - 12 - THE_GAP);
    }

    /// The parts go away in the sequence that the row names, and a row of no
    /// room for the last of them holds nothing at all. See T-340.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_parts_go_away_in_the_sequence_of_the_row() {
        // The offline mode of 40 columns: the account of 13 columns, the words
        // of the offline mode of 33, and the name of the program of 12.
        let places = the_places_of_a_row(40, 13, 33, 12, OF_THE_FIRST_ROW);

        // The name of the program goes away first, and the account after it:
        // the words of the offline mode are the last part of the sequence.
        assert_eq!(places.at_the_right, None);
        assert_eq!(places.at_the_left, None);
        assert_eq!(places.in_the_middle, Some((40 - 33) / 2));

        // The second row of the header names another sequence: the address of
        // the server is the last part to go away, because a row that cuts it
        // says an address that the user does not have.
        const OF_THE_SECOND_ROW: [ThePart; 3] = [
            ThePart::InTheMiddle,
            ThePart::AtTheRight,
            ThePart::AtTheLeft,
        ];

        // The address of 34 columns and the notice of 24 columns.
        let of_the_address = the_places_of_a_row(40, 34, 0, 24, OF_THE_SECOND_ROW);
        assert_eq!(of_the_address.at_the_left, Some(0));
        assert_eq!(of_the_address.at_the_right, None);

        // A row that holds no part at all: the address alone is longer than it.
        let of_nothing = the_places_of_a_row(20, 34, 0, 24, OF_THE_SECOND_ROW);
        assert_eq!(of_nothing, ThePlaces::default());

        // A row of no width holds nothing, and no arithmetic of it goes under
        // zero.
        assert_eq!(
            the_places_of_a_row(0, 13, 15, 12, OF_THE_FIRST_ROW),
            ThePlaces::default()
        );
    }

    /// A part that the row has nothing to draw for leaves no gap of its own.
    /// See T-340.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn a_part_of_no_column_leaves_no_gap() {
        // The notice of the key `R` is empty at most moments of the program.
        assert_eq!(the_room_that_they_need(18, 0, 0), 18);
        assert_eq!(the_room_that_they_need(18, 0, 6), 18 + THE_GAP + 6);

        // A row of the address alone holds it at the column 0, and the row
        // gives every column after it to nothing.
        let places = the_places_of_a_row(
            20,
            18,
            0,
            0,
            [
                ThePart::InTheMiddle,
                ThePart::AtTheRight,
                ThePart::AtTheLeft,
            ],
        );

        assert_eq!(places.at_the_left, Some(0));
        assert_eq!(places.in_the_middle, None);
        assert_eq!(places.at_the_right, None);

        // The middle of a row of no part at the left starts at the column 0
        // when the row is exactly as wide as it.
        let of_the_middle = the_places_of_a_row(
            15,
            0,
            15,
            0,
            [
                ThePart::AtTheRight,
                ThePart::AtTheLeft,
                ThePart::InTheMiddle,
            ],
        );

        assert_eq!(of_the_middle.in_the_middle, Some(0));
    }

    /// The columns of a text of the header count the marks of it as two
    /// columns. See T-340 and the trap 245.
    ///
    /// **The parts of this test stay in one function.**
    #[test]
    fn the_columns_of_a_mark_are_two() {
        // `👋 toutuitest`: the mark of two columns, the space, and ten letters.
        assert_eq!(the_columns("👋 toutuitest"), 13);
        assert_eq!(the_columns("📖 Large (book)"), 15);
        assert_eq!(the_columns("🦜 v0.8.171"), 11);

        // The bytes of that text are more than its columns.
        assert!("👋 toutuitest".len() > usize::from(the_columns("👋 toutuitest")));
    }
}

//! The panel 6 of the gallery takes the rows that the panel 5 of the cover does
//! not need, and it takes no row that the facts of the media need. See T-350.
//!
//! # The fault, of the real program v0.8.180 inside tmux
//!
//! The Home view of the library `Books` of the sandbox, at 160 columns, with
//! `tmux resize-window -t check -x 160 -y N` after the first frame. The media of
//! the cursor is `A Long Test Book`, and the facts of the design of it take nine
//! lines: the author, the narrator, the length, the day of the start, the genre,
//! the files, the ebook, the place of the user, and the bar of the progress.
//!
//! At **27 rows** the panel 5 said the nine lines and the description under
//! them, and no gallery stood:
//!
//! ```text
//! │Author    Long Author                     │
//! │Narrator  A Test Narrator                 │
//! │Time      30m, 15m left                   │
//! │Started   17 Aug 2026                     │
//! │Genre     Fiction, Adventure              │
//! │Files     1 file, 7.0 MB                  │
//! │Ebook     epub                            │
//! │Progress  50%, Not finished               │
//! │█████████████████████░░░░░░░░░░░░░░░░░░░░░│
//! │No description available                  │
//! ```
//!
//! At **28 rows**, one row **more** of the screen, the gallery stood and the
//! panel 5 said five lines of the nine:
//!
//! ```text
//! │Author    Long Author                     │
//! │Narrator  A Test Narrator                 │
//! │Time      30m, 15m left                   │
//! │Started   17 Aug 2026                     │
//! │Genre     Fiction, Adventure              │
//! └──────────────────────────────────────────┘
//! ┌6 Gallery ────────────────────────────────┐
//! ```
//!
//! The place of the user and the bar of the progress went away, and T-325 says
//! that the two of them always take a line. A screen that is taller said less
//! about the media.
//!
//! # Why
//!
//! `the_two_panels` read the constant `THE_SMALLEST_PANEL_OF_THE_COVER`, which
//! is `THE_SMALLEST_PANEL_OF_THE_WORDS + 2`, and that number holds
//! `THE_ROWS_OF_THE_FACTS`, which is **three**. The facts of the design take a
//! line each (T-325), and this book gives nine of them. The gallery therefore
//! left the panel 5 fifteen rows of a column of twenty-three, and the panel then
//! cut four lines of its facts.
//!
//! # The other face of the same fault
//!
//! The library `Large` of the sandbox holds its media **with no cover at all**
//! (`GET /api/items/:id/cover` answers 404), and the facts of such a media take
//! three lines. The panel 5 of it needs no row for a picture (T-319 and T-349),
//! therefore seven rows say the whole of it. The constant reserved fifteen: at
//! **27 rows** the gallery went away, the panel 5 said its three lines, and
//! **twenty rows of that panel held no character at all**.
//!
//! # The correction
//!
//! One pure function, `the_whole_panel_of_the_cover(a_picture_comes,
//! of_the_facts)`, gives the rows that the panel 5 needs for the whole of its
//! words, and `the_two_panels` reads that function. The picture and the
//! description keep their smallest number of rows, because each of them says its
//! words in the rows that it has; the facts hold no such rule.

use ratatui::layout::Rect;
use ratatui_image::FontSize;
use toutui::ui::the_panel_of_the_gallery::{
    the_smallest_gallery, the_two_panels, the_whole_panel_of_the_cover, THE_WIDTHS_OF_A_CELL,
    THE_WIDTH_OF_THE_START,
};

/// The font of the terminal of the measurement: a cell of 8 pixels by 16.
const FONT: FontSize = FontSize {
    width: 8,
    height: 16,
};

/// The rows of the facts of `A Long Test Book` of the library `Books`, of the
/// measurement of the real program.
const OF_THE_FACTS_OF_THE_BOOK: u16 = 9;

/// The rows of the facts of `Large Book 0001` of the library `Large`, which the
/// server holds with no cover.
const OF_THE_FACTS_WITH_NO_COVER: u16 = 3;

/// The rows of the column of the covers of a screen of 28 rows, of the
/// measurement: the header takes two rows, the row of the message one, and the
/// footer two.
const THE_COLUMN_OF_28_ROWS: u16 = 23;

/// The rows of the column of the covers of a screen of 27 rows.
const THE_COLUMN_OF_27_ROWS: u16 = 22;

/// The rows that the panel 5 needs for the whole of its words.
///
/// **The numbers of this test are the numbers of the measurement, and not the
/// value of the function that it measures** (T-349): a test that takes its own
/// bounds of that function cannot fail.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_panel_of_the_cover_needs_the_rows_of_its_own_facts() {
    // The border takes two rows, the picture eight, the facts nine, and the
    // description two.
    assert_eq!(
        the_whole_panel_of_the_cover(true, OF_THE_FACTS_OF_THE_BOOK),
        21,
        "a media of nine facts with a picture needs 2 + 8 + 9 + 2 rows"
    );

    // A media that the server holds with no cover keeps no row for a picture
    // (T-319 and T-349): the border takes two rows, the facts three, and the
    // description two.
    assert_eq!(
        the_whole_panel_of_the_cover(false, OF_THE_FACTS_WITH_NO_COVER),
        7,
        "a media of three facts with no picture needs 2 + 0 + 3 + 2 rows"
    );

    // The value of the program before this item, which the constant held: a
    // picture and the three rows of the facts under the list.
    assert_eq!(
        the_whole_panel_of_the_cover(true, OF_THE_FACTS_WITH_NO_COVER),
        15
    );

    // A media of more facts needs more rows, and it never needs fewer.
    let mut before = 0;

    for of_the_facts in 0..12 {
        let rows = the_whole_panel_of_the_cover(true, of_the_facts);
        assert!(
            rows >= before,
            "the facts of {of_the_facts} lines need fewer rows"
        );
        before = rows;
    }
}

/// The gallery goes away before the panel 5 cuts a line of its facts, and it
/// stands under a panel that says the whole of its words already.
///
/// **The parts of this test stay in one function.**
#[test]
fn the_gallery_stands_under_a_panel_that_says_its_words_whole() {
    let of_a_cell = THE_WIDTHS_OF_A_CELL[THE_WIDTH_OF_THE_START];

    // The panel 6 of the measurement held eight rows: the border of two, and
    // one row of the grid of six.
    assert_eq!(the_smallest_gallery(of_a_cell, FONT), 8);

    // **The fault**: a book of nine facts, at the column of a screen of 28
    // rows. The gallery took eight rows of the twenty-three, the panel 5 kept
    // fifteen, and the facts of that book then lost four of their nine lines.
    let of_the_book = Rect::new(111, 2, 50, THE_COLUMN_OF_28_ROWS);
    assert_eq!(
        the_two_panels(of_the_book, of_a_cell, FONT, true, OF_THE_FACTS_OF_THE_BOOK),
        (of_the_book, None),
        "the panel 5 of a book of nine facts needs 21 rows, and the gallery \
         needs 8 more of a column of 23"
    );

    // The same column says the whole of a media of three facts, and the gallery
    // then stands under it.
    let (cover, gallery) = the_two_panels(
        of_the_book,
        of_a_cell,
        FONT,
        true,
        OF_THE_FACTS_WITH_NO_COVER,
    );
    let gallery = gallery.expect("a panel 5 of 15 rows and a gallery of 8 fill a column of 23");
    assert_eq!(cover.height, 15);
    assert_eq!(gallery.height, 8);

    // **The other face**: a media that the server holds with no cover, at the
    // column of a screen of 27 rows. The panel 5 of it needs seven rows, and
    // the gallery of eight stands under it. The program before this item gave
    // that panel every row of the column, and twenty of them held nothing.
    let with_no_cover = Rect::new(111, 2, 50, THE_COLUMN_OF_27_ROWS);
    let (cover, gallery) = the_two_panels(
        with_no_cover,
        of_a_cell,
        FONT,
        false,
        OF_THE_FACTS_WITH_NO_COVER,
    );
    let gallery = gallery.expect("a panel 5 of a media with no cover needs 7 rows of the 22");
    assert_eq!(cover.height + gallery.height, THE_COLUMN_OF_27_ROWS);
    assert!(
        cover.height >= 7,
        "the panel 5 holds {} rows and the words of it need 7",
        cover.height
    );
    assert_eq!(gallery.height, 8);

    // The rule over every column of the sweep of the measurement, for the two
    // media and for every width of a cell of the design: a gallery that stands
    // leaves the panel 5 the rows of the whole of its words.
    for of_a_cell in THE_WIDTHS_OF_A_CELL {
        for (a_picture_comes, of_the_facts, needs) in [
            (true, OF_THE_FACTS_OF_THE_BOOK, 21),
            (false, OF_THE_FACTS_WITH_NO_COVER, 7),
            (true, OF_THE_FACTS_WITH_NO_COVER, 15),
        ] {
            for height in 1..60 {
                let column = Rect::new(111, 2, 50, height);
                let (cover, gallery) =
                    the_two_panels(column, of_a_cell, FONT, a_picture_comes, of_the_facts);

                let Some(gallery) = gallery else {
                    assert_eq!(cover, column, "a column of no gallery keeps every row");
                    continue;
                };

                assert!(
                    cover.height >= needs,
                    "the panel 5 of a media of {of_the_facts} facts holds {} rows \
                     of a column of {height}, and the words of it need {needs}",
                    cover.height
                );
                assert_eq!(cover.height + gallery.height, height);
                assert!(gallery.height >= the_smallest_gallery(of_a_cell, FONT));
            }
        }
    }
}

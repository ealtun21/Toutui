//! The view of the chapters holds the media that the user opened. See T-162.
//!
//! **The media that plays changes while that view stands open, and no key of the
//! user does it**: the media comes to its end, and the queue starts the media of
//! its front. The list of the chapters is then the list of another media, and the
//! line keeps the number of the line. The measurement of 2026-08-14: the user
//! chose "The third part" of a book of 30 minutes, the queue started a book of
//! eight hours with three chapters of its own, and the key `l` took that book
//! from 4:50:35 to 5:33:20. The server holds that place.
//!
//! **The loop of the program is the one place of that work**, because no key of
//! this user comes at that moment: the loop calls the rule at each frame, beside
//! the rule of the queue of T-161. This test reads the source, as the tests of
//! T-135, T-143, and T-161 do: a session that takes the call away takes the rule
//! away with it.

/// The loop of the program holds the rule at each frame. See T-162.
#[test]
fn the_loop_of_the_program_reads_the_media_of_the_chapters_at_each_frame() {
    let source = include_str!("../src/main.rs");

    assert!(
        source.contains("app.the_view_of_the_chapters_holds_its_media();"),
        "the loop of src/main.rs must hold the media of the view of the \
         chapters at each frame: the media that plays changes with no key of \
         the user, and the key l then seeks in a media that the user did not \
         choose (T-162)"
    );

    let start = source
        .find("app.tick_the_timer_for_sleep();")
        .expect("the loop of the program holds the timer for sleep");
    // The draw of the loop, and not the draw of a screen of the start: the
    // program draws the login screen and the screen of the load before it.
    let end = start
        + source[start..]
            .find("terminal.draw(|frame| {")
            .expect("the loop of the program draws the screen");

    let before_the_draw = &source[start..end];

    assert!(
        before_the_draw.contains("app.the_view_of_the_chapters_holds_its_media();"),
        "the rule must stand before the draw of the frame: the user must read \
         the line of the same frame that names the media that went away (T-162)"
    );
}

/// The view opens with the media of this moment, therefore the program reads
/// that media again. See T-162.
#[test]
fn the_view_that_opens_reads_the_media_that_plays() {
    let source = include_str!("../src/app.rs");

    let start = source
        .find("pub fn show_the_chapters(&mut self) {")
        .expect("the program shows the chapters");
    let block = &source[start..start + 1500];

    assert!(
        block.contains("self.the_media_of_the_view_of_the_chapters = None;"),
        "the view opens with the media that plays now, therefore the program \
         must read that media again (T-162)"
    );
}

/// A key that does nothing must say why. See T-79 and T-162.
#[test]
fn the_key_of_the_view_says_why_it_does_nothing() {
    let source = include_str!("../src/app.rs");

    let start = source
        .find("pub fn go_to_the_chapter(&mut self) {")
        .expect("the program goes to a chapter");
    let block = &source[start..start + 1200];

    assert!(
        block.contains("No line is selected."),
        "the line of the view stands on nobody after the media went away, and \
         the key l must then say why it does nothing (T-79 and T-162)"
    );
}

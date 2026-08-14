//! The cursor of the view of the queue holds the media that the user chose. See
//! T-161.
//!
//! **The queue changes while that view stands open, and no key of the user does
//! it**: the media that plays comes to its end and the queue takes the media of
//! the front away, and a second program of the account takes a media out with
//! the key `X`. The lines keep the number of the line, therefore a media that
//! the user did not choose moved under the cursor with no word at all. The
//! measurement of 2026-08-14: the user chose the line 2, the media that played
//! came to its end, and the key `X` then took the media of the line 3 out of the
//! queue. The key `l` played that media and it stopped the media that the queue
//! had started.
//!
//! **The loop of the program is the one place of that work**, because no key of
//! this user comes at that moment: the loop calls the rule at each frame, beside
//! the timer for sleep. This test reads the source, as the tests of T-135 and of
//! T-143 do: a session that takes the call away takes the rule away with it.

/// The loop of the program holds the rule at each frame. See T-161.
#[test]
fn the_loop_of_the_program_reads_the_line_of_the_queue_at_each_frame() {
    let source = include_str!("../src/main.rs");

    assert!(
        source.contains("app.the_line_of_the_queue_holds_its_media();"),
        "the loop of src/main.rs must hold the media of the line of the queue \
         at each frame: the queue moves with no key of the user, and a key of \
         the selection then reaches a media that the user did not choose (T-161)"
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
        before_the_draw.contains("app.the_line_of_the_queue_holds_its_media();"),
        "the rule must stand before the draw of the frame: the user must read \
         the line of the same frame that names the media that went away (T-161)"
    );
}

/// The two keys of the view act on the line of the user, and the program reads
/// the media of the new line after them. See T-161.
#[test]
fn the_keys_of_the_view_give_the_media_of_the_line_back() {
    let source = include_str!("../src/app.rs");

    for key in [
        "pub fn remove_from_the_queue(&mut self) {",
        "pub fn start_the_media_of_the_queue(&mut self) {",
        "pub fn show_the_queue(&mut self) {",
    ] {
        let start = source.find(key).unwrap_or_else(|| panic!("{}", key));
        let block = &source[start..start + 3000];

        assert!(
            block.contains("self.the_media_of_the_line_of_the_queue = None;"),
            "the media of the line goes with this key of the user, therefore \
             the program must read the media of the new line again: {}",
            key
        );
    }
}

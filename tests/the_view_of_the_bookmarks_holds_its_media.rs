//! The view of the bookmarks holds the media that the user opened. See T-163.
//!
//! **The media that plays changes while that view stands open, and no key of the
//! user does it**: the media comes to its end, and the queue starts the media of
//! its front (T-24). The list of this view holds the bookmarks of the media that
//! the user opened, and the key `b` wrote a place of the media that **plays**.
//! The measurement of 2026-08-14: the user read the one bookmark of a book of 30
//! minutes, the queue started a book of eight hours, and the key `b` wrote a
//! bookmark of that book at 19530 seconds. The view showed the same one line
//! before the key and after it, and the title of the view named no media at all.
//!
//! The rule stands in the key, and not in the loop of the program: no line of
//! this view moves under the cursor, therefore the user reads the truth of their
//! view until they press `b`. This test reads the source, as the tests of T-135,
//! T-143, T-161, and T-162 do: a session that takes the guard away takes the rule
//! away with it.

/// The key `b` of this view writes a place of the media of this view alone. See
/// T-163.
#[test]
fn the_key_that_writes_a_bookmark_holds_the_media_of_the_view() {
    let source = include_str!("../src/app.rs");

    let start = source
        .find("pub fn write_a_bookmark(&mut self) {")
        .expect("the program writes a bookmark");
    let end = start
        + source[start..]
            .find("pub fn show_the_bookmarks(&mut self) {")
            .expect("the program shows the bookmarks");
    let block = &source[start..end];

    assert!(
        block.contains("what_the_media_of_the_bookmarks_is("),
        "the key b of the view of the bookmarks must ask if the media of that \
         view plays: the queue starts another media with no key of the user, \
         and the key then writes a place of a media that the user did not \
         choose (T-163)"
    );

    assert!(
        block.contains("AppView::Bookmarks"),
        "the rule belongs to the view of the bookmarks: the key b of every \
         other view writes a place of the media that plays (T-163)"
    );

    assert!(
        block.contains("the_text_of_the_media_that_does_not_play("),
        "a key that does nothing must say why (T-79 and T-163)"
    );
}

/// The view holds the name of its media, therefore the title names it. See
/// T-163.
#[test]
fn the_view_that_opens_holds_the_name_of_its_media() {
    let source = include_str!("../src/app.rs");

    let start = source
        .find("pub fn show_the_bookmarks(&mut self) {")
        .expect("the program shows the bookmarks");
    let block = &source[start..start + 1500];

    assert!(
        block.contains("self.bookmarks_of_name = name;"),
        "the view must hold the name of the media that the user opened: the \
         title of the view names it, and the key b names it (T-163)"
    );
}

/// The title of the view names the media of the view. See T-163.
#[test]
fn the_title_of_the_view_names_the_media() {
    let source = include_str!("../src/ui/tui.rs");

    let start = source
        .find("fn render_bookmarks(&mut self, area: Rect, buf: &mut Buffer) {")
        .expect("the program draws the bookmarks");
    let block = &source[start..start + 2500];

    assert!(
        block.contains("crate::logic::bookmarks::the_title(")
            && block.contains("self.bookmarks_of_name"),
        "the title must name the media of the view: the media that plays \
         changes with no key of the user, and a title of \"The bookmarks\" \
         alone leaves the user with no way to tell whose places they read \
         (T-163)"
    );
}

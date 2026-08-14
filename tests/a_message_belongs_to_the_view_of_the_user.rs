//! A message of a view belongs to that view, and to no other view. See T-164.
//!
//! Three rules of the loop of `src/main.rs` write a message with **no key of the
//! user**: the shelf Continue Listening of the Home view (T-160), the line of
//! the view of the queue (T-161), and the media of the view of the chapters
//! (T-162). Each of them wrote its text to the one slot of the message, and the
//! last writer won whatever view the user was looking at.
//!
//! The measurement of 2026-08-14: the user stood in the view of the queue, the
//! media that played came to its end, the line of their cursor went to nobody
//! (T-161), and the six seconds of their screen said `The media "A Long Test
//! Book" is not on the shelf Continue Listening now.` — the sentence of a view
//! that they were not in. **The sentence of T-161 never reached the screen.**
//!
//! This test reads the source, as the tests of T-135, T-143, T-161, T-162, and
//! T-163 do: a session that gives one of the three rules the plain `say` again
//! takes the rule away with it.

/// The three rules of the loop name the view of their message. See T-164.
#[test]
fn the_rules_of_the_loop_say_their_text_in_their_own_view() {
    let source = include_str!("../src/app.rs");

    for (rule, view, text) in [
        (
            "the shelf Continue Listening of the Home view (T-160)",
            "AppView::Home",
            "crate::logic::home_view::the_text_of_the_media_that_went_away",
        ),
        (
            "the line of the view of the queue (T-161)",
            "AppView::Queue",
            "crate::logic::queue::the_text_of_the_media_that_went_away",
        ),
        (
            "the media of the view of the chapters (T-162)",
            "AppView::Chapters",
            "crate::logic::chapters::the_text_of_the_media_that_went_away",
        ),
    ] {
        let at = source
            .find(text)
            .unwrap_or_else(|| panic!("the program must hold the text of {}", rule));
        // The call of the message stands above the text of it.
        let start = at.saturating_sub(200);
        let block = &source[start..at];

        assert!(
            block.contains("crate::logic::message::say_in(") && block.contains(view),
            "the rule of {} writes its message with no key of the user, and the \
             user can stand in any view: it must name {} (T-164)",
            rule,
            view
        );
    }
}

/// The render names the view of the user. See T-164.
#[test]
fn the_render_names_the_view_of_the_user() {
    let source = include_str!("../src/ui/tui.rs");

    assert!(
        source.contains("crate::logic::message::for_the_screen(self.view_state)"),
        "the render must name the view of the user: a message of a view waits \
         for that view, and a render that names no view gives the message of \
         every view to every user (T-164)"
    );
}

/// A message of a view waits for that view, and its life starts when the user
/// reads it. See T-164.
#[test]
fn a_message_of_a_view_waits_for_its_view() {
    use toutui::app::AppView;
    use toutui::logic::message;

    message::forget();

    // The two rules of one frame, and the user stands in the view of the queue.
    message::say_in(AppView::Queue, "The media is not in the queue now.");
    message::say_in(AppView::Home, "The media is not on the shelf now.");

    assert_eq!(
        message::for_the_screen(AppView::Queue).as_deref(),
        Some("The media is not in the queue now."),
        "the user of the view of the queue must read the sentence of their own \
         view (T-164)"
    );

    // The user comes to the Home view later, and the reason of that view stands
    // there: the line of that view is still on nobody.
    assert_eq!(
        message::for_the_screen(AppView::Home).as_deref(),
        Some("The media is not on the shelf now."),
        "a message of a view must wait for that view (T-164)"
    );

    // A view of no message of its own shows nothing.
    assert_eq!(message::for_the_screen(AppView::Chapters), None);

    message::forget();
}

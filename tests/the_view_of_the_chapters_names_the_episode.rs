//! The view of the chapters names the episode of a podcast. See T-227.
//!
//! **The title of a playback of a podcast is the name of the podcast**, and
//! every episode of that podcast holds it (T-223). The three sentences of the
//! view of the chapters read that title alone, therefore the two episodes of one
//! podcast gave one set of words.
//!
//! The measurement of the real program v0.8.55 inside tmux, against the sandbox
//! (podman on :13399), of the podcast `Arthur Gordon Pym` of the library
//! `Podcasts`: the user played `Chapter 01` and pressed the key `C`, and the
//! header said `"Arthur Gordon Pym" holds no chapter. Press h to go back.` while
//! the row of the player of that same frame said
//! `Arthur Gordon Pym — Chapter 01`. The queue then started `Chapter 00` of that
//! same podcast with no key of the user: the sentence of T-162 said
//! `The media "Arthur Gordon Pym" does not play now.`, and two seconds later the
//! header said `"Arthur Gordon Pym" holds no chapter.` again while the row said
//! `Arthur Gordon Pym — Chapter 00`. **The program named the podcast three
//! times, and the user could not tell which episode the view holds.**
//!
//! **The parts of this test stay in one function**: two test functions of one
//! module fight for the slot of that module, and `cargo test` then finds a fault
//! that nextest hides (T-144 and T-157).
//!
//! The three functions are pure, therefore this test needs no server and no
//! screen. **Three builds of the fault each fail it**: a name of a media that
//! drops the episode, a header of the view that reads the title alone, and the
//! sentence of the media that went away that reads the title alone.

use toutui::logic::chapters::{
    the_header_of_the_view, the_reason_of_no_chapter, the_text_of_the_media_that_went_away,
};
use toutui::logic::media_name::the_name_of_the_media;

/// The name of the podcast of the measurement.
const THE_PODCAST: &str = "Arthur Gordon Pym";

/// The two episodes of the measurement. Both of them hold the name of the
/// podcast above, and the program said that name for each of them.
const THE_EPISODE: &str = "Chapter 01";
const THE_SECOND_EPISODE: &str = "Chapter 00";

/// The name of a media, the two headers of the view, and the sentence of the
/// media that went away each name the episode. See T-227.
#[test]
fn the_three_sentences_of_the_view_name_the_episode() {
    // The name of a media, of `crate::logic::media_name`. The row of the player
    // reads it since T-225, and the view of the chapters reads it now.
    assert_eq!(
        the_name_of_the_media(THE_PODCAST, Some(THE_EPISODE)),
        "Arthur Gordon Pym — Chapter 01",
        "the name of an episode holds the podcast and the episode (T-225)"
    );
    assert_eq!(
        the_name_of_the_media("A Long Test Book", None),
        "A Long Test Book",
        "a media with no name of an episode keeps its own name alone (T-91)"
    );

    // The header of a media that holds chapters. **The two episodes of one
    // podcast must not give one header.**
    let of_the_first = the_header_of_the_view(THE_PODCAST, Some(THE_EPISODE), 3, false);
    let of_the_second = the_header_of_the_view(THE_PODCAST, Some(THE_SECOND_EPISODE), 3, false);

    assert_eq!(
        of_the_first, "The chapters of \"Arthur Gordon Pym — Chapter 01\" [3 items]",
        "the header of the chapters of an episode names that episode (T-227)"
    );
    assert_ne!(
        of_the_first, of_the_second,
        "two episodes of one podcast must not give one header: the queue starts \
         a second episode with no key of the user (T-225 and T-227)"
    );

    // The reason of a media that holds no chapter. The episodes of the sandbox
    // give 0 chapters, therefore this is the sentence of the measurement.
    //
    // **The sentence of a view with no line stands in the body of the panel
    // and never in the title of it** (T-361): a title takes no wrap, and the
    // words of this sentence went away in a narrow terminal.
    let of_no_chapter = the_reason_of_no_chapter(THE_PODCAST, Some(THE_EPISODE), false);

    assert_eq!(
        of_no_chapter, "\"Arthur Gordon Pym — Chapter 01\" holds no chapter. Press h to go back.",
        "the sentence of an episode of no chapter names that episode (T-227)"
    );
    assert_ne!(
        of_no_chapter,
        the_reason_of_no_chapter(THE_PODCAST, Some(THE_SECOND_EPISODE), false),
        "two episodes of one podcast must not give one sentence of no chapter \
         (T-227)"
    );

    // **The header of that same view names the episode too** (T-227 and
    // T-361): the name of the list stands in the title, with the count of its
    // lines.
    assert_eq!(
        the_header_of_the_view(THE_PODCAST, Some(THE_EPISODE), 0, false),
        "The chapters of \"Arthur Gordon Pym — Chapter 01\" [0 items]",
        "the header of an episode of no chapter names that episode (T-227)"
    );
    assert_ne!(
        the_header_of_the_view(THE_PODCAST, Some(THE_EPISODE), 0, false),
        the_header_of_the_view(THE_PODCAST, Some(THE_SECOND_EPISODE), 0, false),
        "two episodes of one podcast must not give one header of no chapter \
         (T-227)"
    );

    // A media that plays no more names no media at all: the program then holds
    // the name of no media of a chapter. See T-59.
    assert_eq!(
        the_reason_of_no_chapter(THE_PODCAST, Some(THE_EPISODE), true),
        "No media plays now. A media that plays gives its chapters. Press h to go back.",
        "a playback that stopped names no media (T-59)"
    );
    assert_eq!(
        the_header_of_the_view(THE_PODCAST, Some(THE_EPISODE), 0, true),
        "The chapters [0 items]",
        "the header of a playback that stopped names no media either (T-59)"
    );

    // A book keeps its own name in the two headers.
    assert_eq!(
        the_header_of_the_view("A Long Test Book", None, 3, false),
        "The chapters of \"A Long Test Book\" [3 items]",
        "a book keeps its own name alone (T-91)"
    );
    assert_eq!(
        the_reason_of_no_chapter("A Long Test Book", None, false),
        "\"A Long Test Book\" holds no chapter. Press h to go back.",
        "a book of no chapter keeps its own name alone (T-91)"
    );

    // The sentence of the media that went away, of T-162. **The queue starts a
    // second episode of that same podcast with no key of the user** (T-225),
    // therefore this sentence must name the episode that went away.
    let text = the_text_of_the_media_that_went_away(THE_PODCAST, Some(THE_EPISODE));

    assert!(
        text.contains("Arthur Gordon Pym — Chapter 01"),
        "the sentence of the media that went away names the episode (T-227): {}",
        text
    );
    assert_ne!(
        text,
        the_text_of_the_media_that_went_away(THE_PODCAST, Some(THE_SECOND_EPISODE)),
        "the episode that went away and the episode of the queue must not give \
         one sentence (T-227)"
    );

    // The rules of T-118, T-143, and T-162 stay: the sentence promises the two
    // keys of the view alone.
    assert!(text.contains("keys j and k"), "{}", text);
    assert!(!text.contains("press h"), "{}", text);

    // A book keeps its own name in that sentence too.
    assert!(
        the_text_of_the_media_that_went_away("A Long Test Book", None)
            .contains("The media \"A Long Test Book\" does not play now."),
        "a book keeps its own name alone (T-91)"
    );
}

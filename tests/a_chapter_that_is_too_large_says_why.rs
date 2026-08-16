//! T-281: a chapter that is larger than the limit of the reader says what the
//! program measured, and it gives the user a road.
//!
//! The measurement, of the real program v0.8.109 inside tmux against the
//! sandbox, of a book of three chapters whose second chapter holds 9437361
//! bytes of plain text. `docs/harness/a_book_of_a_chapter_that_is_too_large.py`
//! writes that book, and **the data of this fault is a book: it needs no proxy
//! at all**. The book stood in the cache of the ebooks of the account
//! `toutuitest`, under the name of the item of `Alice in Wonderland`, because a
//! book of the cache costs no request of the server.
//!
//! The keys `Tab`, 15 keys `j`, `e`, and `p` gave
//!
//! ```text
//! This chapter is too large.
//! ```
//!
//! and the file of the log held **no line of the reader at all**: 14 lines
//! before the key, and 14 lines after it.
//!
//! The three faults of that one sentence:
//!
//! 1. **The sentence names no number.** `ReaderError::BookTooLarge` of the
//!    same file names the size of the file and the limit of the book. The
//!    sentence of the chapter names neither, therefore the user cannot know
//!    whether the book is hostile or the limit is low.
//! 2. **The sentence names no key.** The view of the reader holds `n` for the
//!    chapter after this one, `p` for the chapter before it, and `h` to leave
//!    the book, and each of the three does the work of this fault (T-170).
//!    `ReaderError::TheArchiveGaveNoChapter` of the same file names the key
//!    `n` already.
//! 3. **The arm takes no line of the log** (T-177). The arm beside it, of a
//!    chapter that the archive did not give, writes one already.
//!
//! **The program measured that the chapter passed the limit, and it did not
//! measure the size of it**: `CappedWriter` stops the copy at
//! `MAX_CHAPTER_BYTES`, therefore "more than" is the one number that this
//! program has, and the size of the whole chapter is a fact that it does not
//! have (T-91).
//!
//! The corrected program of the same condition said the sentence on three
//! rows, and the log held two lines of the reader: one of the pass of
//! `chapter_sizes` at the open of the book, and one of the render of that
//! chapter. The controls of the same run: the key `n` gave chapter 3 and its
//! text, the key `p` twice gave chapter 1 and its text, the key `h` gave the
//! Library view, and the good book of that name gave
//! `Alice's Adventures in Wonderland — chapter 2 of 14` with no line of the
//! reader in the log.

use toutui::logic::reader::book::{ReaderError, MAX_CHAPTER_BYTES};

/// The sentence says what the program measured, and it says no size that the
/// program does not have.
#[test]
fn the_sentence_says_what_the_program_measured() {
    let text = ReaderError::ChapterTooLarge.to_string();

    assert!(
        text.contains(&MAX_CHAPTER_BYTES.to_string()),
        "the sentence must name the limit of one chapter: {text}"
    );
    assert!(
        text.contains("more than"),
        "the program measured no size of the whole chapter: {text}"
    );
    assert_ne!(
        "This chapter is too large.", text,
        "the sentence is the one of before the correction"
    );
}

/// The sentence names the three keys of the view of the reader, and the file
/// of the log.
#[test]
fn the_sentence_gives_the_road_back() {
    let text = ReaderError::ChapterTooLarge.to_string();

    for key in ["Press n ", "or p ", "Press h "] {
        assert!(
            text.contains(key),
            "the sentence does not name {key}: {text}"
        );
    }
    assert!(
        text.contains("file of the log"),
        "the sentence does not name the log: {text}"
    );
}

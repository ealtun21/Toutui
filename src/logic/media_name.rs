//! The name of a media for the user. See T-225 and T-227.
//!
//! **The title of a playback of a podcast is the name of the podcast**, and
//! every episode of that podcast holds it (T-223). A part of the screen that
//! shows that title alone therefore says the same words for every episode of
//! one podcast, and the user cannot tell which episode it names.
//!
//! The function here is pure, therefore a test needs no engine and no screen.

/// Gives the name of a media, with the name of its episode.
///
/// The row of the player said `Arthur Gordon Pym by LibriVox` for `Chapter 00`
/// and, after the queue started a second episode with no key of the user,
/// `Arthur Gordon Pym by LibriVox` again: the length of the row was the one
/// value that moved, and a length names no episode (T-225).
///
/// The view of the chapters said the same words in three sentences (T-227):
/// the header `The chapters of "Arthur Gordon Pym"`, the header
/// `"Arthur Gordon Pym" holds no chapter.` of a media of no chapter, and the
/// sentence of T-162, `The media "Arthur Gordon Pym" does not play now.`, which
/// stood on the screen while the row of the player of that same frame said that
/// `Arthur Gordon Pym — Chapter 00` plays.
///
/// A media with no name of an episode keeps its own name alone: a book, and an
/// episode whose name the server did not give (T-91).
pub fn the_name_of_the_media(title: &str, episode_title: Option<&str>) -> String {
    match episode_title {
        Some(episode) => format!("{} — {}", title, episode),
        None => title.to_string(),
    }
}
